use super::voxel::Voxel;
use super::arena::{Arena, ArenaNodeIndice, ArenaNode};
use std::ops::{Index, IndexMut};
use std::fmt::Write;
use crate::util::tuple_strip::IterUtil;

// Can represent max 21 layers of structures
// Always prepend index path with a 1
// 0..63 bits: 21 index path, grouped by 3 bits
#[derive(Copy, Clone)]
pub struct IndexPath (std::num::NonZeroU64);

impl std::fmt::Debug for IndexPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut current = self.clone();
        while !current.is_empty() {
            f.write_char((current.peek() + '0' as u8).into())?;
            current = current.pop();
            f.write_char('/')?;
        }
        Ok(())
    }
}

impl IndexPath {
    const MAX_SIZE: u8 = 21;
    fn empty() -> IndexPath {
        unsafe {
            IndexPath(std::num::NonZeroU64::new_unchecked(1))
        }
    }
    fn new(octant: u8) -> IndexPath {
        IndexPath::empty().push(octant)
    }
    fn is_empty(&self) -> bool {
        self.0.get() == 1
    }
    fn is_full(&self) -> bool {
        // Check highest bit
        (self.0.get() >> 63) == 1
    }
    fn peek(&self) -> u8 {
        assert!(!self.is_empty());
        self.0.get() as u8 & 0b111
    }
    fn pop(&self) -> IndexPath {
        assert!(!self.is_empty());
        unsafe {
            IndexPath(std::num::NonZeroU64::new_unchecked(self.0.get() >> 3))
        }
    }
    fn push(&self, octant: u8) -> IndexPath {
        assert!(octant < 8);
        assert!(!self.is_full(), "The index path is full");
        unsafe {
            IndexPath(std::num::NonZeroU64::new_unchecked((self.0.get() << 3) | (octant as u64)))
        }
    }
    fn replace(&self, octant: u8) -> IndexPath {
        assert!(octant < 8);
        unsafe {
            IndexPath(std::num::NonZeroU64::new_unchecked((self.0.get() & !0b111) | (octant as u64)))
        }
    }
    fn len(&self) -> u8 {
        let num_empty_slots = self.0.get().leading_zeros() as u8 / 3;
        Self::MAX_SIZE - num_empty_slots
    }
}

pub struct Chunk {
    arena: Arena,
    root_node: ArenaNodeIndice,
}

impl Chunk {
    pub fn new() -> Chunk {
        let mut arena = Arena::new();
        let root_node = arena.alloc(1).child(0);
        Chunk {
            arena,
            root_node,
        }
    }
    fn condense_along_path(&mut self, path: &Vec<(ArenaNodeIndice, u8)>) {
        for ((current_index, _), (parent_index, dir)) in path.iter().rev().tuple_strip() {
            let current_node = self.arena.get_node(*current_index);
            if !current_node.is_condensable() {
                return;
            }
            let voxel = current_node.data[0];
            let dir = *dir;
            let parent_node = self.arena.get_node_mut(*parent_index);
            parent_node.set_on_dir(dir, voxel);
            assert!(parent_node.has_child_on_dir(dir));
            let new_mask = parent_node.leaf_mask & !(1 << dir);
            self.arena.realloc(*parent_index, new_mask);
        }
    }
    pub fn set(&mut self, path: IndexPath, voxel: Voxel) {
        let mut current = path;
        let mut node_index = self.root_node;

        // Stack saving the node index, and the dir to take next
        let mut node_index_stack = Vec::with_capacity(path.len() as usize);
        loop {
            // Strip the top most path element
            let dir = current.peek();
            node_index_stack.push((node_index, dir));
            current = current.pop();
            if current.is_empty() { // If this is the final path
                // Set the leaf
                // node.data[dir as usize] = voxel;
                self.arena.get_node_mut(node_index).set_on_dir(dir, voxel);

                // Condense the octree
                self.condense_along_path(&node_index_stack);
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
    pub fn sample(&self, path: IndexPath) -> Voxel {
        let mut current = path;
        let mut node_index = self.root_node;
        loop {
            // Strip the top most path element
            let dir = current.peek();
            current = current.pop();
            let node = self.arena.get_node(node_index);
            if current.is_empty() { // If this is the final path ||
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
    use std::mem::size_of;
    use rand::Rng;

    #[test]
    fn test_index_path() {
        assert_eq!(size_of::<IndexPath>(), size_of::<u64>());
        assert_eq!(size_of::<Option<IndexPath>>(), size_of::<u64>());

        let mut path = IndexPath::empty();
        for i in 0..IndexPath::MAX_SIZE {
            assert_eq!(path.len(), i);
            path = path.push(0);
        }
        assert_eq!(path.len(), IndexPath::MAX_SIZE);
    }

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

    #[test]
    fn test_condense_on_set() {
        let mut chunk = Chunk::new();
        for i in 0 ..8 {
            let index = IndexPath::new(i).push(1);
            chunk.set(index, Voxel { data: 13 });
            assert_eq!(chunk.sample(IndexPath::new(i).push(1)), Voxel { data: 13 });
        }
    }
}

