mod hyper_params;

use hyper_params::HyperParams;
use pad::PadStr;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::{collections::HashMap, f32::consts::E, fmt};

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
    pub neighbours: Vec<u32>, // indices of neighbours
    pub grafitti: HashMap<AgentSpecies, f32>,
    pub pull_strength: HashMap<AgentSpecies, f32>,
    pub blue_agents: u32,
    pub red_agents: u32,
}

impl Node {
    pub fn new(index: u32, edges: &HashMap<u32, Vec<u32>>) -> Node {
        Node {
            neighbours: edges.get(&index).unwrap().to_vec(),
            grafitti: HashMap::new(),
            pull_strength: HashMap::new(),
            blue_agents: 0,
            red_agents: 0,
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
            AgentSpecies::Red => self.red_agents,
            AgentSpecies::Blue => self.blue_agents,
        }
    }

    pub fn update_grafitti_and_pull(&mut self, grid_size: u32) {
        for species in &AGENT_SPECIES {
            let num_agents_of_species = self.agents_with_species(species);

            let entry: &mut f32 = self.grafitti.entry(*species).or_insert(0.0);

            // 0 - Decrement current grafitti by lambda
            *entry *= HYPER_PARAMS.lambda;

            // 1 - Increase grafiti by gamma * sum of same agent' count
            *entry += HYPER_PARAMS.gamma * num_agents_of_species as f32;

            // 2 - Calculate pull strength
            let l = 1.0 / grid_size as f32;
            let xi = *entry / (l * 2.0);
            let pull_strength = E.powf(-HYPER_PARAMS.beta * xi);
            self.pull_strength.insert(*species, pull_strength);
        }
    }
}

pub struct Universe2D {
    size: u32,
    nodes: Vec<Node>,
    prng: ChaCha8Rng,
    iteration: u32,
}

impl Universe2D {
    pub fn new(size: u32, agent_size: u32) -> Universe2D {
        let mut prng = ChaCha8Rng::seed_from_u64(2);

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
            let node_index = prng.gen_range(0..(size * size));
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
        for node in &mut self.nodes {
            node.update_grafitti_and_pull(self.size);
        }

        // 1) move agents out of nodes
        let mut new_nodes = self.nodes.clone();
        // reset agents in nodes
        for node in &mut new_nodes {
            node.red_agents = 0;
            node.blue_agents = 0;
        }

        for node in &self.nodes {
            for species in &AGENT_SPECIES {
                let neighbours = &node.neighbours;
                let _neighbour_strengths = neighbours
                    .iter()
                    .map(|neighbour_idx| {
                        self.nodes[*neighbour_idx as usize]
                            .pull_strength
                            .get(&AgentSpecies::Red)
                            .unwrap_or(&0.0)
                    })
                    .collect::<Vec<&f32>>();

                let mut new_agents: HashMap<u32, u32> = HashMap::new(); // HashMap with key: node_index, value: num_agents
                neighbours.iter().for_each(|idx| {
                    new_agents.insert(*idx, 0);
                });

                for _ in 0..node.agents_with_species(species) {
                    let new_node_index: u32 = self.prng.gen_range(0..neighbours.len()) as u32;

                    let neighbour_index = neighbours[new_node_index as usize];
                    *new_agents.get_mut(&neighbour_index).unwrap() += 1;
                }

                new_agents.iter().for_each(|(node_index, amount)| {
                    new_nodes[*node_index as usize].add_agents(*amount, *species)
                });
            }
        }

        self.nodes = new_nodes;

        self.iteration += 1;
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

                let blue_agents =
                    self.nodes[index as usize].agents_with_species(&AgentSpecies::Blue);
                let red_agents = self.nodes[index as usize].agents_with_species(&AgentSpecies::Red);

                let blue_graffiti = node.grafitti.get(&AgentSpecies::Blue).unwrap_or(&0.0);
                let red_graffiti = node.grafitti.get(&AgentSpecies::Red).unwrap_or(&0.0);

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
        for node in &universe.nodes {
            for species in &AGENT_SPECIES {
                assert_eq!(node.grafitti.contains_key(species), true); // TODO
                assert_eq!(node.grafitti.get(species).unwrap() > &0.0, true); // TODO
            }
        }
    }

    #[test]
    fn performance_test_tick() {
        let mut universe = Universe2D::new(100, 100000);
        let start = Instant::now();

        for _ in 0..10 {
            universe.tick();
        }

        println!("{:?}", start.elapsed());

        // all notes should have a grafitti of >0.0
        for node in &universe.nodes {
            for species in &AGENT_SPECIES {
                assert_eq!(node.grafitti.contains_key(species), true); // TODO
                assert_eq!(node.grafitti.get(species).unwrap() > &0.0, true); // TODO
            }
        }
    }
}
