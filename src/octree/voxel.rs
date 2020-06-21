use std::fmt::{Formatter, Error};

#[derive(Copy, Clone, Debug)]
pub struct Voxel {
    data: u16
}

impl std::default::Default for Voxel {
    fn default() -> Self {
        Voxel {
            data: 0,
        }
    }
}
