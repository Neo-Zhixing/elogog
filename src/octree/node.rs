use super::voxel::Voxel;
use super::chunk::Chunk;

// sizeof(Node) = 160bits = 20 bytes
pub struct Node {
    pub(super) children_index: u16, // meaningless when all leaf
    pub(super) leaf_mask: u8, // 1 for child, 0 for leaf
    load_mask: u8,
    pub(super) data: [Voxel; 8],
}

impl Node {
    pub fn num_leafs(&self) -> u8 {
        self.leaf_mask.count_zeros() as u8
    }
    pub fn num_child_nodes(&self) -> u8 {
        self.leaf_mask.count_ones() as u8
    }
    pub fn has_child_on(&self, dir: u8) -> bool {
        self.leaf_mask & (1 << dir) != 0
    }
    pub fn is_leaf(&self) -> bool {
        return self.leaf_mask == 0 // all leaf
    }
    pub(super) fn get_child_node_index_on(&self, dir: u8) -> u8 {
        assert!(self.has_child_on(dir));
        let mask = std::u8::MAX << dir;
        let ones_pop = self.leaf_mask & !(mask); // bit clear
        ones_pop.count_ones() as u8
    }
}

impl std::default::Default for Node {
    fn default() -> Self {
        Node {
            leaf_mask: 0, // All leaf
            load_mask: 0,
            data: Default::default(),
            children_index: 0, // Meaningless because all leaf
        }
    }
}