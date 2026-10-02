#[derive(Clone, Debug, PartialEq)]
pub struct LabelProbabilities {
    pub task: String,
    pub labels: Vec<String>,
    pub probabilities: Vec<f32>,
}
