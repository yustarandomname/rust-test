mod test_1 {
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use rand::seq::SliceRandom;
    use rayon::prelude::*;

    use crate::{AgentSpecies, Node};

    #[test]
    fn it_works_with_one_node() {
        // 0
        let mut node = Node {
            neighbours: vec![],
            grafitti: HashMap::new(),
            pull_strength: HashMap::new(),
            blue_agents: 0,
            red_agents: 0,
        };

        let agents: Vec<u32> = (0..10).into_par_iter().map(|_| 1).collect();

        node.blue_agents += agents.iter().sum::<u32>();

        assert_eq!(node.blue_agents, 10);
    }

    #[test]
    fn it_works_with_one_node_arc() {
        let node = Node {
            neighbours: vec![],
            grafitti: HashMap::new(),
            pull_strength: HashMap::new(),
            blue_agents: 0,
            red_agents: 0,
        };

        let arc_node = Arc::new(Mutex::new(node));

        (0..10000).into_par_iter().for_each(|_| {
            let mut node = arc_node.lock().unwrap();
            node.add_agents(1, AgentSpecies::Blue);
        });

        assert_eq!(arc_node.lock().unwrap().blue_agents, 10000);
    }

    #[test]
    fn it_works_with_ten_node_arc() {
        let node = Node {
            neighbours: vec![],
            grafitti: HashMap::new(),
            pull_strength: HashMap::new(),
            blue_agents: 0,
            red_agents: 0,
        };
        let nodes = vec![node.clone(); 10];

        let arc_node = Arc::new(Mutex::new(nodes));
        let prng = rand::thread_rng();
        const SIZE: usize = 100000;

        (0..SIZE).into_par_iter().for_each(|_| {
            let mut nodes_guard = arc_node.lock().unwrap();
            // let node = nodes_guard.choose_mut(&mut prng).unwrap();
            nodes_guard[0].add_agents(1, AgentSpecies::Blue);
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
