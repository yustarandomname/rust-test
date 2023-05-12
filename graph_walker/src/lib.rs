mod hyper_params;
mod testing;

use hyper_params::HyperParams;
use oorandom::Rand32;
use pad::PadStr;
use rayon::prelude::*;
use std::{
    collections::HashMap,
    f32::consts::E,
    fmt,
    sync::{Arc, Mutex},
};

const HYPER_PARAMS: HyperParams = HyperParams {
    gamma: 0.5,
    lambda: 0.5,
    beta: 1.0 / 100.0,
};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum AgentSpecies {
    Red,
    Blue,
}

const AGENT_SPECIES: [AgentSpecies; 2] = [AgentSpecies::Red, AgentSpecies::Blue];

#[derive(Debug, Clone)]
struct Node {
    pub index: u32,
    pub neighbours: Vec<u32>, // indices of neighbours
    pub grafitti: [f32; 2],
    pub push_strength: [f32; 2],
    pub blue_agents: u32,
    pub red_agents: u32,
}

impl Node {
    pub fn new(index: u32, edges: &HashMap<u32, Vec<u32>>) -> Node {
        Node {
            index,
            neighbours: edges.get(&index).unwrap().to_vec(),
            grafitti: [0.0; 2],
            push_strength: [0.0; 2],
            blue_agents: 0,
            red_agents: 0,
        }
    }

    pub fn get_push_strength(&self, species: &AgentSpecies) -> f32 {
        match species {
            AgentSpecies::Red => self.push_strength[0],
            AgentSpecies::Blue => self.push_strength[1],
        }
    }

    pub fn add_agents(&mut self, amount: u32, species: AgentSpecies) {
        match species {
            AgentSpecies::Red => self.red_agents += amount,
            AgentSpecies::Blue => self.blue_agents += amount,
        }
    }

    pub fn agents_with_species(&self, species: &AgentSpecies) -> u32 {
        match species {
            AgentSpecies::Blue => self.red_agents,
            AgentSpecies::Red => self.blue_agents,
        }
    }

    pub fn update_grafitti_and_push_strength(&mut self, grid_size: u32) {
        // 0 - Decrement current grafitti by lambda
        self.grafitti = self.grafitti.map(|entry| entry * HYPER_PARAMS.lambda);

        // 1 - Increase grafiti by gamma * sum of same agent' count
        self.grafitti[0] += HYPER_PARAMS.gamma * self.red_agents as f32;
        self.grafitti[1] += HYPER_PARAMS.gamma * self.blue_agents as f32;

        // 2 - Calculate push strength
        let l = 1.0 / grid_size as f32;

        self.push_strength = self.grafitti.map(|entry| {
            let xi = entry / (l * 2.0);
            E.powf(-HYPER_PARAMS.beta * xi)
        });
    }
}

pub struct Universe2D {
    size: u32,
    nodes: Vec<Node>,
    prng: Rand32,
    iteration: u32,
}

impl Universe2D {
    pub fn new(size: u32, agent_size: u32) -> Universe2D {
        let mut prng = Rand32::new(100);

        let mut edges: HashMap<u32, Vec<u32>> = HashMap::new();

        for y in 0..size {
            for x in 0..size {
                let index = y * size + x;

                let left_index = y * size + (x + size - 1) % size;
                let right_index = y * size + (x + 1) % size;
                let top_index = (y + size - 1) % size * size + x;
                let bottom_index = (y + 1) % size * size + x;

                let mut new_edges = vec![left_index, right_index, top_index, bottom_index];
                new_edges.sort();

                match edges.get_mut(&index) {
                    Some(_) => {}
                    None => {
                        edges.insert(index, new_edges);
                    }
                }
            }
        }

        let mut nodes: Vec<Node> = (0..(size * size))
            .map(|index| Node::new(index, &edges))
            .collect();

        (0..agent_size * 2).for_each(|id| {
            let node_index = prng.rand_range(0..(size * size));
            let species = if id % 2 == 0 {
                AgentSpecies::Red
            } else {
                AgentSpecies::Blue
            };

            nodes[node_index as usize].add_agents(1, species);
        });

        Universe2D {
            size,
            nodes,
            prng,
            iteration: 0,
        }
    }
}

