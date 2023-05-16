use rand::prelude::*;
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum AgentSpecies {
    Red,
    Blue,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Agent {
    id: String,
    species: AgentSpecies,
}

impl Agent {
    fn new(id: String, species: AgentSpecies) -> Agent {
        Agent { id, species }
    }
}

// CELL
#[derive(Clone, Debug, PartialEq)]
pub struct Cell {
    x: u32,
    y: u32,

    agents: HashSet<Agent>,
    graffiti: HashMap<AgentSpecies, f32>,
    pull_strength: HashMap<AgentSpecies, f32>,
}

impl Cell {
    fn new(x: u32, y: u32) -> Self {
        Self {
            x,
            y,
            agents: HashSet::new(),
            graffiti: HashMap::new(),
            pull_strength: HashMap::new(),
        }
    }

    /**
     * Add an agent to the cell
     */
    fn add_agent(&mut self, agent: Agent) {
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
    fn new(size: u32) -> Universe {
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
    fn add_agents(&mut self, amount: u32) {
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

fn main() {
    let start = Instant::now();
    let mut u = Universe::new(100);
    u.add_agents(100000);

    // println!("{:?}", u.cells[0].agents.len());
    for _ in 0..300 {
        u.tick();
    }

    let duration = start.elapsed();
    println!("Time elapsed in expensive_function() is: {:?}", duration);
}
