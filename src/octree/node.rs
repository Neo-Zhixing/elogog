use super::index_path::IndexPath;
use super::voxel::Voxel;
use super::chunk::Chunk;
use super::arena::ArenaNodeIndice;
use super::direction::Direction;
use super::bounds::Bounds;

#[derive(Clone)]
pub struct Node {
    pub index_path: IndexPath,
    pub bounds: Bounds,
    pub voxel: Voxel,

    pub(super) parent_node: Option<ArenaNodeIndice>,
    pub(super) arena_node: Option<ArenaNodeIndice>, // Is null when it's a leaf node
}

impl Node {
    #[inline]
    pub fn is_leaf(&self) -> bool {
        self.arena_node.is_none()
    }

    #[inline]
    pub fn is_subdivided(&self) -> bool {
        self.arena_node.is_some()
    }

    pub fn set(&mut self, voxel: Voxel, chunk: &mut Chunk) {
        self.voxel = voxel;
        let dir = self.index_path.get();

        if let Some(parent_node) = self.parent_node {
            chunk.arena.get_node(parent_node).set_on_dir(dir, voxel);
        } else {
            // TODO: when calling set() on the root node
        }

        if let Some(arena_node) = self.arena_node {
            // This node is not a leaf node.
            // Since we're setting on this entire node, we can free all child nodes
            // And set the parent nodes value to the corresponding value
        }
    }

    pub fn del(&mut self, chunk: &mut Chunk) {
        if let Some(parent) = self.parent_node {

        }
    }

    // Delete the child on the specified octant
    pub fn del_child(&mut self, octant: Direction, chunk: &mut Chunk) {
        if let Some(arena_node) = self.arena_node {
            // Only do the deletion when we're not a leaf node
            let mut child = self.child(octant, chunk);
            for dir in Direction::all().iter() {
                child.del_child(*dir, chunk);
            }

            chunk.arena.realloc_del(arena_node, dir);
        }
    }

    pub fn child(&self, dir: Direction, chunk: &Chunk) -> Node {
        if let Some(arena_node) = self.arena_node {
            let node = chunk.arena.get_node(arena_node);
            let mut result = Node {
                index_path: self.index_path.put(dir),
                voxel: node.data[dir],
                parent_node: Some(arena_node),
                arena_node: None,
                bounds: self.bounds.half(dir),
            };
            if let Some(child_node) = node.child_on_dir(dir) {
                // The node is an actual node
                result.arena_node = Some(child_node);
                result
            } else {
                // The node is an leaf node
                result
            }
        } else {
            self.clone()
        }
    }
}