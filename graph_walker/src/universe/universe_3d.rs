use super::universe::Universe;
use crate::{
    agent_species::AgentSpecies,
    hyper_params::HyperParams,
    neighbour_data::{NeigbourIndeces3D, NeighbourData3D},
    nodes::Node3D,
};
use oorandom::Rand32;
use pad::PadStr;
use rayon::prelude::*;
use std::{collections::HashMap, fmt};

pub struct Universe3D {
    size: u32,
    nodes: Vec<Node3D>,
    iteration: u32,
    hyper_params: HyperParams,
}

impl Universe for Universe3D {
    fn new(size: u32, agent_size: u32) -> Universe3D {
        let mut prng = Rand32::new(100);

        let mut edges: HashMap<u32, NeigbourIndeces3D> = HashMap::new(); // TODO: convert to array

        for z in 0..size {
            for y in 0..size {
                for x in 0..size {
                    let index = z * (size * size) + y * size + x;

                    let top_index = ((z + size - 1) % size) * (size * size) + y * size + x;
                    let bottom_index = ((z + 1) % size) * (size * size) + y * size + x;
                    let front_index = z * (size * size) + ((y + size - 1) % size) * size + x;
                    let back_index = z * (size * size) + ((y + 1) % size) * size + x;
                    let left_index = z * (size * size) + y * size + (x + size - 1) % size;
                    let right_index = z * (size * size) + y * size + (x + 1) % size;

                    let new_edges = NeigbourIndeces3D::new(
                        top_index,
                        right_index,
                        bottom_index,
                        left_index,
                        front_index,
                        back_index,
                    );

                    edges.insert(index, new_edges);
                }
            }
        }

        let mut nodes: Vec<Node3D> = (0..(size * size * size))
            .map(|index| Node3D::new(index, &edges))
            .collect();

        // Set initial agents
        (0..agent_size * 2).for_each(|id| {
            let node_index = prng.rand_range(0..(size * size * size));
            let species = if id % 2 == 0 {
                AgentSpecies::Red
            } else {
                AgentSpecies::Blue
            };

            nodes[node_index as usize].add_agents(1, species);
        });

        Universe3D {
            size,
            nodes,
            iteration: 0,
            hyper_params: HyperParams::default(),
        }
    }

    fn set_hyper_params(&mut self, hyper_params: HyperParams) {
        self.hyper_params = hyper_params;
    }

    fn tick(&mut self) {
        // 0) update graffiti in nodes
        self.nodes.par_iter_mut().for_each(|node| {
            node.update_graffiti_and_push_strength(&self.hyper_params, self.size);
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

    fn iterate(&mut self, iterations: u32) {
        for _ in 0..iterations {
            self.tick();
        }
    }
}

impl fmt::Debug for Universe3D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} UNIVERSE 3D {}\n", "=".repeat(10), "=".repeat(10))?;

        write!(f, "size: {}\n", self.size)?;
        write!(f, "node size: {}\n", self.nodes.len())?;
        write!(f, "iterations: {}\n", self.iteration)?;

        write!(f, "{}\n", "=".repeat(30))?;
        for z in 0..self.size {
            for y in 0..self.size {
                for x in 0..self.size {
                    let index: u32 = z * (self.size * self.size) + y * self.size + x;
                    let node = &self.nodes[index as usize];

                    let blue_agents =
                        self.nodes[index as usize].get_agents_with_species(&AgentSpecies::Blue);
                    let red_agents =
                        self.nodes[index as usize].get_agents_with_species(&AgentSpecies::Red);

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
        }
        write!(f, "")
    }
}

impl fmt::Display for Universe3D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} UNIVERSE 3D {}\n", "=".repeat(10), "=".repeat(10))?;

        write!(f, "size: {}\n", self.size)?;
        write!(f, "node size: {}\n", self.nodes.len())?;
        write!(f, "iterations: {}\n", self.iteration)?;

        for z in 0..self.size {
            write!(f, "z: {}\n", z)?;
            for y in 0..self.size {
                for x in 0..self.size {
                    let index = z * (self.size * self.size) + y * self.size + x;
                    let node = &self.nodes[index as usize];

                    let blue_graffiti = node.graffiti.blue;
                    let red_graffiti = node.graffiti.red;

                    let delta = blue_graffiti - red_graffiti;

                    if delta.abs() < 0.1 {
                        write!(f, "🟩")?;
                    } else if delta > 0.0 {
                        write!(f, "🟦")?;
                    } else {
                        write!(f, "🟥")?;
                    }
                }
                write!(f, "\n")?;
            }
            write!(f, "\n")?;
        }

        write!(f, "")
    }
}
