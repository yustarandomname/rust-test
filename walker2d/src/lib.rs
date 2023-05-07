pub mod agent;
pub mod cell;

use agent::{Agent, AgentSpecies};
use cell::Cell;
use rand::prelude::*;
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::collections::HashSet;

// UNIVERSE
#[derive(Debug, Clone, PartialEq)]
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

    /**
     * Get a reference of the top, bottom, left and right cells of a given cell
     */
    fn neighbours_of(&self, row: u32, col: u32) -> Vec<&Cell> {
        let top = self.get_cell(self.size + row - 1, col);
        let bottom = self.get_cell(row + 1, col);
        let left = self.get_cell(row, self.size + col - 1);
        let right = self.get_cell(row, col + 1);

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

                let random_neigh = self.prng.clone().gen_range(0..neighbours.len()); // Get pseudo random neighbour 0..3
                let neighbour_cell = neighbours[random_neigh];

                let next_cell_idx = self.get_index(neighbour_cell.y, neighbour_cell.x);
                let new_agent = agent.clone();
                next_cells[next_cell_idx].add_agent(new_agent);
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

    fn number_of_agents_in_cells(cells: &Vec<Cell>) -> usize {
        cells.iter().fold(0, |acc, cell| acc + cell.agents.len())
    }

    #[test]
    fn number_of_agents_in_universe() {
        let mut u = Universe::new(100);

        const AGENT_SIZE: u32 = 100000;

        u.add_agents(AGENT_SIZE);

        assert_eq!(
            number_of_agents_in_cells(&u.cells),
            (AGENT_SIZE * 2) as usize
        );

        u.tick();
        assert_eq!(
            number_of_agents_in_cells(&u.cells),
            (AGENT_SIZE * 2) as usize
        );
    }

    #[test]
    fn neighbours_of() {
        let u = Universe::new(100);

        let neighbours = u.neighbours_of(1, 1);

        assert_eq!(neighbours.len(), 4);
        assert_eq!(neighbours.contains(&u.get_cell(0, 1)), true); // TOP
        assert_eq!(neighbours.contains(&u.get_cell(2, 1)), true); // BOTTOM
        assert_eq!(neighbours.contains(&u.get_cell(1, 0)), true); // LEFT
        assert_eq!(neighbours.contains(&u.get_cell(1, 2)), true); // RIGHT
    }

    #[test]
    fn neighbours_of_top_right() {
        let u = Universe::new(100);

        let neighbours = u.neighbours_of(0, 99);

        assert_eq!(neighbours.len(), 4);
        assert_eq!(neighbours.contains(&u.get_cell(99, 99)), true); // TOP
        assert_eq!(neighbours.contains(&u.get_cell(1, 99)), true); // BOTTOM
        assert_eq!(neighbours.contains(&u.get_cell(0, 0)), true); // LEFT
        assert_eq!(neighbours.contains(&u.get_cell(0, 98)), true); // RIGHT
    }

    #[test]
    fn neighbours_of_bottom_right() {
        let u = Universe::new(100);

        let neighbours = u.neighbours_of(99, 0);

        assert_eq!(neighbours.len(), 4);
        assert_eq!(neighbours.contains(&u.get_cell(98, 0)), true); // TOP
        assert_eq!(neighbours.contains(&u.get_cell(0, 0)), true); // BOTTOM
        assert_eq!(neighbours.contains(&u.get_cell(99, 99)), true); // LEFT
        assert_eq!(neighbours.contains(&u.get_cell(99, 1)), true); // RIGHT
    }

    #[test]
    fn prng_works() {
        let mut u1 = Universe::new(10);
        let mut u2 = Universe::new(10);

        assert_eq!(u1.prng.next_u32(), u2.prng.next_u32());

        u1.add_agents(100);
        u2.add_agents(100);

        assert_eq!(u1.cells[0].agents, u2.cells[0].agents);

        let u_before_tick = u1.clone();

        u1.tick();
        u2.tick();
        assert_ne!(u_before_tick, u2);
        assert_eq!(u1.cells[0].agents, u2.cells[0].agents);
    }

    #[test]
    fn it_works() {
        let start = Instant::now();
        let mut u = Universe::new(100);
        u.add_agents(100000);
        u.tick();

        println!("Time to add agents: {:?}", start.elapsed());

        assert_eq!(u.cells.len(), 100 * 100);
        assert_eq!(number_of_agents_in_cells(&u.cells), 200000);
    }

    #[test]
    fn same_agents() {}
}
