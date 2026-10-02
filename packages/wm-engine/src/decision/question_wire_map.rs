use std::collections::BTreeMap;

use crate::models::{AnswerValue, QType, Question};

use super::classification_mode_model::ClassificationMode;
use super::classification_spec_model::ClassificationSpec;
use super::decision_answer_model::DecisionAnswer;
use super::decision_error_model::DecisionError;
use super::label_probabilities_model::LabelProbabilities;

pub const MULTI_LABEL_THRESHOLD: f32 = 0.5;
pub const NOUL_FALSE: &str = "false";
pub const NOUL_TRUE: &str = "true";

/// Map a record question onto the gliner-rs wire contract: `task` = id,
/// `prompt` = instructions, `labels` = options/levels (noul => false/true),
/// and `multi` => sigmoid at the schema threshold.
pub fn spec_for_question(question: &Question) -> ClassificationSpec {
    let (labels, mode) = match question.qtype {
        QType::Choice if question.multi => (
            question.options.clone(),
            ClassificationMode::Sigmoid {
                threshold: MULTI_LABEL_THRESHOLD,
            },
        ),
        QType::Choice => (question.options.clone(), ClassificationMode::Softmax),
        QType::Score => (question.levels.clone(), ClassificationMode::Softmax),
        QType::Noul => (
            vec![NOUL_FALSE.to_owned(), NOUL_TRUE.to_owned()],
            ClassificationMode::Softmax,
        ),
    };
    ClassificationSpec {
        task: question.id.clone(),
        prompt: question.instructions.clone(),
        labels,
        mode,
    }
}

/// Decode backend probabilities back into a typed answer. The value for
/// `score` is the level label, for `noul` a bool (reporting P(true)), and for
/// `choice` a label or a label list when `multi`.
pub fn decode_answer(
    question: &Question,
    probabilities: &LabelProbabilities,
) -> Result<DecisionAnswer, DecisionError> {
    if probabilities.task != question.id {
        return Err(DecisionError::MissingTaskResult(question.id.clone()));
    }
    let spec = spec_for_question(question);
    if probabilities.labels != spec.labels || probabilities.probabilities.len() != spec.labels.len()
    {
        return Err(DecisionError::ProbabilityMismatch {
            task: question.id.clone(),
        });
    }
    let distribution = distribution(&spec.labels, &probabilities.probabilities);
    let index =
        argmax(&probabilities.probabilities).ok_or_else(|| DecisionError::ProbabilityMismatch {
            task: question.id.clone(),
        })?;

    let (value, probability) = match spec.mode {
        ClassificationMode::Softmax => {
            softmax_answer(question, &spec.labels, &probabilities.probabilities, index)
        }
        ClassificationMode::Sigmoid { threshold } => {
            sigmoid_answer(&spec.labels, &probabilities.probabilities, threshold)
        }
    };

    Ok(DecisionAnswer {
        value,
        probability,
        distribution: Some(distribution),
    })
}

fn softmax_answer(
    question: &Question,
    labels: &[String],
    probabilities: &[f32],
    index: usize,
) -> (AnswerValue, f64) {
    if question.qtype != QType::Noul {
        return (
            AnswerValue::Label(label_at(labels, index)),
            probability_at(probabilities, index),
        );
    }
    let true_index = labels.iter().position(|label| label == NOUL_TRUE);
    let probability = true_index
        .map(|position| probability_at(probabilities, position))
        .unwrap_or_default();
    let answer = true_index == Some(index);
    (AnswerValue::Bool(answer), probability)
}

fn sigmoid_answer(labels: &[String], probabilities: &[f32], threshold: f32) -> (AnswerValue, f64) {
    let selected: Vec<String> = labels
        .iter()
        .zip(probabilities.iter())
        .filter(|(_, probability)| **probability >= threshold)
        .map(|(label, _)| label.clone())
        .collect();
    let probability = labels
        .iter()
        .zip(probabilities.iter())
        .filter(|(label, _)| selected.contains(label))
        .map(|(_, probability)| f64::from(*probability))
        .fold(0.0_f64, f64::max);
    (AnswerValue::Labels(selected), probability)
}

fn label_at(labels: &[String], index: usize) -> String {
    labels.get(index).cloned().unwrap_or_default()
}

fn probability_at(probabilities: &[f32], index: usize) -> f64 {
    probabilities
        .get(index)
        .map(|probability| f64::from(*probability))
        .unwrap_or_default()
}

fn distribution(labels: &[String], probabilities: &[f32]) -> BTreeMap<String, f64> {
    labels
        .iter()
        .cloned()
        .zip(
            probabilities
                .iter()
                .map(|probability| f64::from(*probability)),
        )
        .collect()
}

