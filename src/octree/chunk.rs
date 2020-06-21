use super::voxel::Voxel;
use super::arena::Arena;
use super::node::Node;
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

pub struct Chunk{
    arena1: Vec<Box<Arena<1>>>,
    arena2: Vec<Box<Arena<2>>>,
    arena3: Vec<Box<Arena<3>>>,
    arena4: Vec<Box<Arena<4>>>,
    arena5: Vec<Box<Arena<5>>>,
    arena6: Vec<Box<Arena<6>>>,
    arena7: Vec<Box<Arena<7>>>,
    arena8: Vec<Box<Arena<8>>>,
    root: Node,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            arena1: Default::default(),
            arena2: Default::default(),
            arena3: Default::default(),
            arena4: Default::default(),
            arena5: Default::default(),
            arena6: Default::default(),
            arena7: Default::default(),
            arena8: Default::default(),
            root: Default::default(),
        }
    }
    pub fn get_children(&self, node: &Node) -> &[Node]{
        let node_index = (node.children_index & 0b11111111) as u8; // 8 bits
        let arena_index = (node.children_index >> 8) as usize;
        match node.num_child_nodes() {
            0 => &[],
            1 => &self.arena1[arena_index][node_index],
            2 => &self.arena2[arena_index][node_index],
            3 => &self.arena3[arena_index][node_index],
            4 => &self.arena4[arena_index][node_index],
            5 => &self.arena5[arena_index][node_index],
            6 => &self.arena6[arena_index][node_index],
            7 => &self.arena7[arena_index][node_index],
            8 => &self.arena8[arena_index][node_index],
            _ => unreachable!(),
        }
    }
    pub fn sample(&self, i: IndexPath) -> Voxel {
        assert!(!i.empty(), "Can't sample a chunk with an empty index path");
        let mut current = i;
        let mut node = &self.root;
        loop {
            let dir = current.peek();
            current = current.pop();
            if current.empty() || !node.has_child_on(dir) {
                return node.data[dir as usize];
            } else {
                let children = self.get_children(node);
                node = &children[node.get_child_node_index_on(dir) as usize];
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::Chunk;
    use super::IndexPath;

    #[test]
    fn test_sample() {
        let mut chunk = Chunk::new();
        let mut index_path = IndexPath::new().push(0b000).push(0b101);
        println!("{:?}", chunk.sample(index_path));
    }
}
