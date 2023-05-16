mod hyper_params;
mod neighbour_data;
mod testing;

use hyper_params::HyperParams;
use neighbour_data::{NeigbourIndeces, NeighbourAgentsOut};
use oorandom::Rand32;
use pad::PadStr;
use rayon::prelude::*;
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

#[derive(Debug, Clone)]
struct Node {
    pub index: u32,
    pub neighbours: NeigbourIndeces, // indices of neighbours
    pub grafitti: [f32; 2],          // [Red_graffiti, Blue_graffiti]
    pub push_strength: [f32; 2],
    pub blue_agents: u32,
    pub red_agents: u32,
    pub agents_out: [NeighbourAgentsOut; 2], // amount of outgoing agents per species
}

impl Node {
    pub fn new(index: u32, edges: &HashMap<u32, NeigbourIndeces>) -> Node {
        Node {
            index,
            neighbours: edges.get(&index).unwrap().to_owned(),
            grafitti: [0.0; 2],
            push_strength: [0.0; 2],
            blue_agents: 0,
            red_agents: 0,
            agents_out: [NeighbourAgentsOut::new(0, 0, 0, 0); 2],
        }
    }

    pub fn get_prng(&self) -> Rand32 {
        Rand32::new((self.index + 1) as u64 * (self.blue_agents + self.red_agents + 1) as u64)
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

    pub fn update_grafitti_and_push_strength(&mut self) {
        // 0 - Decrement current grafitti by lambda
        self.grafitti = self.grafitti.map(|entry| entry * HYPER_PARAMS.lambda);

        // 1 - Increase grafiti by gamma * sum of same agent' count
        self.grafitti[0] += HYPER_PARAMS.gamma * self.red_agents as f32;
        self.grafitti[1] += HYPER_PARAMS.gamma * self.blue_agents as f32;

        self.push_strength[0] += E.powf(-HYPER_PARAMS.beta * self.grafitti[0]);
        self.push_strength[1] += E.powf(-HYPER_PARAMS.beta * self.grafitti[1]);
    }

    pub fn move_agents_out(&mut self, nodes: &Vec<Node>, _grid_size: u32) {
        let neighbours_idx = &self.neighbours;

        // TODO: check if algorithm still works without grid_size
        // 1 - Calculate neighbour strengths
        let mut total_neigh_push_strengths_red = 0.0;
        let mut total_neigh_push_strengths_blue = 0.0;

        let neighbour_push_stengths_iter = neighbours_idx.into_iter().map(|neighbour_idx| {
            let neighbour = &nodes[neighbour_idx as usize];
            let red_push = neighbour.get_push_strength(&AgentSpecies::Red);
            let blue_push = neighbour.get_push_strength(&AgentSpecies::Blue);

            total_neigh_push_strengths_red += red_push;
            total_neigh_push_strengths_blue += blue_push;
            (red_push, blue_push)
        });

        // neighbour_push_stengths.0 is a Vec of all red neighbour push strengths
        // neighbour_push_stengths.1 is a Vec of all blue neighbour push strengths
        let neighbour_push_stengths: (Vec<f32>, Vec<f32>) = neighbour_push_stengths_iter.unzip(); // Vec<(ps1_red, ps2_blue), (ps_2_red, ps2_blue)> => (Vec(ps1_red, ps_2_red), Vec(ps1_blue, ps2_blue))
        assert!(neighbour_push_stengths.0.len() == neighbour_push_stengths.1.len());

        let mut red_agents_out = NeigbourIndeces::new(0, 0, 0, 0);
        let mut blue_agents_out = NeigbourIndeces::new(0, 0, 0, 0);
        let mut prng = self.get_prng();

        // 2 - Move agents out
        for _ in 0..self.red_agents {
            red_agents_out.add_agent_to_random_cell(
                &neighbour_push_stengths.1,      // vec of blue push strengths
                total_neigh_push_strengths_blue, // sum of all blue push strengths
                &mut prng,
            );
        }

        for _ in 0..self.blue_agents {
            blue_agents_out.add_agent_to_random_cell(
                &neighbour_push_stengths.0,     // vec of red push strengths
                total_neigh_push_strengths_red, // sum of all red push strengths
                &mut prng,
            );
        }

        self.agents_out = [red_agents_out, blue_agents_out];
    }

    pub fn move_agents_in(&mut self, nodes: &Vec<Node>) {
        let neighbours_idx = &self.neighbours.clone();
        self.red_agents = 0;
        self.blue_agents = 0;

        // Move agents from the top neighbour to this node which is at the bottom of the top neighbour
        let top_idx = neighbours_idx.top;
        let top_node_agents = nodes[top_idx as usize].agents_out;
        self.add_agents(top_node_agents[0].bottom, AgentSpecies::Red); // top_node_agents[0] is the red agents out of the top neighbour
        self.add_agents(top_node_agents[1].bottom, AgentSpecies::Blue); // top_node_agents[1] is the blue agents out of the top neighbour

        // Move agents from the right neighbour to this node which is at the left of the right neighbour
        let right_idx = neighbours_idx.right;
        let right_node_agents = nodes[right_idx as usize].agents_out;
        self.add_agents(right_node_agents[0].left, AgentSpecies::Red); // right_node_agents[0] is the red agents out of the right neighbour
        self.add_agents(right_node_agents[1].left, AgentSpecies::Blue); // right_node_agents[1] is the blue agents out of the right neighbour

        // Move agents from the bottom neighbour to this node which is at the top of the bottom neighbour
        let bottom_idx = neighbours_idx.bottom;
        let bottom_node_agents = nodes[bottom_idx as usize].agents_out;
        self.add_agents(bottom_node_agents[0].top, AgentSpecies::Red); // bottom_node_agents[0] is the red agents out of the bottom neighbour
        self.add_agents(bottom_node_agents[1].top, AgentSpecies::Blue); // bottom_node_agents[1] is the blue agents out of the bottom neighbour

        // Move agents from the left neighbour to this node which is at the right of the left neighbour
        let left_idx = neighbours_idx.left;
        let left_node_agents = nodes[left_idx as usize].agents_out;
        self.add_agents(left_node_agents[0].right, AgentSpecies::Red); // left_node_agents[0] is the red agents out of the left neighbour
        self.add_agents(left_node_agents[1].right, AgentSpecies::Blue); // left_node_agents[1] is the blue agents out of the left neighbour
    }
}

pub struct Universe2D {
    size: u32,
    nodes: Vec<Node>,
    iteration: u32,
}

impl Universe2D {
    pub fn new(size: u32, agent_size: u32) -> Universe2D {
        let mut prng = Rand32::new(100);

        let mut edges: HashMap<u32, NeigbourIndeces> = HashMap::new(); // TODO: convert to array

        for y in 0..size {
            for x in 0..size {
                let index = y * size + x;

                let left_index = y * size + (x + size - 1) % size;
                let right_index = y * size + (x + 1) % size;
                let top_index = (y + size - 1) % size * size + x;
                let bottom_index = (y + 1) % size * size + x;

                let new_edges =
                    NeigbourIndeces::new(top_index, right_index, bottom_index, left_index);

                edges.insert(index, new_edges);
            }
        }

        let mut nodes: Vec<Node> = (0..(size * size))
            .map(|index| Node::new(index, &edges))
            .collect();

        // Set initial agents
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
            iteration: 0,
        }
    }
}

