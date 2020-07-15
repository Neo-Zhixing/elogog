use super::voxel::Voxel;
use super::arena::{Arena, ArenaNodeIndice, ArenaNode};
use super::index_path::IndexPath;
use std::ops::{Index, IndexMut};
use crate::util::tuple_strip::IterUtil;
use amethyst::ecs::{Component, DenseVecStorage};

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
                for ((current_index, _), (parent_index, dir)) in node_index_stack.iter().rev().tuple_strip() {
                    let current_node = self.arena.get_node(*current_index);
                    if !current_node.is_condensable() {
                        return;
                    }
                    let voxel = current_node.data[0];
                    let dir = *dir;
                    let parent_node = self.arena.get_node_mut(*parent_index);
                    parent_node.set_on_dir(dir, voxel);
                    debug_assert!(parent_node.has_child_on_dir(dir));
                    let old_mask = parent_node.leaf_mask;
                    let new_mask = old_mask & !(1 << dir);
                    self.arena.realloc(*parent_index, new_mask);
                }
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


struct ChunkVoxelIterator<'a> {
    chunk: &'a Chunk,
    stack: Vec<(u8, ArenaNodeIndice)>,
    dir: u8, // Next voxel to emit
}

impl Chunk {
    fn iter_leaf(&self) -> ChunkVoxelIterator {
        ChunkVoxelIterator {
            chunk: &self,
            stack: vec![(0, self.root_node)],
            dir: 0
        }
    }
}
impl<'a> Iterator for ChunkVoxelIterator<'a> {
    type Item = (Voxel);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(tuple) = self.stack.last() {
                let (fromdir, indice) = *tuple;
                if self.dir >= 8 {
                    // Pop from stack
                    self.stack.pop();
                    self.dir = fromdir + 1;
                    continue;
                }
                let node = self.chunk.arena.get_node(indice);
                if let Some(subnode) = node.child_on_dir(self.dir) {
                    // Has a child on that dir, needs to go deeper
                    self.stack.push((self.dir, subnode));
                    self.dir = 0;
                    continue;
                } else {
                    let dir = self.dir;
                    self.dir += 1;
                    return Some(node.data[dir as usize]);
                }
            } else {
                return None;
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let num_nodes = self.chunk.arena.count_nodes();
        (num_nodes, Some(num_nodes * 8))
    }
}


#[cfg(test)]
mod tests {
    use super::Chunk;
    use crate::octree::index_path::IndexPath;
    use crate::octree::voxel::Voxel;
    use std::mem::size_of;
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

    #[test]
    fn test_condense_on_set() {
        let mut chunk = Chunk::new();
        assert_eq!(chunk.arena.count_nodes(), 1);
        for i in 0..8 {
            let index = IndexPath::new(i).push(1);
            chunk.set(index, Voxel { data: 13 });
            assert_eq!(chunk.sample(IndexPath::new(i).push(1)), Voxel { data: 13 });
            assert_eq!(chunk.arena.count_nodes(), if i == 7 { 1 } else { 2 });
        }

        // Test multiple levels
        let mut chunk = Chunk::new();
        assert_eq!(chunk.arena.count_nodes(), 1);
        for i in 0..7 {
            let index = IndexPath::new(i).push(1);
            chunk.set(index, Voxel { data: 13 });
            assert_eq!(chunk.sample(IndexPath::new(i).push(1)), Voxel { data: 13 });
            assert_eq!(chunk.arena.count_nodes(), 2);
        }
        for i in 0..7 {
            let index = IndexPath::new(i).push(7).push(1);
            chunk.set(index, Voxel { data: 13 });
            assert_eq!(chunk.sample(index), Voxel { data: 13 });
            assert_eq!(chunk.arena.count_nodes(), 3);
        }
        // Adding this node should cause everything to collapse back to 0
        let index = IndexPath::new(7).push(7).push(1);
        chunk.set(index, Voxel { data: 13 });
        assert_eq!(chunk.sample(index), Voxel { data: 13 });
        assert_eq!(chunk.arena.count_nodes(), 1);
    }

    #[test]
    fn test_chunk_leaf_iterator() {
        let mut chunk = Chunk::new();
        for i in 0..7 {
            chunk.set(IndexPath::new(i), Voxel::new(i as u16));
        }
        for i in 0..7 {
            chunk.set(IndexPath::new(i).push(7), Voxel::new(i as u16 + 16));
        }

        for i in 0..8 {
            chunk.set(IndexPath::new(i).push(7).push(7), Voxel::new(i as u16 + 32));
        }

        let mut iter = chunk.iter_leaf();
        for (i, voxel) in iter.enumerate() {
            if i < 7 {
                assert_eq!(voxel.data, i as u16);
            } else if i < 14 {
                assert_eq!(voxel.data, i as u16 + 9);
            } else {
                assert_eq!(voxel.data, i as u16 + 18);
            }
        }
    }
}

impl Component for Chunk {
    type Storage = DenseVecStorage<Self>;
}
