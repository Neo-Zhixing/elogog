use super::index_path::IndexPath;
use super::voxel::Voxel;
use super::chunk::Chunk;
use super::arena::ArenaNodeIndice;
use super::direction::Direction;
use super::bounds::Bounds;

#[derive(Clone)]
pub struct Node {
    pub index_path: IndexPath,
    pub voxel: Voxel,

    pub(super) parent_node: Option<ArenaNodeIndice>,
    pub(super) arena_node: Option<ArenaNodeIndice>, // Is null when it's a leaf node
}

impl Node {
    pub fn bounds(&self) -> Bounds {
        self.index_path.into()
    }

    #[inline]
    pub fn is_leaf(&self) -> bool {
        self.arena_node.is_none()
    }

    #[inline]
    pub fn is_subdivided(&self) -> bool {
        self.arena_node.is_some()
    }

    pub fn child(&self, dir: Direction, chunk: &Chunk) -> Node {
        if let Some(arena_node) = self.arena_node {
            let node = chunk.arena.get_node(arena_node);
            let mut result = Node {
                index_path: self.index_path.put(dir),
                voxel: node.data[dir],
                parent_node: Some(arena_node),
                arena_node: None,
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