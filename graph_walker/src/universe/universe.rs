use std::fmt::{Debug, Display};

use crate::hyper_params::HyperParams;

pub trait Universe: Debug + Display {
    fn new(size: u32, agent_size: u32) -> Self;
    fn set_hyper_params(&mut self, hyper_params: HyperParams);
    fn tick(&mut self);
    fn iterate(&mut self, iterations: u32);
}
