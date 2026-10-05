use std::path::Path;

use wm_engine::{
    ClassificationMode, ClassificationSpec, DecisionBackend, DecisionError, LabelProbabilities,
};

pub const GLINER_OFFLINE_ENV: &str = "GLINER_OFFLINE";

pub struct GlinerBackend {
    model: gliner_rs::GLiNER2,
}

impl GlinerBackend {
    pub fn load(model_dir: &Path) -> Result<Self, DecisionError> {
        std::env::set_var(GLINER_OFFLINE_ENV, "1");
        let device = candle_core::Device::Cpu;
        let model = gliner_rs::GLiNER2::load(model_dir, &device, candle_core::DType::F32).map_err(
            |error| DecisionError::Backend {
                detail: error.to_string(),
            },
        )?;
        Ok(Self { model })
    }
}

impl DecisionBackend for GlinerBackend {
    fn classify(
        &self,
        text: &str,
        specs: &[ClassificationSpec],
    ) -> Result<Vec<LabelProbabilities>, DecisionError> {
        let wire_specs: Vec<gliner_rs::ClassificationSpec> =
            specs.iter().map(to_wire_spec).collect();
        let results = self
            .model
            .classification_probabilities(text, &wire_specs)
            .map_err(|error| DecisionError::Backend {
                detail: error.to_string(),
            })?;
        specs
            .iter()
            .map(|spec| to_label_probabilities(spec, &results))
            .collect()
    }
}

fn to_wire_spec(spec: &ClassificationSpec) -> gliner_rs::ClassificationSpec {
    let mut wire =
        gliner_rs::ClassificationSpec::new(spec.task.clone(), spec.labels.iter().cloned());
    wire.prompt = Some(spec.prompt.clone());
    match spec.mode {
        ClassificationMode::Softmax => {
            wire.activation = gliner_rs::ClassActivation::Softmax;
        }
        ClassificationMode::Sigmoid { threshold } => {
            wire.multi_label = true;
            wire.cls_threshold = threshold;
            wire.activation = gliner_rs::ClassActivation::Sigmoid;
        }
    }
    wire
}

fn to_label_probabilities(
    spec: &ClassificationSpec,
    results: &[(String, Vec<(String, f32)>)],
) -> Result<LabelProbabilities, DecisionError> {
    let pairs = results
        .iter()
        .find(|(task, _)| task == &spec.task)
        .map(|(_, pairs)| pairs)
        .ok_or_else(|| missing_result(&spec.task))?;
    let probabilities = spec
        .labels
        .iter()
        .map(|label| {
            pairs
                .iter()
                .find(|(name, _)| name == label)
                .map(|(_, probability)| *probability)
                .ok_or_else(|| missing_label(&spec.task, label))
        })
        .collect::<Result<Vec<f32>, DecisionError>>()?;
    Ok(LabelProbabilities {
        task: spec.task.clone(),
        labels: spec.labels.clone(),
        probabilities,
    })
}

fn missing_result(task: &str) -> DecisionError {
    DecisionError::Backend {
        detail: format!("backend returned no result for task '{task}'"),
    }
}

fn missing_label(task: &str, label: &str) -> DecisionError {
    DecisionError::Backend {
        detail: format!("backend omitted label '{label}' for task '{task}'"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> ClassificationSpec {
        ClassificationSpec {
            task: "outcome".to_owned(),
            prompt: "What is the recorded outcome of this decision?".to_owned(),
            labels: vec!["adopted".to_owned(), "rejected".to_owned()],
            mode: ClassificationMode::Softmax,
        }
    }

    #[test]
    fn maps_present_task_and_labels_in_spec_order() {
        let results = vec![(
            "outcome".to_owned(),
            vec![
                ("rejected".to_owned(), 0.8_f32),
                ("adopted".to_owned(), 0.2_f32),
            ],
        )];
        let decoded = to_label_probabilities(&spec(), &results).expect("labels present");
        assert_eq!(decoded.labels, vec!["adopted".to_owned(), "rejected".to_owned()]);
        assert_eq!(decoded.probabilities, vec![0.2_f32, 0.8_f32]);
    }

    #[test]
    fn errors_when_the_task_is_absent() {
        let error = to_label_probabilities(&spec(), &[]).expect_err("missing task must fail");
        assert!(matches!(error, DecisionError::Backend { .. }));
    }

    #[test]
    fn errors_when_a_required_label_is_absent() {
        let results = vec![("outcome".to_owned(), vec![("adopted".to_owned(), 1.0_f32)])];
        let error =
            to_label_probabilities(&spec(), &results).expect_err("missing label must fail");
        assert!(matches!(error, DecisionError::Backend { .. }));
    }
}
