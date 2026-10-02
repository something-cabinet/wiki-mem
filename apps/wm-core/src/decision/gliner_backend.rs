use std::path::Path;

use wm_engine::{
    ClassificationMode, ClassificationSpec, DecisionBackend, DecisionError, LabelProbabilities,
};

pub const GLINER_OFFLINE_ENV: &str = "GLINER_OFFLINE";

/// Local gliner-rs backend. The checkpoint directory is resolved explicitly by
/// the caller; `GLINER_OFFLINE` is set so the library never auto-downloads.
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
        Ok(specs
            .iter()
            .map(|spec| to_label_probabilities(spec, &results))
            .collect())
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
) -> LabelProbabilities {
    let pairs = results
        .iter()
        .find(|(task, _)| task == &spec.task)
        .map(|(_, pairs)| pairs);
    LabelProbabilities {
        task: spec.task.clone(),
        labels: spec.labels.clone(),
        probabilities: spec
            .labels
            .iter()
            .map(|label| {
                pairs
                    .and_then(|pairs| pairs.iter().find(|(name, _)| name == label))
                    .map(|(_, probability)| *probability)
                    .unwrap_or_default()
            })
            .collect(),
    }
}
