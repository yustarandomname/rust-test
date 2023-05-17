use oorandom::Rand32;

use super::neighbour_data::{NeighbourData, NeighbourData2D};

#[derive(Debug, Clone, Copy)]
pub struct Neighbours2D {
    pub top: u32,
    pub bottom: u32,
    pub left: u32,
    pub right: u32,
    pub size: u32,
}

impl NeighbourData for Neighbours2D {
    fn add_agent_to_random_cell(
        &mut self,
        neighbour_push_stengths: &Vec<f32>,
        total_neighbour_push_stengths: f32,
        prng: &mut Rand32,
    ) {
        let random_number = prng.rand_float() * total_neighbour_push_stengths;
        let mut sum = 0.0;
        for (i, neighbour_push_stength) in neighbour_push_stengths.iter().enumerate() {
            sum += neighbour_push_stength;
            if sum >= random_number {
                match i {
                    0 => self.top += 1,
                    1 => self.right += 1,
                    2 => self.bottom += 1,
                    3 => self.left += 1,
                    _ => panic!("Invalid neighbour index"),
                }
                break;
            }
        }
    }
}

impl NeighbourData2D for Neighbours2D {
    fn new(top: u32, right: u32, bottom: u32, left: u32) -> Neighbours2D {
        Neighbours2D {
            top,
            bottom,
            left,
            right,
            size: 4,
        }
    }
}

impl IntoIterator for Neighbours2D {
    type Item = u32;
    type IntoIter = NeighboursIntoIterator2D;

    fn into_iter(self) -> Self::IntoIter {
        NeighboursIntoIterator2D {
            neighbours: self,
            index: 0,
        }
    }
}

pub struct NeighboursIntoIterator2D {
    neighbours: Neighbours2D,
    index: u32,
}

impl Iterator for NeighboursIntoIterator2D {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        let result = match self.index {
            0 => self.neighbours.top,
            1 => self.neighbours.right,
            2 => self.neighbours.bottom,
            3 => self.neighbours.left,
            _ => return None,
        };
        self.index += 1;
        Some(result)
    }
}

#[cfg(test)]
mod test_neighbours {
    use super::*;

    #[test]
    fn test_into_iter() {
        let neighbours_idx = Neighbours2D::new(1, 2, 3, 4);
        let mut iter = neighbours_idx.into_iter();
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(4));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_add_agent_to_random_cell1() {
        let mut neighbours_out = Neighbours2D::new(0, 0, 0, 0);

        let neighbour_push_stength = vec![1.0, 0.0, 0.0, 0.0]; // chance of choosing top is 1.0 others are 0.0
        let prng = &mut Rand32::new(0);

        neighbours_out.add_agent_to_random_cell(&neighbour_push_stength, 1.0, prng);

        assert_eq!(neighbours_out.top, 1);
        assert_eq!(neighbours_out.right, 0);
        assert_eq!(neighbours_out.bottom, 0);
        assert_eq!(neighbours_out.left, 0);
    }

    #[test]
    fn test_add_agent_to_random_cell2() {
        let mut neighbours_out = Neighbours2D::new(0, 0, 0, 0);

        let neighbour_push_stength = vec![1.0, 2.0, 3.0, 6.0]; // chance of choosing top is 1.0 others are 0.0
        let prng = &mut Rand32::new(0);

        for _ in 0..120_000 {
            neighbours_out.add_agent_to_random_cell(&neighbour_push_stength, 12.0, prng);
        }

        assert_eq!(neighbours_out.top, 9982); // aprox 120_000/12 = 10_000
        assert_eq!(neighbours_out.right, 20142); // aprox 120_000/6 = 20_000
        assert_eq!(neighbours_out.bottom, 30029); // aprox 120_000/4 = 30_000
        assert_eq!(neighbours_out.left, 59847); // aprox 120_000/2 = 60_000
    }
}
