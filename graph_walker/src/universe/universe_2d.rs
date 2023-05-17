use super::universe::Universe;
use crate::{
    agent_species::AgentSpecies, hyper_params::HyperParams, neighbour_data::NeigbourIndeces2D,
    neighbour_data::NeighbourData2D, node::Node,
};
use oorandom::Rand32;
use pad::PadStr;
use rayon::prelude::*;
use std::{collections::HashMap, fmt};

pub struct Universe2D {
    size: u32,
    nodes: Vec<Node>,
    iteration: u32,
    hyper_params: HyperParams,
}

impl Universe for Universe2D {
    fn new(size: u32, agent_size: u32) -> Universe2D {
        let mut prng = Rand32::new(100);

        let mut edges: HashMap<u32, NeigbourIndeces2D> = HashMap::new(); // TODO: convert to array

        for y in 0..size {
            for x in 0..size {
                let index = y * size + x;

                let left_index = y * size + (x + size - 1) % size;
                let right_index = y * size + (x + 1) % size;
                let top_index = (y + size - 1) % size * size + x;
                let bottom_index = (y + 1) % size * size + x;

                let new_edges =
                    NeigbourIndeces2D::new(top_index, right_index, bottom_index, left_index);

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
            write!(f, "|\n")?;
        }
        write!(f, "")
    }
}

#[cfg(test)]
mod test_2d_universe {
    use crate::agent_species::AgentSpecies;

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
            // assert that each node has 4 neighbours
            assert_eq!(node.neighbours.size, 4);
        }

        fn total_agent_size_of_species(universe: &Universe2D, species: AgentSpecies) -> u32 {
            universe
                .nodes
                .iter()
                .map(|node| node.get_agents_with_species(&species))
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
        universe.tick();
        assert_eq!(total_agent_size(&universe), 200, "2 iteration agents");

        let cache = vec![
            (5, 5),
            (8, 2),
            (4, 11),
            (13, 7),
            (8, 6),
            (6, 5),
            (5, 8),
            (5, 7),
            (5, 5),
            (4, 6),
            (10, 4),
            (3, 2),
            (9, 8),
            (6, 10),
            (5, 7),
            (4, 7),
        ];

        let mut universe_hash_i = 0;

        universe
            .nodes
            .iter()
            .zip(cache)
            .for_each(|(node, cache_node_agents)| {
                universe_hash_i += node.blue_agents + (node.red_agents * (node.index + 1));
                print!(
                    "({}, {}, {}), ",
                    node.index, node.red_agents, node.blue_agents
                );
                assert_eq!(
                    node.red_agents, cache_node_agents.0,
                    "red agents on index {}",
                    node.index
                );
                assert_eq!(
                    node.blue_agents, cache_node_agents.1,
                    "blue agents on index {}",
                    node.index
                );
            });
        println!("universe_hash_i: {}", universe_hash_i);
    }
}
