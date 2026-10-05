use std::collections::BTreeMap;

use crate::models::DecisionRecord;

use super::classification_spec_model::ClassificationSpec;
use super::decision_answer_model::DecisionAnswer;
use super::decision_backend::DecisionBackend;
use super::decision_error_model::DecisionError;
use super::decision_result_model::DecisionResult;
use super::label_probabilities_model::LabelProbabilities;
use super::question_wire_map::{decode_answer, spec_for_question};
use super::state_chunking::chunk_state;

pub struct DecisionRuntime<B: DecisionBackend> {
    backend: B,
    model: String,
}

impl<B: DecisionBackend> DecisionRuntime<B> {
    pub fn new(backend: B, model: impl Into<String>) -> Self {
        Self {
            backend,
            model: model.into(),
        }
    }

    pub fn answer(
        &self,
        page: Option<String>,
        record: &DecisionRecord,
    ) -> Result<DecisionResult, DecisionError> {
        let specs: Vec<ClassificationSpec> =
            record.questions.iter().map(spec_for_question).collect();
        let chunks = chunk_state(&record.state);
        let chunk_count = chunks.len();
        let aggregated = self.classify_chunks(&specs, &chunks)?;

        let mut answers: BTreeMap<String, DecisionAnswer> = BTreeMap::new();
        for (question, probabilities) in record.questions.iter().zip(aggregated.iter()) {
            answers.insert(question.id.clone(), decode_answer(question, probabilities)?);
        }

        Ok(DecisionResult {
            page,
            model: self.model.clone(),
            answers,
            chunks: chunk_count,
            truncated: chunk_count > 1,
        })
    }

    fn classify_chunks(
        &self,
        specs: &[ClassificationSpec],
        chunks: &[String],
    ) -> Result<Vec<LabelProbabilities>, DecisionError> {
        let mut sums: Vec<BTreeMap<String, f64>> = specs.iter().map(|_| BTreeMap::new()).collect();
        for text in chunks {
            let results = self.backend.classify(text, specs)?;
            accumulate(&mut sums, specs, &results)?;
        }
        Ok(average(specs, &sums, chunks.len()))
    }
}

fn accumulate(
    sums: &mut [BTreeMap<String, f64>],
    specs: &[ClassificationSpec],
    results: &[LabelProbabilities],
) -> Result<(), DecisionError> {
    for (index, spec) in specs.iter().enumerate() {
        let result = results
            .iter()
            .find(|result| result.task == spec.task)
            .ok_or_else(|| DecisionError::MissingTaskResult(spec.task.clone()))?;
        if result.labels != spec.labels || result.probabilities.len() != spec.labels.len() {
            return Err(DecisionError::ProbabilityMismatch {
                task: spec.task.clone(),
            });
        }
        let bucket = &mut sums[index];
        for (label, probability) in result.labels.iter().zip(result.probabilities.iter()) {
            *bucket.entry(label.clone()).or_insert(0.0) += f64::from(*probability);
        }
    }
    Ok(())
}

