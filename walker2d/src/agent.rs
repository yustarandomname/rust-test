#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AgentSpecies {
    Red,
    Blue,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Agent {
    id: String,
    pub species: AgentSpecies,
}

impl Agent {
    pub fn new(id: String, species: AgentSpecies) -> Agent {
        Agent { id, species }
    }
}

#[cfg(test)]
mod tests {
    use rand::{RngCore, SeedableRng};
    use rand_chacha::ChaCha8Rng;

    use super::*;

    #[test]
    fn new_agent() {
        let agent = Agent::new("test".to_string(), AgentSpecies::Red);

        assert_eq!(agent.id, "test");
        assert_eq!(agent.species, AgentSpecies::Red);
    }

    #[test]
    fn all_id_unique() {
        let mut prng = ChaCha8Rng::seed_from_u64(2);
        let agents = (0..100)
            .map(|agent| {
                Agent::new(
                    agent.to_string(),
                    if prng.next_u32() % 2 == 0 {
                        AgentSpecies::Red
                    } else {
                        AgentSpecies::Blue
                    },
                )
            })
            .collect::<Vec<Agent>>();

        let mut ids = agents.iter().map(|a| a.id.clone()).collect::<Vec<String>>();

        ids.sort();

        let mut unique = ids.clone();
        unique.dedup();

        assert_eq!(ids, unique);
    }
}
