use super::classification_spec_model::ClassificationSpec;
use super::decision_error_model::DecisionError;
use super::label_probabilities_model::LabelProbabilities;

/// Swappable inference backend. Production uses a local gliner-rs model;
/// tests use a deterministic mock. One call classifies one text against every
/// task at once.
pub trait DecisionBackend {
    fn classify(
        &self,
        text: &str,
        specs: &[ClassificationSpec],
    ) -> Result<Vec<LabelProbabilities>, DecisionError>;
}
