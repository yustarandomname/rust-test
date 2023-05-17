use oorandom::Rand32;

pub trait NeighbourData: IntoIterator {
    fn add_agent_to_random_cell(
        &mut self,
        neighbour_push_stengths: &Vec<f32>,
        total_neighbour_push_stengths: f32,
        prng: &mut Rand32,
    );
}

pub trait NeighbourData2D: NeighbourData {
    fn new(top: u32, right: u32, bottom: u32, left: u32) -> Self;
}

pub trait NeighbourData3D: NeighbourData {
    fn new(top: u32, right: u32, bottom: u32, left: u32, front: u32, back: u32) -> Self;
}
