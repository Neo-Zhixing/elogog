use super::voxel::Voxel;
use super::arena::{Arena, ArenaNodeIndice};
use std::ops::{Index, IndexMut};
use std::fmt::Write;

// Can represent max 21 layers of structures
#[derive(Copy, Clone)]
pub struct IndexPath {
    // Always prepend index path with a 1
    // 0..63 bits: 21 index path, grouped by 3 bits
    data: u64
}

impl std::fmt::Debug for IndexPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut current = self.clone();
        while !current.empty() {
            f.write_char((current.peek() + '0' as u8).into())?;
            current = current.pop();
            f.write_char('/')?;
        }
        Ok(())
    }
}

impl IndexPath {
    fn new(octant: u8) -> IndexPath {
        IndexPath {
            data: 1,
        }.push(octant)
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
    root_node: ArenaNodeIndice,
}

impl Chunk {
    fn new() -> Chunk {
        let mut arena = Arena::new();
        let root_node = arena.alloc(1).child(0);
        Chunk {
            arena,
            root_node,
        }
    }
    fn set(&mut self, path: IndexPath, voxel: Voxel) {
        let mut current = path;
        let mut node_index = self.root_node;
        loop {
            // Strip the top most path element
            let dir = current.peek();
            current = current.pop();
            if current.empty() { // If this is the final path ||
                // Set the leaf
                // node.data[dir as usize] = voxel;
                self.arena.get_node_mut(node_index).set_on_dir(dir, voxel);
                return;
            } else if let Some(child) = self.arena.get_node(node_index).child_on_dir(dir) {
                // If there is already a child there
                node_index = child;
                continue;
            } else {
                // Realloc
                let old_freemask = self.arena.get_node(node_index).leaf_mask;
                self.arena.realloc(node_index, old_freemask | (1 << dir));
                node_index = self.arena.get_node(node_index).child_on_dir(dir).unwrap();
                continue;
            }
        }
    }
    fn sample(&self, path: IndexPath) -> Voxel {
        let mut current = path;
        let mut node_index = self.root_node;
        loop {
            // Strip the top most path element
            let dir = current.peek();
            current = current.pop();
            let node = self.arena.get_node(node_index);
            if current.empty() { // If this is the final path ||
                // Get the leaf
                return node.data[dir as usize];
            } else if let Some(child) = self.arena.get_node(node_index).child_on_dir(dir) {
                // If there is already a child there
                node_index = child;
                continue;
            } else {
                return node.data[dir as usize];
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::octree::chunk::{Chunk, IndexPath};
    use crate::octree::voxel::Voxel;
    use rand::Rng;

    #[test]
    fn test_set_first_level() {
        let mut chunk = Chunk::new();
        for i in 0..8 {
            assert_eq!(chunk.sample(IndexPath::new(i)), Voxel { data: 0 });
        }
        for i in 0..8 {
            chunk.set(IndexPath::new(i), Voxel { data: i as u16 });
        }
        for i in 0..8 {
            assert_eq!(chunk.sample(IndexPath::new(i)), Voxel { data: i as u16 });
        }
    }
    #[test]
    fn test_set_second_level() {
        let mut chunk = Chunk::new();
        let index_path = IndexPath::new(0).push(1);
        chunk.set(index_path, Voxel { data: 13 });
        assert_eq!(chunk.sample(index_path), Voxel { data: 13 });

        // Another allocation
        chunk.set(IndexPath::new(0).push(2), Voxel { data : 12 });
        assert_eq!(chunk.sample(IndexPath::new(0).push(1)), Voxel { data: 13 });
        assert_eq!(chunk.sample(IndexPath::new(0).push(2)), Voxel { data: 12 });

        chunk.set(IndexPath::new(0).push(5), Voxel { data : 5 });
        assert_eq!(chunk.sample(IndexPath::new(0).push(1)), Voxel { data: 13 });
        assert_eq!(chunk.sample(IndexPath::new(0).push(2)), Voxel { data: 12 });
        assert_eq!(chunk.sample(IndexPath::new(0).push(5)), Voxel { data: 5 });

        chunk.set(IndexPath::new(1).push(4), Voxel { data : 4 });
        assert_eq!(chunk.sample(IndexPath::new(0).push(1)), Voxel { data: 13 });
        assert_eq!(chunk.sample(IndexPath::new(0).push(2)), Voxel { data: 12 });
        assert_eq!(chunk.sample(IndexPath::new(0).push(5)), Voxel { data: 5 });
        assert_eq!(chunk.sample(IndexPath::new(1).push(4)), Voxel { data: 4 });

        chunk.set(IndexPath::new(7).push(6), Voxel { data : 86 });
        assert_eq!(chunk.sample(IndexPath::new(0).push(1)), Voxel { data: 13 });
        assert_eq!(chunk.sample(IndexPath::new(0).push(2)), Voxel { data: 12 });
        assert_eq!(chunk.sample(IndexPath::new(0).push(5)), Voxel { data: 5 });
        assert_eq!(chunk.sample(IndexPath::new(1).push(4)), Voxel { data: 4 });
        assert_eq!(chunk.sample(IndexPath::new(7).push(6)), Voxel { data: 86 });
    }
}

