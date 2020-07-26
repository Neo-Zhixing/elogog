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

trait Dimension {
    type FaceEdges1: Dimension;
    type FaceEdges2: Dimension;
    const EDGE_PROC_DIR_GROUPS: [[Direction; 4]; 2];
    const EDGE_PROC_DIR_TUPLES: [(usize, Direction); 8];
    const FACE_PROC_DIR_GROUPS: [(Direction, Direction); 4];
    const FACE_PROC_DIR_TUPLES: [(usize, Direction); 8];
}

struct X;
struct Y;
struct Z;

impl Dimension for X {
    type FaceEdges1 = X;
    type FaceEdges2 = Y;
    const EDGE_PROC_DIR_GROUPS: [[Direction; 4]; 2] = [
        [Direction::RearLeftBottom, Direction::FrontLeftBottom, Direction::FrontLeftTop, Direction::RearLeftTop],
        [Direction::RearRightBottom, Direction::FrontRightBottom, Direction::FrontRightTop, Direction::RearRightTop],
    ];
    const EDGE_PROC_DIR_TUPLES: [(usize, Direction); 8] = [
        (1, Direction::RearLeftTop),
        (1, Direction::RearRightTop),
        (0, Direction::FrontLeftTop),
        (0, Direction::FrontRightTop),
        (2, Direction::RearLeftBottom),
        (2, Direction::RearRightBottom),
        (3, Direction::FrontLeftBottom),
        (3, Direction::FrontRightBottom),
    ];
    const FACE_PROC_DIR_GROUPS: [(Direction, Direction); 4] = [
        (Direction::RearLeftBottom, Direction::FrontLeftBottom),
        (Direction::RearRightBottom, Direction::FrontRightBottom),
        (Direction::RearLeftTop, Direction::FrontRightTop),
        (Direction::RearRightTop, Direction::FrontRightTop),
    ];
    const FACE_PROC_DIR_TUPLES: [(usize, Direction); 8] = [
        (1, Direction::RearLeftBottom),
        (1, Direction::RearRightBottom),
        (0, Direction::FrontLeftBottom),
        (0, Direction::FrontRightBottom),
        (1, Direction::RearLeftTop),
        (1, Direction::RearRightTop),
        (0, Direction::FrontLeftTop),
        (0, Direction::FrontRightTop),
    ];
}

impl Dimension for Y {
    type FaceEdges1 = Z;
    type FaceEdges2 = Y;
    const EDGE_PROC_DIR_GROUPS: [[Direction; 4]; 2] = [
        [Direction::RearLeftBottom, Direction::RearRightBottom, Direction::FrontRightBottom, Direction::FrontLeftBottom],
        [Direction::RearLeftTop, Direction::RearRightTop, Direction::FrontRightTop, Direction::FrontLeftTop],
    ];
    const EDGE_PROC_DIR_TUPLES: [(usize, Direction); 8] = [
        (3, Direction::RearRightBottom),
        (2, Direction::RearLeftBottom),
        (0, Direction::FrontRightBottom),
        (1, Direction::FrontLeftBottom),
        (3, Direction::RearRightTop),
        (2, Direction::RearLeftTop),
        (0, Direction::FrontRightTop),
        (1, Direction::FrontLeftTop),
    ];

    const FACE_PROC_DIR_GROUPS: [(Direction, Direction); 4] = [
        (Direction::RearLeftBottom, Direction::RearRightBottom),
        (Direction::FrontLeftBottom, Direction::FrontRightBottom),
        (Direction::RearLeftTop, Direction::RearRightTop),
        (Direction::FrontLeftTop, Direction::FrontRightTop),
    ];

    const FACE_PROC_DIR_TUPLES: [(usize, Direction); 8] = [
        (0, Direction::FrontRightBottom),
        (1, Direction::FrontLeftBottom),
        (0, Direction::RearRightBottom),
        (1, Direction::RearLeftBottom),
        (0, Direction::FrontRightTop),
        (1, Direction::FrontLeftTop),
        (0, Direction::RearRightTop),
        (1, Direction::RearLeftTop),
    ];
}

