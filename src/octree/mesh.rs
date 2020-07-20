use super::chunk::Chunk;
use amethyst::{
    controls::{FlyControlBundle, FlyControlTag},
    core::{
        math::{Point3, Vector3},
        transform::{Transform, TransformBundle},
        Time,
    },
    derive::SystemDesc,
    ecs::{Read, System, SystemData, WorldExt, Write},
    input::{is_close_requested, is_key_down, InputBundle, StringBindings},
    prelude::*,
    renderer::{
        camera::{Camera, Projection},
        debug_drawing::{DebugLines, DebugLinesComponent, DebugLinesParams},
        palette::Srgba,
        plugins::{RenderDebugLines, RenderSkybox, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
        rendy::mesh::{
            MeshBuilder,
            Position,
            Normal,
            Tangent,
            TexCoord,
            Indices,
        },

    },
    utils::application_root_dir,
    winit::VirtualKeyCode,
};
use crate::octree::bounds::Bounds;
use crate::octree::direction::Direction;

pub fn gen_wireframe(chunk: &Chunk) -> DebugLinesComponent {
    let mut debug_lines_component = DebugLinesComponent::with_capacity(100);
    for node in chunk.iter_leaf() {
        let bounds: Bounds = node.bounds();
        let position = bounds.get_position();
        let width = bounds.get_width();

        for i in 0..3 {
            let mut dir: [f32; 3] = [0.0, 0.0, 0.0];
            dir[i] = width;
            debug_lines_component.add_direction(
                position,
                dir.into(),
                Srgba::new(1.0, 0.5, 0.23, 1.0),
            );
        }
    }
    debug_lines_component
}
