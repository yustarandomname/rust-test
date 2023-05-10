use std::collections::{HashMap, HashSet};

use petgraph::graphmap::NodeTrait;

use crate::{
    agent::{Agent, AgentSpecies},
    hyper_params::HyperParams,
};

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Cell {
    pub agents: HashSet<Agent>,
    pub pull_strength: HashMap<AgentSpecies, f32>,
    grafitti: HashMap<AgentSpecies, f32>,
    hyper_params: HyperParams,
}

impl Cell {
    /**
     * Create a new cell
     *
     * # Examples
     * ```
     * use walker2d::cell::Cell;
     * use walker2d::hyper_params::HyperParams;
     * let cell = Cell::new(0, 0, HyperParams::default());
     * assert!(cell.x == 0);
     * assert!(cell.x == 0);
     * ```
     */
    pub fn new(hyper_params: HyperParams) -> Self {
        Self {
            agents: HashSet::new(),
            grafitti: HashMap::new(),
            pull_strength: HashMap::new(),
            hyper_params,
        }
    }
}