impl Dimension for Z {
    type FaceEdges1 = X;
    type FaceEdges2 = Z;
    const EDGE_PROC_DIR_GROUPS: [[Direction; 4]; 2] = [
        [Direction::FrontLeftTop, Direction::FrontRightTop, Direction::FrontRightBottom, Direction::FrontLeftBottom],
        [Direction::RearLeftTop, Direction::RearRightTop, Direction::RearRightBottom, Direction::RearLeftBottom],
    ];
    const EDGE_PROC_DIR_TUPLES: [(usize, Direction); 8] = [
        (3, Direction::FrontRightTop),
        (2, Direction::FrontLeftTop),
        (3, Direction::RearRightTop),
        (2, Direction::RearLeftTop),
        (0, Direction::FrontRightBottom),
        (1, Direction::FrontLeftBottom),
        (0, Direction::RearRightBottom),
        (1, Direction::RearLeftBottom),
    ];

    const FACE_PROC_DIR_GROUPS: [(Direction, Direction); 4] =[
        (Direction::RearLeftTop, Direction::RearLeftBottom),
        (Direction::RearRightTop, Direction::RearRightBottom),
        (Direction::FrontLeftTop, Direction::FrontLeftBottom),
        (Direction::FrontRightTop, Direction::FrontRightBottom),
    ];

    const FACE_PROC_DIR_TUPLES: [(usize, Direction); 8] = [
        (1, Direction::FrontLeftTop),
        (1, Direction::FrontRightTop),
        (1, Direction::RearLeftTop),
        (1, Direction::RearRightTop),
        (0, Direction::FrontLeftBottom),
        (0, Direction::FrontRightBottom),
        (0, Direction::RearLeftBottom),
        (0, Direction::RearRightBottom),
    ];
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

        self.face_proc_children::<X>(&children);
        self.face_proc_children::<Y>(&children);
        self.face_proc_children::<Z>(&children);

        self.edge_proc_children::<X>(&children);
        self.edge_proc_children::<Y>(&children);
        self.edge_proc_children::<Z>(&children);

        self.vert_proc(children.data);
    }
    fn face_proc_children<T: Dimension>(&mut self, children: &DirectionMapper<Node>) {
        for (dir1, dir2) in T::FACE_PROC_DIR_GROUPS.iter() {
            self.face_proc::<T>([
                &children[*dir1],
                &children[*dir2]
            ]);
        }
    }
    fn face_proc<T: Dimension>(&mut self, nodes: [&Node; 2]) {
        if nodes.iter().all(|n| n.is_leaf()) {
            return;
        }
        let tuples = T::FACE_PROC_DIR_TUPLES;
        let children = DirectionMapper::new([
            nodes[tuples[0].0].child(tuples[0].1, self.chunk),
            nodes[tuples[1].0].child(tuples[1].1, self.chunk),
            nodes[tuples[2].0].child(tuples[2].1, self.chunk),
            nodes[tuples[3].0].child(tuples[3].1, self.chunk),
            nodes[tuples[4].0].child(tuples[4].1, self.chunk),
            nodes[tuples[5].0].child(tuples[5].1, self.chunk),
            nodes[tuples[6].0].child(tuples[6].1, self.chunk),
            nodes[tuples[7].0].child(tuples[7].1, self.chunk),
        ]);

        self.face_proc_children::<T>(&children);
        self.edge_proc_children::<T::FaceEdges1>(&children);
        self.edge_proc_children::<T::FaceEdges2>(&children);
        self.vert_proc(children.data);
    }
    fn edge_proc_children<T>(&mut self, children: &DirectionMapper<Node>)
        where T: Dimension {
        let dir_groups = T::EDGE_PROC_DIR_GROUPS;

        for group in dir_groups.iter() {
            self.edge_proc::<T>([
                &children[group[0]],
                &children[group[1]],
                &children[group[2]],
                &children[group[3]],
            ]);
        }
    }
    fn edge_proc<T>(&mut self, nodes: [&Node; 4])
        where T: Dimension {
        if nodes.iter().all(|n| n.is_leaf()) {
            return;
        }

        let t = T::EDGE_PROC_DIR_TUPLES;

        let children = DirectionMapper::new([
            nodes[t[0].0].child(t[0].1, self.chunk),
            nodes[t[1].0].child(t[1].1, self.chunk),
            nodes[t[2].0].child(t[2].1, self.chunk),
            nodes[t[3].0].child(t[3].1, self.chunk),
            nodes[t[4].0].child(t[4].1, self.chunk),
            nodes[t[5].0].child(t[5].1, self.chunk),
            nodes[t[6].0].child(t[6].1, self.chunk),
            nodes[t[7].0].child(t[7].1, self.chunk),
        ]);
        self.edge_proc_children::<T>(&children);
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
