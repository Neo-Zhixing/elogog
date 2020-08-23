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
    fn new(chunk: &'a Chunk, size: f32) -> Self;
    fn gen_wireframe(&self) -> DebugLinesComponent;
    fn into_mesh_builder(self) -> MeshBuilder<'static>;
}