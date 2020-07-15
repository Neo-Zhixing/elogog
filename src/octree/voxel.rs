use std::fmt::Debug;

#[derive(Copy, Clone)]
pub struct Voxel {
    pub data: u16
}

impl std::fmt::Debug for Voxel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:02X?}", self.data)
    }
}

impl std::default::Default for Voxel {
    fn default() -> Self {
        Voxel {
            data: 0,
        }
    }
}

impl Voxel {
    #[inline]
    pub(crate) fn new(data: u16) -> Self {
        Voxel {
            data
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
