use super::voxel::Voxel;
use super::arena::{Arena, ArenaRangeIndice, ArenaNode};
use std::ops::{Index, IndexMut};
use std::hint::unreachable_unchecked;

// Can represent max 21 layers of structures
#[derive(Copy, Clone)]
pub struct IndexPath {
    // Always prepend index path with a 1
    // 0..63 bits: 21 index path, grouped by 3 bits
    data: u64
}

impl IndexPath {
    fn new() -> IndexPath {
        IndexPath {
            data: 1,
        }
    }
    fn empty(&self) -> bool {
        self.data == 1
    }
    fn full(&self) -> bool {
        // Check highest bit
        (self.data >> 63) == 1
    }
    fn peek(&self) -> u8 {
        assert!(!self.empty());
        self.data as u8 & 0b111
    }
    fn pop(&self) -> IndexPath {
        assert!(!self.empty());
        IndexPath {
            data: self.data >> 3,
        }
    }
    fn push(&self, octant: u8) -> IndexPath {
        assert!(octant < 8);
        assert!(!self.full(), "The index path is full");
        IndexPath {
            data: (self.data << 3) | (octant as u64)
        }
    }
}

pub struct Chunk {
    arena: Arena,
    root_node: ArenaRangeIndice,
}

impl Chunk {
    fn new() {

    }
    fn set(&mut self, path: IndexPath, voxel: Voxel) {
        let current = self.root_node;
        let index = path.peek();
    }
}


#[cfg(test)]
mod tests {

    #[test]
    fn test_available_at() {
    }
}

