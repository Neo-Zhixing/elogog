use crate::octree::{Bounds, Voxel, Chunk, Direction, Node};

struct WorldGenerator {
}

impl WorldGenerator {
    fn gen(&self) -> Chunk {
        let mut chunk = Chunk::new(8.0);
        let root = chunk.root();
        self.gen_bounds(root, &mut chunk);
        chunk
    }

    fn gen_bounds(&self, node: Node, chunk: &mut Chunk) {
        for dir in Direction::all().iter() { // Iterate over all 8 children
            let child = node.child(*dir, &chunk);
            if let Some(voxel) = self.get(child.bounds) {

            } else {
                // Needs to split
            }
        }
    }

    fn get(&self, bounds: Bounds) -> Option<Voxel> {
        None
    }
}
