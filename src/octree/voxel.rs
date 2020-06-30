#[derive(Copy, Clone, Debug)]
pub struct Voxel {
    pub data: u16
}

impl std::default::Default for Voxel {
    fn default() -> Self {
        Voxel {
            data: 0,
        }
    }
}

impl PartialEq for Voxel {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl Eq for Voxel {
}
