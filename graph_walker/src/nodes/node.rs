use std::collections::HashMap;

use oorandom::Rand32;

use crate::{agent_species::AgentSpecies, hyper_params::HyperParams};

pub trait Node<T>: Sized {
    fn new(index: u32, edges: &HashMap<u32, T>) -> Self;
    fn get_prng(&self) -> Rand32;
    fn get_push_strength(&self, species: &AgentSpecies) -> f32;
    fn add_agents(&mut self, amount: u32, species: AgentSpecies);
    fn get_agents_with_species(&self, species: &AgentSpecies) -> u32;
    fn update_graffiti_and_push_strength(&mut self, hyper_params: &HyperParams, _grid_size: u32);
    fn move_agents_out(&mut self, nodes: &Vec<Self>, _grid_size: u32);
    fn move_agents_in(&mut self, nodes: &Vec<Self>);
}
