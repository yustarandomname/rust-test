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
     */
    pub fn add_agent(&mut self, agent: Agent) {
        self.agents.insert(agent);
    }

    /**
     * Remove an agent from the cell
     */
    fn remove_agent(&mut self, agent: Agent) -> Option<Agent> {
        return self.agents.take(&agent);
    }

    /**
     * Move an agent from this cell to another
     * If the target cell is not empty, the agent will be moved to a target neighbour
     */
    fn move_agent(&mut self, agent: Agent, target: &mut Cell) {
        let removed_agent = self.remove_agent(agent);

        match removed_agent {
            Some(agent) => target.add_agent(agent),
            None => (),
        }
    }

    fn num_agents(&self, species: AgentSpecies) -> usize {
        return self
            .agents
            .iter()
            .filter(|agent| agent.species == species)
            .count();
    }
}
