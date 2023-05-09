pub mod agent;
pub mod cell;
pub mod hyper_params;

use agent::{Agent, AgentSpecies};
use cell::Cell;
use hyper_params::HyperParams;
use rand::prelude::*;
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum ComputationType {
    Serial,
    Parallel,
}

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
        let hyper_params = HyperParams::new(0.5, 0.5, 1.0 / 100.0);
        let cells = (0..size * size)
            .map(|i| Cell::new(i % size, i / size, hyper_params))
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

    fn get_next_cells(&self) -> Vec<Cell> {
        let next_cells: Vec<Cell> = self
            .cells
            .iter()
            .cloned()
            .map(|mut cell| {
                cell.reset();
                cell
            })
            .collect();
        return next_cells;
    }

    pub fn tick(&mut self, computation: ComputationType) {
        if computation == ComputationType::Parallel {
            self.tick_parallel();
        } else {
            self.tick_serial();
        }
    }
}

/**
 * SERIAL implementations of the tick function
 */
impl Universe {
    fn tick_serial(&mut self) {
        // Clone the next itteration of cells and reset the agents
        let mut next_cells: Vec<Cell> = self.get_next_cells();

        // let cells = Arc::new(Mutex::new(self.cells.clone()));

        // Calculate grafitti
        for cell in self.cells.iter_mut() {
            cell.increment_graffiti(self.size);
        }

        // Iterate over the cells and move agents
        for cell in self.cells.iter() {
            for agent in cell.agents.iter() {
                let neighbours = self.neighbours_of(cell.x, cell.y); // [Cell(ps: 5.0), Cell(ps: 10.0), Cell(ps: 2.0), Cell(ps: 3.0)]
                let mut neighbour_cum_pull: Vec<f32> = vec![]; // [5.0, 15.0, 17.0, 20.0]
                let mut total_strength = 0.0;

                for cell in neighbours.clone() {
                    let pull_strength = *cell.pull_strength.get(&agent.species).unwrap_or(&0.0);
                    neighbour_cum_pull.push(pull_strength + total_strength);
                    total_strength += pull_strength;
                }

                let random_neigh = self.prng.clone().gen_range(0.0..total_strength);

                for (index, strength) in neighbour_cum_pull.iter_mut().enumerate() {
                    if *strength > random_neigh {
                        let next_cell_idx =
                            self.get_index(neighbours[index].y, neighbours[index].x);
                        let new_agent = agent.clone();
                        next_cells[next_cell_idx].add_agent(new_agent);
                        break;
                    }
                }
            }
        }

        self.iteration += 1;
        self.cells = next_cells;
    }
}

/**
 * Parallel implementation of the tick function
 */
impl Universe {
    fn tick_parallel(&mut self) {
        // Clone the next itteration of cells and reset the agents
        // let mut next_cells: Vec<Cell> = self.get_next_cells();

        // Calculate grafitti
        self.cells.par_iter_mut().for_each(|cell| {
            cell.increment_graffiti(self.size);
        });
        let mut next_cells: Vec<Cell> = self.get_next_cells();

        // Iterate over the cells and move agents
        let agents = self
            .cells
            .iter()
            .flat_map(|cell| cell.agents.iter())
            .collect::<Vec<&Agent>>();

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
    fn test_parallel_increment_grafitti() {
        let mut u1 = Universe::new(100);
        u1.add_agents(100000);

        let mut u2 = u1.clone();

        let now = Instant::now();

        for _ in 0..300 {
            u1.tick(ComputationType::Parallel);
        }

        println!("Time taken multi thread: {:?}", now.elapsed());

        let now = Instant::now();

        for i in 0..300 {
            if i % 10 == 0 {
                println!("Serial tick: {} in {:?}", i, now.elapsed())
            }
            u2.tick(ComputationType::Serial);
        }
        println!("Time taken serial: {:?}", now.elapsed());
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

        u.tick(ComputationType::Serial);
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

        u1.tick(ComputationType::Serial);
        u2.tick(ComputationType::Parallel);
        assert_ne!(u_before_tick, u2);
        assert_eq!(u1.cells[0].agents, u2.cells[0].agents);
    }

    #[test]
    fn it_works() {
        let start = Instant::now();
        let mut u = Universe::new(100);
        u.add_agents(100000);
        u.tick(ComputationType::Serial);

        println!("Time to add agents: {:?}", start.elapsed());

        assert_eq!(u.cells.len(), 100 * 100);
        assert_eq!(number_of_agents_in_cells(&u.cells), 200000);
    }

    #[test]
    fn same_agents() {}
}
