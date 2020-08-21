use crate::octree::Chunk;
use amethyst::{
    renderer::{
        debug_drawing::{DebugLinesComponent},
        rendy::mesh::{
            MeshBuilder,
        }
    },
};


pub mod dualmc;

pub trait Mesher<'a> {
    fn new(chunk: &'a Chunk) -> Self;
    fn gen_wireframe(&self) -> DebugLinesComponent;
    fn into_mesh_builder(self) -> MeshBuilder<'static>;
}