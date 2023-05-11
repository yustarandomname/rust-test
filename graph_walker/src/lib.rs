mod hyper_params;

use hyper_params::HyperParams;
use pad::PadStr;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::{
    collections::{HashMap, HashSet},
    f32::consts::E,
    fmt,
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
struct Agent {
    id: u32,
    node_index: u32,
    species: AgentSpecies,
}

#[derive(Debug, Clone)]
struct Node {
    index: u32,
    pub neighbours: Vec<u32>, // indices of neighbours
    pub grafitti: HashMap<AgentSpecies, f32>,
    pub pull_strength: HashMap<AgentSpecies, f32>,
}

impl Node {
    pub fn new(index: u32, edges: &HashMap<u32, Vec<u32>>) -> Node {
        Node {
            index,
            neighbours: edges.get(&index).unwrap().to_vec(),
            grafitti: HashMap::new(),
            pull_strength: HashMap::new(),
        }
    }

    pub fn agents_with_species(
        &self,
        agents: &HashMap<u32, Vec<Agent>>,
        species: AgentSpecies,
    ) -> u32 {
        agents
            .get(&self.index)
            .unwrap()
            .iter()
            .filter(|agent| agent.species == species)
            .count() as u32
    }

    pub fn update_grafitti_and_pull(&mut self, agents: &Vec<HashSet<Agent>>, grid_size: u32) {
        for species in &AGENT_SPECIES {
            let entry: &mut f32 = self.grafitti.entry(*species).or_insert(0.0);

            // 0 - Decrement current grafitti by lambda
            *entry *= HYPER_PARAMS.lambda;

            // TODO: optimise this operation to O(1) instead of O(n)
            let num_agents_of_species = agents
                .get(self.index as usize)
                .unwrap()
                .iter()
                .filter(|agent| agent.species == *species)
                .count() as f32;

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
    agents: Vec<HashSet<Agent>>,
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

        let mut agents: Vec<HashSet<Agent>> = vec![HashSet::new(); (size * size) as usize];

        (0..agent_size * 2).for_each(|id| {
            let node_index = prng.gen_range(0..(size * size));
            let agent = Agent {
                id,
                node_index,
                species: if id % 2 == 0 {
                    AgentSpecies::Red
                } else {
                    AgentSpecies::Blue
                },
            };

            agents[node_index as usize].insert(agent);
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

impl Universe2D {
    pub fn tick(&mut self) {
        // 0) update grafitti in nodes
        for node in &mut self.nodes {
            node.update_grafitti_and_pull(&self.agents, self.size);
        }

        // 2) move agents
        let mut new_agents: Vec<HashSet<Agent>> =
            vec![HashSet::new(); (self.size * self.size) as usize];

        for (node_idx, hash_agents) in self.agents.iter_mut().enumerate() {
            for species in &AGENT_SPECIES {
                let neighbours = &self.nodes[node_idx].neighbours;

                let neighbour_strengths = neighbours
                    .iter()
                    .map(|neighbour_idx| {
                        self.nodes[*neighbour_idx as usize]
                            .pull_strength
                            .get(species)
                            .unwrap_or(&0.0)
                    })
                    .collect::<Vec<&f32>>();

                for agent in hash_agents.iter() {
                    let new_node_index = self.prng.gen_range(0..neighbours.len()); // TODO: make this biased

                    new_agents[new_node_index].insert(agent.clone());
                }
            }
        }

        self.iteration += 1;
    }
}

impl fmt::Display for Universe2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} UNIVERSE 2D {}\n", "=".repeat(10), "=".repeat(10))?;

        write!(f, "size: {}\n", self.size)?;
        write!(f, "node size: {}\n", self.nodes.len())?;
        write!(f, "agent size: {}\n", self.agents.len())?;
        write!(f, "iterations: {}\n", self.iteration)?;

        write!(f, "{}\n", "=".repeat(30))?;
        for y in 0..self.size {
            for x in 0..self.size {
                let index = y * self.size + x;
                let node = &self.nodes[index as usize];

                let blue_agents = self.agents[index as usize]
                    .iter()
                    .filter(|agent| agent.species == AgentSpecies::Blue)
                    .count();

                let red_agents = self.agents[index as usize].iter().count() - blue_agents;
                let blue_graffiti = node.grafitti.get(&AgentSpecies::Blue).unwrap_or(&0.0);
                let red_graffiti = node.grafitti.get(&AgentSpecies::Red).unwrap_or(&0.0);

                write!(
                    f,
                    "| i:{}|b:{},r:{}|g:({},{})",
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
    use super::*;

    fn total_agent_size(universe: &Universe2D) -> u32 {
        universe
            .agents
            .iter()
            .map(|hash_agents| hash_agents.len() as u32)
            .sum()
    }

    fn total_agent_size_of_species(universe: &Universe2D, species: AgentSpecies) -> u32 {
        universe
            .agents
            .iter()
            .map(|hash_agents| {
                hash_agents
                    .iter()
                    .filter(|agent| agent.species == species)
                    .count() as u32
            })
            .sum()
    }

    #[test]
    fn test_universe2d() {
        let universe = Universe2D::new(4, 100);

        assert_eq!(universe.agents.len(), 16);

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
        universe.tick();
        println!("{}", universe);
        universe.tick();
        println!("{}", universe);

        // all notes should have a grafitti of >0.0
        for node in &universe.nodes {
            for species in &AGENT_SPECIES {
                assert_eq!(node.grafitti.contains_key(species), true);
                assert_eq!(node.grafitti.get(species).unwrap() > &0.0, true);
            }
        }
    }
}
