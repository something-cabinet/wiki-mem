#[derive(Clone, Debug, PartialEq)]
pub struct EmbedVector(pub Vec<f32>);

impl EmbedVector {
    pub fn dim(&self) -> usize {
        self.0.len()
    }

    pub fn normalize(&mut self) {
        let norm_sq: f32 = self.0.iter().map(|x| x * x).sum();
        if norm_sq > 1e-12 {
            let norm = norm_sq.sqrt();
            for x in &mut self.0 {
                *x /= norm;
            }
        }
    }

    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }
}
