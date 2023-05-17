use oorandom::Rand32;

use super::neighbour_data::{NeighbourData, NeighbourData3D};

#[derive(Debug, Clone, Copy)]
pub struct Neighbours3D {
    pub top: u32,
    pub bottom: u32,
    pub left: u32,
    pub right: u32,
    pub front: u32,
    pub back: u32,
    pub size: u32,
}

impl NeighbourData for Neighbours3D {
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
                    4 => self.front += 1,
                    5 => self.back += 1,
                    _ => panic!("Invalid neighbour index"),
                }
                break;
            }
        }
    }
}

impl NeighbourData3D for Neighbours3D {
    fn new(top: u32, right: u32, bottom: u32, left: u32, front: u32, back: u32) -> Neighbours3D {
        Neighbours3D {
            top,
            bottom,
            left,
            right,
            front,
            back,
            size: 4,
        }
    }
}

impl IntoIterator for Neighbours3D {
    type Item = u32;
    type IntoIter = NeighboursIntoIterator3D;

    fn into_iter(self) -> Self::IntoIter {
        NeighboursIntoIterator3D {
            neighbours: self,
            index: 0,
        }
    }
}

pub struct NeighboursIntoIterator3D {
    neighbours: Neighbours3D,
    index: u32,
}

impl Iterator for NeighboursIntoIterator3D {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        let result = match self.index {
            0 => self.neighbours.top,
            1 => self.neighbours.right,
            2 => self.neighbours.bottom,
            3 => self.neighbours.left,
            4 => self.neighbours.front,
            5 => self.neighbours.back,
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
        let neighbours_idx = Neighbours3D::new(1, 2, 3, 4, 5, 6);
        let mut iter = neighbours_idx.into_iter();
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(4));
        assert_eq!(iter.next(), Some(5));
        assert_eq!(iter.next(), Some(6));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_add_agent_to_random_cell1() {
        let mut neighbours_out = Neighbours3D::new(0, 0, 0, 0, 0, 0);

        let neighbour_push_stength = vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0]; // chance of choosing top is 1.0 others are 0.0
        let prng = &mut Rand32::new(0);

        neighbours_out.add_agent_to_random_cell(&neighbour_push_stength, 1.0, prng);

        assert_eq!(neighbours_out.top, 1);
        assert_eq!(neighbours_out.right, 0);
        assert_eq!(neighbours_out.bottom, 0);
        assert_eq!(neighbours_out.left, 0);
        assert_eq!(neighbours_out.front, 0);
        assert_eq!(neighbours_out.right, 0);
    }

    #[test]
    fn test_add_agent_to_random_cell2() {
        let mut neighbours_out = Neighbours3D::new(0, 0, 0, 0, 0, 0);

        let neighbour_push_stength = vec![1.0, 2.0, 3.0, 6.0, 12.0, 24.0]; // chance of choosing top is 1.0 others are 0.0
        let neighbour_push_stength_total: f32 = neighbour_push_stength.iter().sum(); // = 48.0
        let prng = &mut Rand32::new(0);

        for _ in 0..480_000 {
            neighbours_out.add_agent_to_random_cell(
                &neighbour_push_stength,
                neighbour_push_stength_total,
                prng,
            );
        }

        assert_eq!(neighbours_out.top, 9937); // aprox 120_000/48*1 = 10_000
        assert_eq!(neighbours_out.right, 20120); // aprox 120_000/48*2 = 20_000
        assert_eq!(neighbours_out.bottom, 29743); // aprox 120_000/48*3 = 30_000
        assert_eq!(neighbours_out.left, 60094); // aprox 120_000/48*6 = 60_000
        assert_eq!(neighbours_out.front, 120394); // aprox 120_000/48*12 = 120_000
        assert_eq!(neighbours_out.back, 239712); // aprox 120_000/48*24 = 240_000
    }
}
