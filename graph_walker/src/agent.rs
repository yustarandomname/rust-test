use enum_iterator::{all, All, Sequence};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Sequence)]
pub enum AgentSpecies {
    Red,
    Blue,
}

impl AgentSpecies {
    /**
     * Iterate over all possible agent species
     */
    pub fn iter() -> All<AgentSpecies> {
        all::<AgentSpecies>()
    }

    /**
     * A vector of all possible agent species
     */
    pub fn all() -> Vec<AgentSpecies> {
        all::<AgentSpecies>().collect::<Vec<_>>()
    }
}

#[derive(Clone, Debug, PartialEq, Hash, Eq)]
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
    fn all_species() {
        let species = AgentSpecies::all();

        assert_eq!(species.len(), 2);
        assert!(species.contains(&AgentSpecies::Red));
        assert!(species.contains(&AgentSpecies::Blue));
    }

    #[test]
    fn iter_species() {
        let species = AgentSpecies::iter().collect::<Vec<AgentSpecies>>();

        assert_eq!(species.len(), 2);
        assert!(species.contains(&AgentSpecies::Red));
        assert!(species.contains(&AgentSpecies::Blue));
    }

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