impl Universe2D {
    pub fn tick(&mut self) {
        // 0) update grafitti in nodes
        // 10 iter => 12ms
        self.nodes.par_iter_mut().for_each(|node| {
            node.update_grafitti_and_push_strength(self.size);
        });

        // 1) move agents out of nodes
        // 10 iter => 12ms
        let new_nodes: Vec<Node> = self
            .nodes
            .par_iter()
            .map(|node| {
                let mut node = node.clone();
                node.blue_agents = 0;
                node.red_agents = 0;
                node
            })
            .collect();

        let arc_nodes = Arc::new(Mutex::new(new_nodes));

        self.nodes.par_iter().for_each(|node| {
            let mut prng = Rand32::new((self.iteration as u64 + 1) * node.index as u64);

            for species in &AGENT_SPECIES {
                let neighbours = &node.neighbours;
                let _neighbour_strengths = neighbours
                    .iter()
                    .map(|neighbour_idx| {
                        self.nodes[*neighbour_idx as usize].get_push_strength(species)
                    })
                    .collect::<Vec<f32>>();

                // new agents for each neighbour
                // [key: neighbour index, value: new agents]
                let mut new_agents: Vec<(u32, u32)> = neighbours
                    .iter()
                    .map(|neighbour_idx| (*neighbour_idx, 0))
                    .collect();

                for _ in 0..node.agents_with_species(species) {
                    let new_node_index = prng.rand_range(0..neighbours.len() as u32);
                    new_agents[new_node_index as usize].1 += 1;
                }

                let mut nodes_guard = arc_nodes.lock().unwrap();
                new_agents.iter().for_each(|(node_index, amount)| {
                    nodes_guard[*node_index as usize].add_agents(*amount, *species)
                });
            }
        });

        self.nodes = arc_nodes.lock().unwrap().clone();

        self.iteration += 1;
    }
}

impl fmt::Debug for Universe2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} UNIVERSE 2D {}\n", "=".repeat(10), "=".repeat(10))?;

        write!(f, "size: {}\n", self.size)?;
        write!(f, "node size: {}\n", self.nodes.len())?;
        write!(f, "iterations: {}\n", self.iteration)?;

        write!(f, "{}\n", "=".repeat(30))?;
        for y in 0..self.size {
            for x in 0..self.size {
                let index = y * self.size + x;
                let node = &self.nodes[index as usize];

                let blue_agents =
                    self.nodes[index as usize].agents_with_species(&AgentSpecies::Blue);
                let red_agents = self.nodes[index as usize].agents_with_species(&AgentSpecies::Red);

                let blue_graffiti = node.blue_agents;
                let red_graffiti = node.red_agents;

                write!(
                    f,
                    "|{} a({},{}) g:({},{})",
                    index.to_string().with_exact_width(2),
                    blue_agents.to_string().with_exact_width(2),
                    red_agents.to_string().with_exact_width(2),
                    blue_graffiti.to_string().with_exact_width(4),
                    red_graffiti.to_string().with_exact_width(4)
                )?;
            }
            write!(f, "|\n")?;
        }
        write!(f, "")
    }
}

impl fmt::Display for Universe2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} UNIVERSE 2D {}\n", "=".repeat(10), "=".repeat(10))?;

        write!(f, "size: {}\n", self.size)?;
        write!(f, "node size: {}\n", self.nodes.len())?;
        write!(f, "iterations: {}\n", self.iteration)?;

        write!(f, "{}\n", "=".repeat(30))?;
        for y in 0..self.size {
            for x in 0..self.size {
                let index = y * self.size + x;
                let node = &self.nodes[index as usize];

                let blue_graffiti = node.grafitti[0];
                let red_graffiti = node.grafitti[1];

                let emoji = if blue_graffiti > red_graffiti {
                    "🔵"
                } else if red_graffiti > blue_graffiti {
                    "🔴"
                } else {
                    "⚪"
                };

                write!(f, "{emoji}")?;
            }
            write!(f, "|\n")?;
        }
        write!(f, "")
    }
}

mod test {
    use std::time::Instant;

    use super::*;

    fn total_agent_size(universe: &Universe2D) -> u32 {
        universe
            .nodes
            .iter()
            .map(|node| node.blue_agents + node.red_agents)
            .sum()
    }

    fn total_agent_size_of_species(universe: &Universe2D, species: AgentSpecies) -> u32 {
        universe
            .nodes
            .iter()
            .map(|node| node.agents_with_species(&species))
            .sum()
    }

    #[test]
    fn test_universe2d() {
        let universe = Universe2D::new(4, 100);

        for node in &universe.nodes {
            // FOR DEBUG println!("{:?}", node);

            assert_eq!(node.neighbours.len(), 4);
        }

        assert_eq!(total_agent_size(&universe), 200);
        assert_eq!(
            total_agent_size_of_species(&universe, AgentSpecies::Blue),
            100
        );
        assert_eq!(
            total_agent_size_of_species(&universe, AgentSpecies::Red),
            100
        );

        println!("{}", universe);
    }

    #[test]
    fn test_tick() {
        let mut universe = Universe2D::new(4, 100);

        for _ in 0..10 {
            universe.tick();
        }
        println!("{}", universe);

        // all notes should have a grafitti of >0.0
        todo!();
    }

    #[test]
    fn performance_test_tick() {
        let mut universe = Universe2D::new(100, 100000);

        let start = Instant::now();

        for _ in 0..30 {
            universe.tick();
        }
        // 1.3s
        println!("{:?}", start.elapsed());
    }
}
