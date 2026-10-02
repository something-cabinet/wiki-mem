#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ClassificationMode {
    Softmax,
    Sigmoid { threshold: f32 },
}
