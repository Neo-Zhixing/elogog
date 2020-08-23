#![feature(alloc_layout_extra)]
#![feature(const_generics)]

mod util;
mod octree;

use amethyst::{
    controls::{FlyControlBundle, FlyControlTag},
    core::{
        math::{Point3, Vector3},
        transform::{Transform, TransformBundle},
        Time,
        frame_limiter::FrameRateLimitStrategy
    },
    derive::SystemDesc,
    ecs::{Read, System, SystemData, WorldExt, Write},
    input::{is_close_requested, is_key_down, InputBundle, StringBindings},
    prelude::*,
    renderer::{
        camera::{Camera},
        light,
        debug_drawing::{DebugLinesComponent},
        palette::{Srgb, Srgba},
        palette::LinSrgba,
        plugins::{RenderDebugLines, RenderSkybox, RenderToWindow, RenderFlat3D, RenderPbr3D, RenderShaded3D},
        types::DefaultBackend,
        RenderingBundle,
        shape::Shape,
        Mesh,
        Texture,
        mtl::{Material, MaterialDefaults},

        rendy::{
            mesh::{Normal, Position, Tangent, TexCoord, MeshBuilder},
            texture::palette::load_from_linear_rgba,
        }
    },
    utils::application_root_dir,
    winit::VirtualKeyCode,
    assets::{AssetLoaderSystemData}
};
use std::time::Duration;
use crate::util::gridline::get_gridline_component;
use crate::octree::VoxelData;
use crate::octree::direction::Direction;
use crate::octree::mesher::Mesher;

struct GameState;

impl SimpleState for GameState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        // Setup debug lines as a component and add lines to render axis&grid
        data.world
            .create_entity()
            .with(get_gridline_component())
            .build();

        // Setup camera
        let mut local_transform = Transform::default();
        local_transform.set_translation_xyz(0.0, 0.5, 2.0);
        data.world
            .create_entity()
            .with(FlyControlTag)
            .with(Camera::perspective(1.33333, 1.0, 0.01))
            .with(local_transform)
            .build();

        let generator: octree::world_builder::WorldBuilder<octree::VoxelData, _> = octree::world_builder::WorldBuilder::new(
            |chunk: &octree::world::ChunkCoordinates, bounds: &octree::bounds::Bounds| {
                let target_bounds = octree::bounds::Bounds::from_discrete_grid((32, 32, 32), 48, 128);

                let intersects = target_bounds.intersects(bounds);
                println!("{:?} {:?} {:?}", target_bounds, intersects, bounds);
                match target_bounds.intersects(bounds) {
                    octree::bounds::BoundsSpacialRelationship::Disjoint => octree::world_builder::Isosurface::Uniform(VoxelData::EMPTY),
                    octree::bounds::BoundsSpacialRelationship::Contain => octree::world_builder::Isosurface::Uniform(1.into()),
                    octree::bounds::BoundsSpacialRelationship::Intersect => octree::world_builder::Isosurface::Surface,
                }
            }
        );
        let chunk = generator.build(& octree::world::ChunkCoordinates::new());

        let mut mesh_generator = crate::octree::mesher::dualmc::MeshGenerator::new(&chunk, 1.0);
        let wireframe = mesh_generator.gen_wireframe();


        // Getting us a ball
        let mesh = data.world
            .exec(|loader: AssetLoaderSystemData<'_, Mesh>| {
                loader.load_from_data(
                    mesh_generator.into_mesh_builder().into(),
                    (),
                )
            });
        let albedo = data.world
            .exec(|loader: AssetLoaderSystemData<'_, Texture>| {
                let albedo = loader.load_from_data(
                    load_from_linear_rgba(LinSrgba::new(1.0, 1.0, 1.0, 0.5))
                        .into(),
                    (),
                );
                albedo
            });
        let mat_defaults = data.world.read_resource::<MaterialDefaults>().0.clone();
        let material = data.world.exec(
            |loader: AssetLoaderSystemData<'_, Material>| {
                loader.load_from_data(
                    Material {
                        albedo,
                        ..mat_defaults.clone()
                    },
                    (),
                )
            });
        let mut pos = Transform::default();
        pos.set_translation_xyz(0.0, 0.0, 0.0);
        data.world
            .create_entity()
            .with(pos)
            .with(mesh)
            .with(material)
            .with(wireframe)
            .build();
        // Creating light source
        let light: light::Light = light::DirectionalLight {
            color: Srgb::new(0.8, 0.0, 0.0),
            ..light::DirectionalLight::default()
        }
        .into();
        let light_pos = Transform::default();
        data.world.create_entity()
            .with(light)
            .with(light_pos)
            .build();
    }

    fn handle_event(
        &mut self,
        _: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        if let StateEvent::Window(event) = event {
            if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                Trans::Quit
            } else {
                Trans::None
            }
        } else {
            Trans::None
        }
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let display_config_path = app_root.join("config/display.ron");
    let key_bindings_path = app_root.join("config/input.ron");
    let assets_dir = app_root.join("assets");

    let fly_control_bundle = FlyControlBundle::<StringBindings>::new(
        Some(String::from("move_x")),
        Some(String::from("move_y")),
        Some(String::from("move_z")),
    )
        .with_sensitivity(0.1, 0.1);

    let game_data = GameDataBuilder::default()
        .with_bundle(
            InputBundle::<StringBindings>::new().with_bindings_from_file(&key_bindings_path)?,
        )?
        .with_bundle(fly_control_bundle)?
        .with_bundle(TransformBundle::new().with_dep(&["fly_movement"]))?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(RenderToWindow::from_config_path(display_config_path)?)
                .with_plugin(RenderDebugLines::default())
                .with_plugin(RenderSkybox::default())
                .with_plugin(RenderShaded3D::default()),
        )?;

    let mut game = Application::build(assets_dir, GameState)?
        .with_frame_limit(FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)), 60)
        .build(game_data)?;
    game.run();
    Ok(())
}
