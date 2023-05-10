#[derive(Clone, Debug, PartialEq, Copy)]
pub struct HyperParams {
    pub gamma: f32,
    pub lambda: f32,
    pub beta: f32,
}

impl HyperParams {
    pub fn new(gamma: f32, lambda: f32, beta: f32) -> Self {
        Self {
            gamma,
            lambda,
            beta,
        }
    }
}

impl Default for HyperParams {
    fn default() -> Self {
        Self {
            gamma: 0.5,
            lambda: 0.5,
            beta: 1.0 / 100.0,
        }
    }
}
