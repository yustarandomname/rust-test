use std::{
    collections::{HashMap, HashSet},
    f32::consts::E,
};

use crate::{
    agent::{Agent, AgentSpecies},
    hyper_params::HyperParams,
};

// CELL
#[derive(Clone, Debug, PartialEq)]
pub struct Cell {
    pub x: u32,
    pub y: u32,

    pub agents: HashSet<Agent>,
    pub pull_strength: HashMap<AgentSpecies, f32>,
    graffiti: HashMap<AgentSpecies, f32>,
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
    pub fn new(x: u32, y: u32, hyper_params: HyperParams) -> Self {
        Self {
            x,
            y,
            agents: HashSet::new(),
            graffiti: HashMap::new(),
            pull_strength: HashMap::new(),
            hyper_params,
        }
    }

    /**
     * Reset the cell to its initial (agent, graffiti & pull_strength) state
     *
     * ## examples
     * ```
     * use walker2d::cell::Cell;
     * use walker2d::agent::{Agent, AgentSpecies};
     * use walker2d::hyper_params::HyperParams;
     * let mut cell = Cell::new(0, 0, HyperParams::default());
     * cell.agents.insert(Agent::new("1".to_string(), AgentSpecies::Blue));
     * cell.reset();
     * assert!(cell.agents.len() == 0);
     * ```
     */
    pub fn reset(&mut self) {
        self.agents = HashSet::new();
        self.pull_strength = HashMap::new();
        self.graffiti = HashMap::new();
    }

    pub fn increment_graffiti(&mut self, grid_size: u32) {
        for species in AgentSpecies::iter() {
            let entry: &mut f32 = self.graffiti.entry(species.clone()).or_insert(0.0);

            // 0 - Decrease graffiti
            *entry *= self.hyper_params.lambda;

            // 1 - Increase graffiti
            let num_agents_of_species = self
                .agents
                .iter()
                .filter(|agent| agent.species == species)
                .count() as f32;
            *entry += num_agents_of_species * self.hyper_params.gamma;

            // 2 - Calulate pull strength
            let l = 1.0 / grid_size as f32;
            let xi = *entry / (l * 2.0);
            let pull_strength = E.powf(-self.hyper_params.beta * xi);
            self.pull_strength.insert(species, pull_strength);
        }
    }

    /**
     * Add an agent to the cell
     *
     * # Examples
     * ```
     * use walker2d::cell::Cell;
     * use walker2d::agent::{Agent, AgentSpecies};
     * use walker2d::hyper_params::HyperParams;
     * let mut cell = Cell::new(0, 0, HyperParams::default());
     * let agent1 = Agent::new("test1".to_string(), AgentSpecies::Red);
     * let agent2 = Agent::new("test2".to_string(), AgentSpecies::Blue);
     * cell.add_agent(agent1);
     * cell.add_agent(agent2);
     *
     * assert!(cell.agents.len() == 2);
     */
    pub fn add_agent(&mut self, mut agent: Agent) {
        *agent.parent_cell = &self;
        self.agents.insert(agent);
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn add_agent() {
        let hp = HyperParams::new(0.0, 0.0, 0.0);
        let mut cell = Cell::new(0, 0, hp);
        let agent = Agent::new("test".to_string(), AgentSpecies::Red);

        cell.add_agent(agent.clone());

        assert_eq!(cell.agents.len(), 1);
        assert_eq!(cell.agents.contains(&agent), true);
    }

    #[test]
    fn test_stng_cum() {
        let hp = HyperParams::new(0.0, 0.0, 0.0);
        let mut cell1 = Cell::new(0, 0, hp);
        cell1.pull_strength.insert(AgentSpecies::Red, 5.0);

        let mut cell2 = cell1.clone();
        cell2.pull_strength.insert(AgentSpecies::Red, 10.0);

        let mut cell3 = cell1.clone();
        cell3.pull_strength.insert(AgentSpecies::Red, 2.0);

        let mut cell4 = cell1.clone();
        cell4.pull_strength.insert(AgentSpecies::Red, 3.5);

        let neighbours = vec![cell1, cell2, cell3, cell4];

        let agent = Agent::new("test".to_string(), AgentSpecies::Red);

        let mut neighbour_cum_pull: Vec<f32> = vec![]; // [5.0, 15.0, 17.0, 20.0]
        let mut pull_total = 0.0;

        for cell in neighbours {
            let pull_strength = *cell.pull_strength.get(&agent.species).unwrap_or(&0.0);
            neighbour_cum_pull.push(pull_strength + pull_total);

            pull_total += pull_strength
        }

        assert_eq!(neighbour_cum_pull, vec![5.0, 15.0, 17.0, 20.5])
    }
}
