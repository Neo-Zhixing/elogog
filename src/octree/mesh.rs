use super::chunk::Chunk;
use super::node::Node;

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
use crate::octree::direction::{Direction, DirectionMapper};

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

    let mut mesh_generator = MeshGenerator::new(chunk);
    mesh_generator.create_dualgrid();

    for cell in mesh_generator.dual_cells {
        let origin = cell[Direction::RearRightTop].bounds().center();
        debug_lines_component.add_line(
            origin,
            cell[Direction::FrontRightTop].bounds().center(),
            Srgba::new(1.0, 0.2, 1.0, 1.8),
        );
        debug_lines_component.add_line(
            origin,
            cell[Direction::RearRightBottom].bounds().center(),
            Srgba::new(1.0, 0.2, 1.0, 1.8),
        );
        debug_lines_component.add_line(
            origin,
            cell[Direction::RearLeftTop].bounds().center(),
            Srgba::new(1.0, 0.2, 1.0, 1.8),
        );
    }

    debug_lines_component
}



struct MeshGenerator<'a> {
    chunk: &'a Chunk,
    dual_cells: Vec<DirectionMapper<Node>>
}

impl<'a> MeshGenerator<'a> {
    pub fn new(chunk: &'a Chunk) -> Self {
        Self {
            chunk,
            dual_cells: Vec::new(),
        }
    }
    pub fn create_dualgrid(&mut self) {
        let root = self.chunk.root();
        self.node_proc(&root);
    }
    fn node_proc(&mut self, node: &Node) {
        if node.is_leaf() {
            return;
        }

        let children = Direction::map(|dir| node.child(dir, self.chunk));

        for child in children.iter() {
            self.node_proc(child);
        }

        for (dir1, dir2) in [
            (Direction::RearLeftBottom, Direction::FrontLeftBottom),
            (Direction::RearRightBottom, Direction::FrontRightBottom),
            (Direction::RearLeftTop, Direction::FrontRightTop),
            (Direction::RearRightTop, Direction::FrontRightTop),
        ].iter() {
            self.face_proc_xy([
                &children[*dir1],
                &children[*dir2]
            ]);
        }

        for (dir1, dir2) in [
            (Direction::RearLeftBottom, Direction::RearRightBottom),
            (Direction::FrontLeftBottom, Direction::FrontRightBottom),
            (Direction::RearLeftTop, Direction::RearRightTop),
            (Direction::FrontLeftTop, Direction::FrontRightTop),
        ].iter() {
            self.face_proc_zy([
                &children[*dir1],
                &children[*dir2]
            ]);
        }

        for (dir1, dir2) in [
            (Direction::RearLeftTop, Direction::RearLeftBottom),
            (Direction::RearRightTop, Direction::RearRightBottom),
            (Direction::FrontLeftTop, Direction::FrontLeftBottom),
            (Direction::FrontRightTop, Direction::FrontRightBottom),
        ].iter() {
            self.face_proc_xz([
                &children[*dir1],
                &children[*dir2]
            ]);
        }

        self.edge_proc_x([
            &children[Direction::RearLeftBottom],
            &children[Direction::FrontLeftBottom],
            &children[Direction::FrontLeftTop],
            &children[Direction::RearLeftTop],
        ]);
        self.edge_proc_x([
            &children[Direction::RearRightBottom],
            &children[Direction::FrontRightBottom],
            &children[Direction::FrontRightTop],
            &children[Direction::RearRightTop],
        ]);
        self.edge_proc_y([
            &children[Direction::RearLeftBottom],
            &children[Direction::RearRightBottom],
            &children[Direction::FrontRightBottom],
            &children[Direction::FrontLeftBottom],
        ]);
        self.edge_proc_y([
            &children[Direction::RearLeftTop],
            &children[Direction::RearRightTop],
            &children[Direction::FrontRightTop],
            &children[Direction::FrontLeftTop],
        ]);
        self.edge_proc_z([
            &children[Direction::FrontLeftTop],
            &children[Direction::FrontRightTop],
            &children[Direction::FrontRightBottom],
            &children[Direction::FrontLeftBottom],
        ]);
        self.edge_proc_z([
            &children[Direction::RearLeftTop],
            &children[Direction::RearRightTop],
            &children[Direction::RearRightBottom],
            &children[Direction::RearLeftBottom],
        ]);

        self.vert_proc(children.data);
    }
    fn face_proc_xy(&mut self, nodes: [&Node; 2]) {
        if nodes.iter().all(|n| n.is_leaf()) {
            return;
        }
        let children = DirectionMapper::new([
            nodes[1].child(Direction::RearLeftBottom, self.chunk),
            nodes[1].child(Direction::RearRightBottom, self.chunk),
            nodes[0].child(Direction::FrontLeftBottom, self.chunk),
            nodes[0].child(Direction::FrontRightBottom, self.chunk),
            nodes[1].child(Direction::RearLeftTop, self.chunk),
            nodes[1].child(Direction::RearRightTop, self.chunk),
            nodes[0].child(Direction::FrontLeftTop, self.chunk),
            nodes[0].child(Direction::FrontRightTop, self.chunk),
        ]);

        for (dir1, dir2) in [
            (Direction::RearLeftBottom, Direction::FrontLeftBottom),
            (Direction::RearRightBottom, Direction::FrontRightBottom),
            (Direction::RearLeftTop, Direction::FrontRightTop),
            (Direction::RearRightTop, Direction::FrontRightTop),
        ].iter() {
            self.face_proc_xy([
                &children[*dir1],
                &children[*dir2]
            ]);
        }

        self.edge_proc_x([
            &children[Direction::RearLeftBottom],
            &children[Direction::FrontLeftBottom],
            &children[Direction::FrontLeftTop],
            &children[Direction::RearLeftTop],
        ]);
        self.edge_proc_x([
            &children[Direction::RearRightBottom],
            &children[Direction::FrontRightBottom],
            &children[Direction::FrontRightTop],
            &children[Direction::RearRightTop],
        ]);
        self.edge_proc_y([
            &children[Direction::RearLeftBottom],
            &children[Direction::RearRightBottom],
            &children[Direction::FrontRightBottom],
            &children[Direction::FrontLeftBottom],
        ]);
        self.edge_proc_y([
            &children[Direction::RearLeftTop],
            &children[Direction::RearRightTop],
            &children[Direction::FrontRightTop],
            &children[Direction::FrontLeftTop],
        ]);
        self.vert_proc(children.data);
    }
    fn face_proc_zy(&mut self, nodes: [&Node; 2]) {
        if nodes.iter().all(|n| n.is_leaf()) {
            return;
        }
        let children = DirectionMapper::new([
            nodes[0].child(Direction::FrontRightBottom, self.chunk),
            nodes[1].child(Direction::FrontLeftBottom, self.chunk),
            nodes[0].child(Direction::RearRightBottom, self.chunk),
            nodes[1].child(Direction::RearLeftBottom, self.chunk),
            nodes[0].child(Direction::FrontRightTop, self.chunk),
            nodes[1].child(Direction::FrontLeftTop, self.chunk),
            nodes[0].child(Direction::RearRightTop, self.chunk),
            nodes[1].child(Direction::RearLeftTop, self.chunk),
        ]);

        for (dir1, dir2) in [
            (Direction::RearLeftBottom, Direction::RearRightBottom),
            (Direction::FrontLeftBottom, Direction::FrontRightBottom),
            (Direction::RearLeftTop, Direction::RearRightTop),
            (Direction::FrontLeftTop, Direction::FrontRightTop),
        ].iter() {
            self.face_proc_zy([
                &children[*dir1],
                &children[*dir2]
            ]);
        }
        self.edge_proc_z([
            &children[Direction::FrontLeftTop],
            &children[Direction::FrontRightTop],
            &children[Direction::FrontRightBottom],
            &children[Direction::FrontLeftBottom],
        ]);
        self.edge_proc_z([
            &children[Direction::RearLeftTop],
            &children[Direction::RearRightTop],
            &children[Direction::RearRightBottom],
            &children[Direction::RearLeftBottom],
        ]);
        self.edge_proc_y([
            &children[Direction::RearLeftBottom],
            &children[Direction::RearRightBottom],
            &children[Direction::FrontRightBottom],
            &children[Direction::FrontLeftBottom],
        ]);
        self.edge_proc_y([
            &children[Direction::RearLeftTop],
            &children[Direction::RearRightTop],
            &children[Direction::FrontRightTop],
            &children[Direction::FrontLeftTop],
        ]);
        self.vert_proc(children.data);
    }
    fn face_proc_xz(&mut self, nodes: [&Node; 2]) {
        if nodes.iter().all(|n| n.is_leaf()) {
            return;
        }
        let children = DirectionMapper::new([
            nodes[1].child(Direction::FrontLeftTop, self.chunk),
            nodes[1].child(Direction::FrontRightTop, self.chunk),
            nodes[1].child(Direction::RearLeftTop, self.chunk),
            nodes[1].child(Direction::RearRightTop, self.chunk),
            nodes[0].child(Direction::FrontLeftBottom, self.chunk),
            nodes[0].child(Direction::FrontRightBottom, self.chunk),
            nodes[0].child(Direction::RearLeftBottom, self.chunk),
            nodes[0].child(Direction::RearRightBottom, self.chunk),
        ]);

        for (dir1, dir2) in [
            (Direction::RearLeftTop, Direction::RearLeftBottom),
            (Direction::RearRightTop, Direction::RearRightBottom),
            (Direction::FrontLeftTop, Direction::FrontLeftBottom),
            (Direction::FrontRightTop, Direction::FrontRightBottom),
        ].iter() {
            self.face_proc_xz([
                &children[*dir1],
                &children[*dir2]
            ]);
        }
        self.edge_proc_x([
            &children[Direction::RearLeftBottom],
            &children[Direction::FrontLeftBottom],
            &children[Direction::FrontLeftTop],
            &children[Direction::RearLeftTop],
        ]);
        self.edge_proc_x([
            &children[Direction::RearRightBottom],
            &children[Direction::FrontRightBottom],
            &children[Direction::FrontRightTop],
            &children[Direction::RearRightTop],
        ]);
        self.edge_proc_z([
            &children[Direction::FrontLeftTop],
            &children[Direction::FrontRightTop],
            &children[Direction::FrontRightBottom],
            &children[Direction::FrontLeftBottom],
        ]);
        self.edge_proc_z([
            &children[Direction::RearLeftTop],
            &children[Direction::RearRightTop],
            &children[Direction::RearRightBottom],
            &children[Direction::RearLeftBottom],
        ]);
        self.vert_proc(children.data);
    }
    fn edge_proc_x(&mut self, nodes: [&Node; 4]) {
        if nodes.iter().all(|n| n.is_leaf()) {
            return;
        }
        let children = DirectionMapper::new([
            nodes[1].child(Direction::RearLeftTop, self.chunk),
            nodes[1].child(Direction::RearRightTop, self.chunk),
            nodes[0].child(Direction::FrontLeftTop, self.chunk),
            nodes[0].child(Direction::FrontRightTop, self.chunk),
            nodes[2].child(Direction::RearLeftBottom, self.chunk),
            nodes[2].child(Direction::RearRightBottom, self.chunk),
            nodes[3].child(Direction::FrontLeftBottom, self.chunk),
            nodes[3].child(Direction::FrontRightBottom, self.chunk),
        ]);
        self.edge_proc_x([
            &children[Direction::RearLeftBottom],
            &children[Direction::FrontLeftBottom],
            &children[Direction::FrontLeftTop],
            &children[Direction::RearLeftTop],
        ]);
        self.edge_proc_x([
            &children[Direction::RearRightBottom],
            &children[Direction::FrontRightBottom],
            &children[Direction::FrontRightTop],
            &children[Direction::RearRightTop],
        ]);
        self.vert_proc(children.data);
    }
    fn edge_proc_y(&mut self, nodes: [&Node; 4]) {
        if nodes.iter().all(|n| n.is_leaf()) {
            return;
        }
        let children = DirectionMapper::new([
            nodes[3].child(Direction::RearRightBottom, self.chunk),
            nodes[2].child(Direction::RearLeftBottom, self.chunk),
            nodes[0].child(Direction::FrontRightBottom, self.chunk),
            nodes[1].child(Direction::FrontLeftBottom, self.chunk),
            nodes[3].child(Direction::RearRightTop, self.chunk),
            nodes[2].child(Direction::RearLeftTop, self.chunk),
            nodes[0].child(Direction::FrontRightTop, self.chunk),
            nodes[1].child(Direction::FrontLeftTop, self.chunk),
        ]);
        self.edge_proc_y([
            &children[Direction::RearLeftBottom],
            &children[Direction::RearRightBottom],
            &children[Direction::FrontRightBottom],
            &children[Direction::FrontLeftBottom],
        ]);
        self.edge_proc_y([
            &children[Direction::RearLeftTop],
            &children[Direction::RearRightTop],
            &children[Direction::FrontRightTop],
            &children[Direction::FrontLeftTop],
        ]);
        self.vert_proc(children.data);
    }
    fn edge_proc_z(&mut self, nodes: [&Node; 4]) {
        if nodes.iter().all(|n| n.is_leaf()) {
            return;
        }
        let children = DirectionMapper::new([
            nodes[3].child(Direction::FrontRightTop, self.chunk),
            nodes[2].child(Direction::FrontLeftTop, self.chunk),
            nodes[3].child(Direction::RearRightTop, self.chunk),
            nodes[2].child(Direction::RearLeftTop, self.chunk),
            nodes[0].child(Direction::FrontRightBottom, self.chunk),
            nodes[1].child(Direction::FrontLeftBottom, self.chunk),
            nodes[0].child(Direction::RearRightBottom, self.chunk),
            nodes[1].child(Direction::RearLeftBottom, self.chunk),
        ]);
        self.edge_proc_z([
            &children[Direction::FrontLeftTop],
            &children[Direction::FrontRightTop],
            &children[Direction::FrontRightBottom],
            &children[Direction::FrontLeftBottom],
        ]);
        self.edge_proc_z([
            &children[Direction::RearLeftTop],
            &children[Direction::RearRightTop],
            &children[Direction::RearRightBottom],
            &children[Direction::RearLeftBottom],
        ]);

        self.vert_proc(children.data);
    }
    fn vert_proc(&mut self, mut nodes: [Node; 8]) {
        loop {
            let mut has_subdivided = false;
            for (index, node) in nodes.iter_mut().enumerate() {
                if node.is_subdivided() {
                    has_subdivided = true;
                    let dir = Direction::from(index as u8);
                    let opposite_dir_node = node.child(dir.opposite(), self.chunk);
                    *node = opposite_dir_node;
                }
            }
            if !has_subdivided {
                break;
            }
        }
        // Now all nodes are leaf node
        self.dual_cells.push(DirectionMapper::new(nodes));
        // TODO add dual cell
    }
}