impl Universe2D {
    pub fn tick(&mut self) {
        // 0) update grafitti in nodes
        self.nodes.par_iter_mut().for_each(|node| {
            node.update_grafitti_and_push_strength();
        });
        let nodes_with_graffiti = self.nodes.clone();

        // 1) move agents out
        self.nodes.par_iter_mut().for_each(|node| {
            node.move_agents_out(&nodes_with_graffiti, self.size);
        });

        // 2) move agents in
        let nodes_with_agents_out = self.nodes.clone();
        self.nodes.par_iter_mut().for_each(|node| {
            node.move_agents_in(&nodes_with_agents_out);
        });

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

    #[test]
    fn test_universe2d() {
        let universe = Universe2D::new(4, 100);

        for node in &universe.nodes {
            // FOR DEBUG println!("{:?}", node);

            assert_eq!(node.neighbours.size, 4);
        }

        fn total_agent_size_of_species(universe: &Universe2D, species: AgentSpecies) -> u32 {
            universe
                .nodes
                .iter()
                .map(|node| node.agents_with_species(&species))
                .sum()
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
    fn test_tick_agent_equal() {
        let mut universe = Universe2D::new(4, 100);

        assert_eq!(total_agent_size(&universe), 200, "0 iteration agents");
        universe.tick();
        assert_eq!(total_agent_size(&universe), 200, "1 iteration agents");

        universe.nodes.iter().for_each(|node| {
            println!(
                "i: {}, red: {}, blue: {}",
                node.index, node.blue_agents, node.red_agents
            );
        })
    }

    #[test]
    fn performance_test_tick() {
        let mut universe = Universe2D::new(100, 100000);

        let start = Instant::now();

        for _ in 0..300 {
            universe.tick();
        }
        // 2.651681208s
        println!("{:?}", start.elapsed());
    }
}
