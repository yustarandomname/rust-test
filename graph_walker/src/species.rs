use std::ops::{AddAssign, MulAssign};

pub type SpeciesGraffiti = Species<f32>;
pub type SpeciesPushStrength = Species<f32>;

#[derive(Debug, Clone, Copy)]
pub struct Species<T: AddAssign + MulAssign> {
    pub red: T,
    pub blue: T,
}

impl<T: AddAssign + MulAssign + Copy> Species<T> {
    pub fn new(red: T, blue: T) -> Species<T> {
        Species { red, blue }
    }

    pub fn set_red(&mut self, amount: T) {
        self.red = amount;
    }
    pub fn set_blue(&mut self, amount: T) {
        self.blue = amount;
    }

    pub fn add_blue(&mut self, amount: T) {
        self.blue += amount;
    }
    pub fn add_red(&mut self, amount: T) {
        self.red += amount;
    }

    pub fn mult_all(&mut self, amount: T) {
        self.red *= amount;
        self.blue *= amount;
    }
}
