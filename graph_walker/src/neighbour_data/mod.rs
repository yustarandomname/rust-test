mod neighbour_data;
mod neighbour_data_2d;
mod neighbour_data_3d;

pub use neighbour_data::NeighbourData;
pub use neighbour_data::NeighbourData2D;
pub use neighbour_data::NeighbourData3D;
use neighbour_data_2d::Neighbours2D;
use neighbour_data_3d::Neighbours3D;

pub type NeigbourIndeces2D = Neighbours2D;
pub type NeighbourAgentsOut2D = Neighbours2D;

pub type NeigbourIndeces3D = Neighbours3D;
pub type NeighbourAgentsOut3D = Neighbours3D;
