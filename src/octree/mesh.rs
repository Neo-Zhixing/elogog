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

pub fn gen(chunk: &Chunk) -> MeshBuilder<'static> {
  let bd = MeshBuilder::new();
    bd
        .with_vertices(vec![
          Position([0.0, 0.0, 0.0]),
          Position([0.0, 1.0, 0.0]),
          Position([0.0, 0.0, 1.0]),
        ])
        .with_vertices(vec![
          Normal([0.0, 0.0, 0.0]),
          Normal([0.0, 0.0, 0.0]),
          Normal([0.0, 0.0, 0.0]),
        ])
        .with_vertices(vec![
            TexCoord([0.0, 0.0]),
            TexCoord([0.0, 0.0]),
            TexCoord([0.0, 0.0]),
        ])
        .with_indices(Indices::U16(vec![0, 1, 2].into()))
}

pub fn gen_wireframe(chunk: &Chunk) -> DebugLinesComponent {
    let mut debug_lines_component = DebugLinesComponent::with_capacity(100);
    debug_lines_component.add_direction(
        [0.0, 0.0001, 0.0].into(),
        [0.2, 0.0, 0.0].into(),
        Srgba::new(1.0, 0.0, 0.23, 1.0),
    );
    debug_lines_component.add_direction(
        [0.0, 0.0, 0.0].into(),
        [0.0, 0.2, 0.0].into(),
        Srgba::new(0.5, 0.85, 0.1, 1.0),
    );
    debug_lines_component.add_direction(
        [0.0, 0.0001, 0.0].into(),
        [0.0, 0.0, 0.2].into(),
        Srgba::new(0.2, 0.75, 0.93, 1.0),
    );
    debug_lines_component
}
