use super::classification_spec_model::ClassificationSpec;
use super::decision_error_model::DecisionError;
use super::label_probabilities_model::LabelProbabilities;

pub trait DecisionBackend {
    fn classify(
        &self,
        text: &str,
        specs: &[ClassificationSpec],
    ) -> Result<Vec<LabelProbabilities>, DecisionError>;
}