fn average(
    specs: &[ClassificationSpec],
    sums: &[BTreeMap<String, f64>],
    chunk_count: usize,
) -> Vec<LabelProbabilities> {
    let divisor = chunk_count.max(1) as f64;
    specs
        .iter()
        .enumerate()
        .map(|(index, spec)| LabelProbabilities {
            task: spec.task.clone(),
            labels: spec.labels.clone(),
            probabilities: spec
                .labels
                .iter()
                .map(|label| {
                    let sum = sums[index].get(label).copied().unwrap_or(0.0);
                    (sum / divisor) as f32
                })
                .collect(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decision::canonical_questions::canonical_questions;
    use crate::models::{AnswerValue, DecisionRecord, PageType, RECORD_SCHEMA_VERSION};

    struct MockBackend {
        probabilities: BTreeMap<String, Vec<f32>>,
    }

    impl DecisionBackend for MockBackend {
        fn classify(
            &self,
            _text: &str,
            specs: &[ClassificationSpec],
        ) -> Result<Vec<LabelProbabilities>, DecisionError> {
            Ok(specs
                .iter()
                .map(|spec| LabelProbabilities {
                    task: spec.task.clone(),
                    labels: spec.labels.clone(),
                    probabilities: self
                        .probabilities
                        .get(&spec.task)
                        .cloned()
                        .unwrap_or_else(|| vec![0.0; spec.labels.len()]),
                })
                .collect())
        }
    }

    fn record(state: &str) -> DecisionRecord {
        DecisionRecord {
            schema_version: RECORD_SCHEMA_VERSION,
            state: state.to_owned(),
            questions: canonical_questions(&PageType::Decision),
            answers: BTreeMap::new(),
        }
    }

    fn decision_backend() -> MockBackend {
        MockBackend {
            probabilities: BTreeMap::from([
                ("outcome".to_owned(), vec![0.1, 0.7, 0.1, 0.05, 0.05]),
                ("reversibility".to_owned(), vec![0.2, 0.8]),
                ("confidence".to_owned(), vec![0.1, 0.2, 0.7]),
                ("impact".to_owned(), vec![0.6, 0.2, 0.1, 0.1]),
            ]),
        }
    }

    #[test]
    fn answers_every_question_with_probabilities_and_distributions() {
        let runtime = DecisionRuntime::new(decision_backend(), "gliner2.5-small-v1");
        let result = runtime
            .answer(Some("wiki:decisions:x".to_owned()), &record("short body"))
            .expect("answer should succeed");

        assert_eq!(result.model, "gliner2.5-small-v1");
        assert_eq!(result.chunks, 1);
        assert!(!result.truncated);
        assert_eq!(result.answers.len(), 4);
        assert_eq!(
            result.answers["outcome"].value,
            AnswerValue::Label("rejected".to_owned())
        );
        assert_eq!(
            result.answers["reversibility"].value,
            AnswerValue::Bool(true)
        );
        assert_eq!(
            result.answers["confidence"].value,
            AnswerValue::Label("high".to_owned())
        );
        assert_eq!(
            result.answers["impact"].value,
            AnswerValue::Label("local".to_owned())
        );
        assert!(result.answers["outcome"].distribution.is_some());
    }

    #[test]
    fn aggregates_long_state_by_mean_per_label() {
        let words: Vec<String> = (0..1500).map(|index| format!("w{index}")).collect();
        let long_state = words.join(" ");
        let runtime = DecisionRuntime::new(decision_backend(), "mock");
        let result = runtime
            .answer(None, &record(&long_state))
            .expect("answer should succeed");

        assert!(result.chunks > 1);
        assert!(result.truncated);
        assert_eq!(
            result.answers["outcome"].value,
            AnswerValue::Label("rejected".to_owned())
        );
    }

    #[test]
    fn averages_sums_by_chunk_count() {
        let specs = vec![ClassificationSpec {
            task: "outcome".to_owned(),
            prompt: "Choose.".to_owned(),
            labels: vec!["a".to_owned(), "b".to_owned()],
            mode: super::super::classification_mode_model::ClassificationMode::Softmax,
        }];
        let sums = vec![BTreeMap::from([
            ("a".to_owned(), 1.0_f64),
            ("b".to_owned(), 0.5_f64),
        ])];
        let averaged = average(&specs, &sums, 2);
        assert!((averaged[0].probabilities[0] - 0.5).abs() < 1e-6);
        assert!((averaged[0].probabilities[1] - 0.25).abs() < 1e-6);
    }

    #[test]
    fn errors_when_backend_omits_a_task() {
        struct Empty;
        impl DecisionBackend for Empty {
            fn classify(
                &self,
                _text: &str,
                _specs: &[ClassificationSpec],
            ) -> Result<Vec<LabelProbabilities>, DecisionError> {
                Ok(Vec::new())
            }
        }
        let runtime = DecisionRuntime::new(Empty, "mock");
        let error = runtime
            .answer(None, &record("short"))
            .expect_err("missing task must fail");
        assert!(matches!(error, DecisionError::MissingTaskResult(_)));
    }
}
