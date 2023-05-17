use oorandom::Rand32;
use std::{collections::HashMap, f32::consts::E};

use crate::{
    agent_species::AgentSpecies,
    hyper_params::HyperParams,
    neighbour_data::{NeigbourIndeces2D, NeighbourAgentsOut2D, NeighbourData, NeighbourData2D},
    species::{SpeciesGraffiti, SpeciesPushStrength},
};

#[derive(Debug, Clone)]
pub struct Node {
    pub index: u32,
    pub neighbours: NeigbourIndeces2D,      // indices of neighbours
    pub graffiti: SpeciesGraffiti,          // {Red_graffiti, Blue_graffiti}
    pub push_strength: SpeciesPushStrength, // {Red_graffiti, Blue_graffiti}
    pub blue_agents: u32,
    pub red_agents: u32,
    pub agents_out: [NeighbourAgentsOut2D; 2], // amount of outgoing agents per species
}

impl Node {
    pub fn new(index: u32, edges: &HashMap<u32, NeigbourIndeces2D>) -> Node {
        Node {
            index,
            neighbours: edges.get(&index).unwrap().to_owned(),
            graffiti: SpeciesGraffiti::new(0.0, 0.0),
            push_strength: SpeciesPushStrength::new(0.0, 0.0),
            blue_agents: 0,
            red_agents: 0,
            agents_out: [NeighbourAgentsOut2D::new(0, 0, 0, 0); 2],
        }
    }

    pub fn get_prng(&self) -> Rand32 {
        Rand32::new((self.index + 1) as u64 * (self.blue_agents + self.red_agents + 1) as u64)
    }

    pub fn get_push_strength(&self, species: &AgentSpecies) -> f32 {
        match species {
            AgentSpecies::Red => self.push_strength.red,
            AgentSpecies::Blue => self.push_strength.blue,
        }
    }

    pub fn add_agents(&mut self, amount: u32, species: AgentSpecies) {
        match species {
            AgentSpecies::Red => self.red_agents += amount,
            AgentSpecies::Blue => self.blue_agents += amount,
        }
    }

    pub fn get_agents_with_species(&self, species: &AgentSpecies) -> u32 {
        match species {
            AgentSpecies::Blue => self.red_agents,
            AgentSpecies::Red => self.blue_agents,
        }
    }

    /**
     * Apply this function ξ_i(x, y, t + 𝝳t) = ξ_i(x, y, t) - (ƛ · 𝝳t)ξi(x, y, t) + (𝛾 · 𝝳t)𝞺_i(x, y, t)
     * Xi_i = graffiti of species i at location x,y multiplied by 1/(l^2) [as defined in paper: ξ_A(x, y, t) = g_i(x, y, t)/l2]
     * ƛ = decay rate
     * 𝛾 = deposition rate
     * 𝞺_i = sum of graffiti of species i at location x,y multiplied by 1/(l^2) [as defined in paper: 𝞺_i(x, y, t) = n_i(x, y, t)/l2]
     */
    pub fn update_graffiti_and_push_strength(
        &mut self,
        hyper_params: &HyperParams,
        _grid_size: u32,
    ) {
        let l_squared: f32 = 1.0; //(1.0 / grid_size as f32).powf(2.0);
                                  // TODO: check if algorithm still works with grid_size

        // 0 - Decrement current graffiti by lambda
        self.graffiti.mult_all(1.0 - hyper_params.lambda);

        // 1 - Increase grafiti by gamma * sum of same agent' count
        self.graffiti
            .add_red(hyper_params.gamma * self.red_agents as f32 / l_squared);
        self.graffiti
            .add_blue(hyper_params.gamma * self.blue_agents as f32 / l_squared);

        // 2 - Calculate push strength
        self.push_strength
            .set_red(E.powf(-hyper_params.beta * self.graffiti.red / l_squared));
        self.push_strength
            .set_blue(E.powf(-hyper_params.beta * self.graffiti.blue / l_squared));
    }

    pub fn move_agents_out(&mut self, nodes: &Vec<Node>, _grid_size: u32) {
        let neighbours_idx = &self.neighbours;

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

        let mut red_agents_out = NeigbourIndeces2D::new(0, 0, 0, 0);
        let mut blue_agents_out = NeigbourIndeces2D::new(0, 0, 0, 0);
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
