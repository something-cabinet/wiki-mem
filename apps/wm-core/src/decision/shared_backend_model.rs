use std::sync::Arc;

use wm_engine::{ClassificationSpec, DecisionBackend, DecisionError, LabelProbabilities};

use super::gliner_backend::GlinerBackend;

pub struct SharedBackend {
    inner: Arc<GlinerBackend>,
}

impl SharedBackend {
    pub fn new(inner: Arc<GlinerBackend>) -> Self {
        Self { inner }
    }
}

impl DecisionBackend for SharedBackend {
    fn classify(
        &self,
        text: &str,
        specs: &[ClassificationSpec],
    ) -> Result<Vec<LabelProbabilities>, DecisionError> {
        self.inner.classify(text, specs)
    }
}
