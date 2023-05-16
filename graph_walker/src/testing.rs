#[cfg(test)]
mod test_1 {
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use rand::Rng;
    use rayon::prelude::*;

    use crate::{neighbour_data::NeigbourIndeces, AgentSpecies, Node};

    fn default_node() -> Node {
        let mut edges = HashMap::new();
        edges.insert(0, NeigbourIndeces::new(1, 2, 3, 4));
        let node = Node::new(0, &edges);

        assert_eq!(node.blue_agents, 0);
        assert_eq!(node.red_agents, 0);
        node
    }

    #[test]
    fn it_works_with_one_node() {
        let mut node = default_node();

        let agents: Vec<u32> = (0..10).into_par_iter().map(|_| 1).collect();

        node.blue_agents += agents.iter().sum::<u32>();

        assert_eq!(node.blue_agents, 10);
    }

    #[test]
    fn it_works_with_one_node_arc() {
        let node = default_node();

        let arc_node = Arc::new(Mutex::new(node));

        (0..10000).into_par_iter().for_each(|_| {
            let mut node = arc_node.lock().unwrap();
            node.add_agents(1, AgentSpecies::Blue);
        });

        assert_eq!(arc_node.lock().unwrap().blue_agents, 10000);
    }

    #[test]
    fn it_works_with_ten_node_arc() {
        let nodes = vec![default_node(); 10];

        let arc_node = Arc::new(Mutex::new(nodes));
        const SIZE: usize = 1000000;

        (0..SIZE).into_par_iter().for_each(|_| {
            let rng = rand::thread_rng();
            let mut nodes_guard = arc_node.lock().unwrap();
            // let node = nodes_guard.choose_mut(&mut prng).unwrap();
            let node_idx = rng.clone().gen_range(0..nodes_guard.len());
            nodes_guard[node_idx].add_agents(1, AgentSpecies::Blue);
        });

        let total_blue_agents: u32 = arc_node
            .lock()
            .unwrap()
            .iter()
            .map(|node| node.blue_agents)
            .sum();

        assert_eq!(total_blue_agents as usize, SIZE);
    }
}