fn argmax(probabilities: &[f32]) -> Option<usize> {
    let mut best: Option<(usize, f32)> = None;
    for (index, probability) in probabilities.iter().copied().enumerate() {
        match best {
            Some((_, best_probability)) if probability <= best_probability => continue,
            _ => best = Some((index, probability)),
        }
    }
    best.map(|(index, _)| index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Question;

    fn choice(id: &str, options: &[&str], multi: bool) -> Question {
        Question {
            id: id.to_owned(),
            qtype: QType::Choice,
            instructions: "Choose one.".to_owned(),
            multi,
            options: options.iter().map(|option| (*option).to_owned()).collect(),
            levels: Vec::new(),
        }
    }

    fn score(id: &str, levels: &[&str]) -> Question {
        Question {
            id: id.to_owned(),
            qtype: QType::Score,
            instructions: "Pick a level.".to_owned(),
            multi: false,
            options: Vec::new(),
            levels: levels.iter().map(|level| (*level).to_owned()).collect(),
        }
    }

    fn noul(id: &str) -> Question {
        Question {
            id: id.to_owned(),
            qtype: QType::Noul,
            instructions: "This is true.".to_owned(),
            multi: false,
            options: Vec::new(),
            levels: Vec::new(),
        }
    }

    fn probabilities(task: &str, labels: &[&str], values: &[f32]) -> LabelProbabilities {
        LabelProbabilities {
            task: task.to_owned(),
            labels: labels.iter().map(|label| (*label).to_owned()).collect(),
            probabilities: values.to_vec(),
        }
    }

    #[test]
    fn maps_choice_score_and_noul_to_wire_specs() {
        let choice_spec = spec_for_question(&choice("outcome", &["a", "b"], false));
        assert_eq!(choice_spec.task, "outcome");
        assert_eq!(choice_spec.prompt, "Choose one.");
        assert_eq!(choice_spec.labels, vec!["a".to_owned(), "b".to_owned()]);
        assert_eq!(choice_spec.mode, ClassificationMode::Softmax);

        let multi_spec = spec_for_question(&choice("surface", &["a", "b"], true));
        assert_eq!(
            multi_spec.mode,
            ClassificationMode::Sigmoid {
                threshold: MULTI_LABEL_THRESHOLD
            }
        );

        let score_spec = spec_for_question(&score("confidence", &["low", "high"]));
        assert_eq!(score_spec.labels, vec!["low".to_owned(), "high".to_owned()]);
        assert_eq!(score_spec.mode, ClassificationMode::Softmax);

        let noul_spec = spec_for_question(&noul("reversibility"));
        assert_eq!(
            noul_spec.labels,
            vec![NOUL_FALSE.to_owned(), NOUL_TRUE.to_owned()]
        );
        assert_eq!(noul_spec.mode, ClassificationMode::Softmax);
    }

    #[test]
    fn decodes_single_choice_argmax_with_distribution() {
        let question = choice("outcome", &["adopted", "rejected"], false);
        let answer = decode_answer(
            &question,
            &probabilities("outcome", &["adopted", "rejected"], &[0.2, 0.8]),
        )
        .expect("decode should succeed");
        assert_eq!(answer.value, AnswerValue::Label("rejected".to_owned()));
        assert!((answer.probability - 0.8).abs() < 1e-6);
        let distribution = answer.distribution.expect("distribution present");
        assert!((distribution["adopted"] - 0.2).abs() < 1e-6);
    }

    #[test]
    fn breaks_argmax_ties_by_label_order() {
        let question = choice("outcome", &["adopted", "rejected"], false);
        let answer = decode_answer(
            &question,
            &probabilities("outcome", &["adopted", "rejected"], &[0.5, 0.5]),
        )
        .expect("decode should succeed");
        assert_eq!(answer.value, AnswerValue::Label("adopted".to_owned()));
    }

    #[test]
    fn decodes_multi_choice_above_threshold() {
        let question = choice("surface", &["mcp-tool", "cli-command", "http-api"], true);
        let answer = decode_answer(
            &question,
            &probabilities(
                "surface",
                &["mcp-tool", "cli-command", "http-api"],
                &[0.7, 0.4, 0.6],
            ),
        )
        .expect("decode should succeed");
        assert_eq!(
            answer.value,
            AnswerValue::Labels(vec!["mcp-tool".to_owned(), "http-api".to_owned()])
        );
        assert!((answer.probability - 0.7).abs() < 1e-6);
    }

    #[test]
    fn decodes_noul_as_bool_and_reports_probability_of_true() {
        let question = noul("reversibility");
        let answer = decode_answer(
            &question,
            &probabilities("reversibility", &[NOUL_FALSE, NOUL_TRUE], &[0.9, 0.1]),
        )
        .expect("decode should succeed");
        assert_eq!(answer.value, AnswerValue::Bool(false));
        assert!((answer.probability - 0.1).abs() < 1e-6);
    }

    #[test]
    fn decodes_score_as_level_label() {
        let question = score("confidence", &["low", "medium", "high"]);
        let answer = decode_answer(
            &question,
            &probabilities("confidence", &["low", "medium", "high"], &[0.1, 0.3, 0.6]),
        )
        .expect("decode should succeed");
        assert_eq!(answer.value, AnswerValue::Label("high".to_owned()));
        assert!((answer.probability - 0.6).abs() < 1e-6);
    }

    #[test]
    fn rejects_label_mismatch() {
        let question = choice("outcome", &["adopted", "rejected"], false);
        let error = decode_answer(
            &question,
            &probabilities("outcome", &["adopted", "other"], &[0.5, 0.5]),
        )
        .expect_err("mismatched labels must fail");
        assert_eq!(
            error,
            DecisionError::ProbabilityMismatch {
                task: "outcome".to_owned()
            }
        );
    }

    #[test]
    fn rejects_wrong_task() {
        let question = choice("outcome", &["a", "b"], false);
        let error = decode_answer(&question, &probabilities("other", &["a", "b"], &[0.5, 0.5]))
            .expect_err("wrong task must fail");
        assert_eq!(
            error,
            DecisionError::MissingTaskResult("outcome".to_owned())
        );
    }
}
