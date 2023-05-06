pub mod agent;
pub mod cell;

use agent::{Agent, AgentSpecies};
use cell::Cell;
use rand::prelude::*;
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::collections::HashSet;

// UNIVERSE
#[derive(Debug)]
pub struct Universe {
    size: u32,
    cells: Vec<Cell>,
    prng: ChaCha8Rng,
    iteration: u32,
}

impl Universe {
    /**
     * Create a new universe with a given size
     */
    pub fn new(size: u32) -> Universe {
        let prng = ChaCha8Rng::seed_from_u64(2);
        let cells = (0..size * size)
            .map(|i| Cell::new(i % size, i / size))
            .collect();

        Universe {
            size,
            cells,
            prng,
            iteration: 0,
        }
    }

    fn get_cell(&self, row: u32, col: u32) -> &Cell {
        return &self.cells[self.get_index(row, col)];
    }

    fn neighbours_of(&self, col: u32, row: u32) -> Vec<&Cell> {
        let top = self.get_cell(col, self.size + row - 1);
        let bottom = self.get_cell(col, row + 1);
        let left = self.get_cell(self.size + col - 1, row);
        let right = self.get_cell(col + 1, row);

        vec![top, bottom, left, right]
    }

    /**
     * Add an agent to a random cell in the universe
     */
    fn add_agent_to_random_cell(&mut self, species: AgentSpecies) {
        let agent = Agent::new(self.prng.next_u32().to_string(), species);

        let idx = self.prng.gen_range(0..(self.size * self.size)) as usize;
        self.cells[idx].add_agent(agent);
    }

    /**
     * Add a number of agents to the universe, split evenly between the two species
     *
     * @param amount The number of agents to add
     */
    pub fn add_agents(&mut self, amount: u32) {
        for _ in 0..amount {
            self.add_agent_to_random_cell(AgentSpecies::Red);
            self.add_agent_to_random_cell(AgentSpecies::Blue);
        }
    }

    fn get_index(&self, row: u32, column: u32) -> usize {
        let rw = row % self.size;
        let col = column % self.size;

        (rw * self.size + col) as usize
    }

    pub fn tick(&mut self) {
        // Clone the next itteration of cells and reset the agents
        let mut next_cells: Vec<Cell> = self
            .cells
            .iter()
            .cloned()
            .map(|mut cell| {
                cell.agents = HashSet::new();
                cell
            })
            .collect();

        // Iterate over the cells and move agents
        for cell in self.cells.iter() {
            for agent in cell.agents.iter() {
                let neighbours = self.neighbours_of(cell.x, cell.y);

                let random_neigh = self.prng.clone().gen_range(0..neighbours.len());
                let neighbour_cell = neighbours[random_neigh];

                let next_cell_idx = self.get_index(neighbour_cell.y, neighbour_cell.x);

                next_cells[next_cell_idx].add_agent(agent.clone());
            }
        }

        self.iteration += 1;
        self.cells = next_cells;
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;

    #[test]
    fn it_works() {
        let start = Instant::now();
        let mut u = Universe::new(100);
        u.add_agents(100000);
        u.tick();

        println!("Time to add agents: {:?}", start.elapsed());

        assert_eq!(u.cells.len(), 100 * 100);
    }
}
