use std::collections::HashMap;

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

#[derive(Debug, PartialEq, Eq)]
enum AgentSpecies {
    Red,
    Blue,
}

#[derive(Debug)]
struct Agent {
    id: String,
    node_index: u32,
    species: AgentSpecies,
}

#[derive(Debug)]
struct Node {
    index: u32,
    pub neighbours: Vec<u32>, // indices of neighbours
}

impl Node {
    pub fn new(index: u32, edges: &HashMap<u32, Vec<u32>>) -> Node {
        Node {
            index,
            neighbours: edges.get(&index).unwrap().to_vec(),
        }
    }

    fn add_edge(&mut self, to_node: u32) {
        self.neighbours.push(to_node)
    }
}

pub struct Universe2D {
    size: u32,
    nodes: Vec<Node>,
    agents: HashMap<u32, Vec<Agent>>,
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

        let nodes: Vec<Node> = (0..(size * size))
            .map(|index| Node::new(index, &edges))
            .collect();

        let mut agents: HashMap<u32, Vec<Agent>> = HashMap::new();

        (0..agent_size * 2).for_each(|index| {
            let node_index = prng.gen_range(0..(size * size));
            let agent = Agent {
                id: index.to_string(),
                node_index,
                species: if index % 2 == 0 {
                    AgentSpecies::Red
                } else {
                    AgentSpecies::Blue
                },
            };
            match agents.get_mut(&node_index) {
                Some(agents) => {
                    agents.push(agent);
                }
                None => {
                    agents.insert(node_index, vec![agent]);
                }
            }
        });

        Universe2D {
            size,
            nodes,
            prng,
            agents,
            iteration: 0,
        }
    }
}

mod test {
    use super::*;

    fn total_agent_size(universe: &Universe2D) -> u32 {
        universe
            .agents
            .iter()
            .fold(0, |acc, (_, agents)| acc + agents.len() as u32)
    }

    fn total_agent_size_of_species(universe: &Universe2D, species: AgentSpecies) -> u32 {
        universe.agents.iter().fold(0, |acc, (_, agents)| {
            acc + agents
                .iter()
                .filter(|agent| agent.species == species)
                .count() as u32
        })
    }

    #[test]
    fn test_universe2d() {
        let universe = Universe2D::new(4, 100);

        for node in &universe.nodes {
            println!("{:?}", node);

            assert_eq!(node.neighbours.len(), 4);
            assert_eq!(universe.agents.contains_key(&node.index), true);
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
    }
}
