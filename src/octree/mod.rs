extern crate octree;
pub mod mesh;
#[derive(Eq, PartialEq, Copy, Clone, Default)]
pub struct VoxelData(u16);
impl VoxelData {
    pub const EMPTY: VoxelData = VoxelData(0);
    pub fn is_empty(&self) -> bool {
        *self == Self::EMPTY
    }
}
pub use octree::*;

impl From<u16> for VoxelData {
    fn from(d: u16) -> Self {
        Self(d)
    }
}

pub type Voxel<'a> = octree::voxel::Voxel<'a, VoxelData>;
pub type Chunk = octree::chunk::Chunk<VoxelData>;
pub type WorldBuilder<ORACLE> = octree::world_builder::WorldBuilder<VoxelData, ORACLE>;
