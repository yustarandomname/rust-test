use std::collections::{HashMap, HashSet};

use crate::agent::{Agent, AgentSpecies};

// CELL
#[derive(Clone, Debug, PartialEq)]
pub struct Cell {
    pub x: u32,
    pub y: u32,

    pub agents: HashSet<Agent>,
    grafitti: HashMap<AgentSpecies, f32>,
    pull_strength: HashMap<AgentSpecies, f32>,
}

impl Cell {
    /**
     * Create a new cell
     *
     * # Examples
     * ```
     * use walker2d::cell::Cell;
     * let cell = Cell::new(0, 0);
     * assert!(cell.x == 0);
     * assert!(cell.x == 0);
     * ```
     */
    pub fn new(x: u32, y: u32) -> Self {
        Self {
            x,
            y,
            agents: HashSet::new(),
            grafitti: HashMap::new(),
            pull_strength: HashMap::new(),
        }
    }

    /**
     * Add an agent to the cell
     *
     * # Examples
     * ```
     * use walker2d::cell::Cell;
     * use walker2d::agent::{Agent, AgentSpecies};
     * let mut cell = Cell::new(0, 0);
     * let agent1 = Agent::new("test1".to_string(), AgentSpecies::Red);
     * let agent2 = Agent::new("test2".to_string(), AgentSpecies::Blue);
     * cell.add_agent(agent1);
     * cell.add_agent(agent2);
     *
     * assert!(cell.agents.len() == 2);
     */
    pub fn add_agent(&mut self, agent: Agent) {
        self.agents.insert(agent);
    }

    /**
     * Count the number of agents in this cell
     */
    pub fn num_agents(&self, species: AgentSpecies) -> usize {
        return self
            .agents
            .iter()
            .filter(|agent| agent.species == species)
            .count();
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn add_agent() {
        let mut cell = Cell::new(0, 0);
        let agent = Agent::new("test".to_string(), AgentSpecies::Red);

        cell.add_agent(agent.clone());

        assert_eq!(cell.agents.len(), 1);
        assert_eq!(cell.agents.contains(&agent), true);
    }
}
