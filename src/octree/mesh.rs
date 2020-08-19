use itertools::Itertools;

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
use crate::octree::direction::{Direction, DirectionMapper, Edge};
use crate::octree::{Chunk, Voxel, VoxelData};

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
        (Direction::RearLeftTop, Direction::FrontLeftTop),
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
/* Looking up the edge table returns a 12 bit number, each bit corresponding to an edge,
   0 if the edge isn't cut by the isosurface, 1 if the edge is cut by the isosurface.
   If none of the edges are cut the table returns a 0, this occurs when cubeindex is 0
   (all vertices below the isosurface) or 0xff (all vertices above the isosurface).
   */
const EDGE_TABLE: [[u16; 5]; 256] = [
    [0xffff, 0xffff, 0xffff, 0xffff, 0xffff],
    [0x02b3, 0xffff, 0xffff, 0xffff, 0xffff],
    [0x0a21, 0xffff, 0xffff, 0xffff, 0xffff],
    [0x01a3, 0x03ab, 0xffff, 0xffff, 0xffff],
    [0x0380, 0xffff, 0xffff, 0xffff, 0xffff],
    [0x02b0, 0x00b8, 0xffff, 0xffff, 0xffff],
    [0x0380, 0x0a21, 0xffff, 0xffff, 0xffff],
    [0x01a0, 0x0a80, 0x0ab8, 0xffff, 0xffff],
    [0x0910, 0xffff, 0xffff, 0xffff, 0xffff],
    [0x0091, 0x0b32, 0xffff, 0xffff, 0xffff],
    [0x0a29, 0x0920, 0xffff, 0xffff, 0xffff],
    [0x0093, 0x09b3, 0x09ab, 0xffff, 0xffff],
    [0x0381, 0x0189, 0xffff, 0xffff, 0xffff],
    [0x02b1, 0x0b91, 0x0b89, 0xffff, 0xffff],
    [0x0382, 0x08a2, 0x089a, 0xffff, 0xffff],
    [0x0a89, 0x0b8a, 0xffff, 0xffff, 0xffff],
    [0x0b67, 0xffff, 0xffff, 0xffff, 0xffff],
    [0x0327, 0x0726, 0xffff, 0xffff, 0xffff],
    [0x021a, 0x07b6, 0xffff, 0xffff, 0xffff],
    [0x067a, 0x071a, 0x0731, 0xffff, 0xffff],
    [0x0803, 0x067b, 0xffff, 0xffff, 0xffff],
    [0x0807, 0x0067, 0x0026, 0xffff, 0xffff],
    [0x0a21, 0x0803, 0x07b6, 0xffff, 0xffff],
    [0x067a, 0x0a71, 0x0781, 0x0801, 0xffff],
    [0x0910, 0x067b, 0xffff, 0xffff, 0xffff],
    [0x0672, 0x0732, 0x0910, 0xffff, 0xffff],
    [0x0092, 0x09a2, 0x07b6, 0xffff, 0xffff],
    [0x0730, 0x0a70, 0x09a0, 0x07a6, 0xffff],
    [0x0918, 0x0138, 0x067b, 0xffff, 0xffff],
    [0x0261, 0x0681, 0x0891, 0x0678, 0xffff],
    [0x07b6, 0x03a2, 0x038a, 0x089a, 0xffff],
    [0x0a67, 0x08a7, 0x09a8, 0xffff, 0xffff],
    [0x056a, 0xffff, 0xffff, 0xffff, 0xffff],
    [0x0b32, 0x056a, 0xffff, 0xffff, 0xffff],
    [0x0561, 0x0162, 0xffff, 0xffff, 0xffff],
    [0x0b36, 0x0356, 0x0315, 0xffff, 0xffff],
    [0x0380, 0x06a5, 0xffff, 0xffff, 0xffff],
    [0x080b, 0x002b, 0x056a, 0xffff, 0xffff],
    [0x0561, 0x0621, 0x0803, 0xffff, 0xffff],
    [0x0b80, 0x05b0, 0x0150, 0x06b5, 0xffff],
    [0x0109, 0x06a5, 0xffff, 0xffff, 0xffff],
    [0x0910, 0x0b32, 0x06a5, 0xffff, 0xffff],
    [0x0569, 0x0609, 0x0620, 0xffff, 0xffff],
    [0x06b3, 0x0630, 0x0560, 0x0950, 0xffff],
    [0x0381, 0x0891, 0x06a5, 0xffff, 0xffff],
    [0x06a5, 0x0291, 0x02b9, 0x0b89, 0xffff],
    [0x0895, 0x0285, 0x0625, 0x0823, 0xffff],
    [0x0956, 0x0b96, 0x089b, 0xffff, 0xffff],
    [0x0a5b, 0x0b57, 0xffff, 0xffff, 0xffff],
    [0x0a52, 0x0532, 0x0573, 0xffff, 0xffff],
    [0x021b, 0x017b, 0x0157, 0xffff, 0xffff],
    [0x0531, 0x0573, 0xffff, 0xffff, 0xffff],
    [0x0a5b, 0x057b, 0x0038, 0xffff, 0xffff],
    [0x0028, 0x0258, 0x0578, 0x052a, 0xffff],
    [0x0380, 0x0721, 0x0571, 0x0b27, 0xffff],
    [0x0780, 0x0170, 0x0571, 0xffff, 0xffff],
    [0x07b5, 0x0ba5, 0x0091, 0xffff, 0xffff],
    [0x0109, 0x03a5, 0x0735, 0x02a3, 0xffff],
    [0x0579, 0x0729, 0x0209, 0x07b2, 0xffff],
    [0x0309, 0x0539, 0x0735, 0xffff, 0xffff],
    [0x057a, 0x07ba, 0x0189, 0x0138, 0xffff],
    [0x0289, 0x0129, 0x0278, 0x052a, 0x0257],
    [0x0257, 0x0b27, 0x0295, 0x0823, 0x0289],
    [0x0789, 0x0795, 0xffff, 0xffff, 0xffff],
    [0x0874, 0xffff, 0xffff, 0xffff, 0xffff],
    [0x0748, 0x02b3, 0xffff, 0xffff, 0xffff],
    [0x0a21, 0x0748, 0xffff, 0xffff, 0xffff],
    [0x01a3, 0x0ab3, 0x0487, 0xffff, 0xffff],
    [0x0034, 0x0437, 0xffff, 0xffff, 0xffff],
    [0x074b, 0x042b, 0x0402, 0xffff, 0xffff],
    [0x0743, 0x0403, 0x0a21, 0xffff, 0xffff],
    [0x0ab1, 0x0b41, 0x0401, 0x04b7, 0xffff],
    [0x0910, 0x0748, 0xffff, 0xffff, 0xffff],
    [0x0109, 0x0748, 0x0b32, 0xffff, 0xffff],
    [0x0a29, 0x0209, 0x0748, 0xffff, 0xffff],
    [0x0874, 0x0b09, 0x0ab9, 0x030b, 0xffff],
    [0x0914, 0x0174, 0x0137, 0xffff, 0xffff],
    [0x0b74, 0x0b49, 0x02b9, 0x0129, 0xffff],
    [0x09a2, 0x0792, 0x0372, 0x0497, 0xffff],
    [0x0b74, 0x09b4, 0x0ab9, 0xffff, 0xffff],
    [0x0486, 0x068b, 0xffff, 0xffff, 0xffff],
    [0x0328, 0x0248, 0x0264, 0xffff, 0xffff],
    [0x0486, 0x08b6, 0x01a2, 0xffff, 0xffff],
    [0x0318, 0x0168, 0x0648, 0x01a6, 0xffff],
    [0x0b63, 0x0603, 0x0640, 0xffff, 0xffff],
    [0x0240, 0x0264, 0xffff, 0xffff, 0xffff],
    [0x0a21, 0x0b03, 0x0b60, 0x0640, 0xffff],
    [0x001a, 0x060a, 0x0406, 0xffff, 0xffff],
    [0x0b68, 0x0648, 0x0109, 0xffff, 0xffff],
    [0x0091, 0x0432, 0x0642, 0x0834, 0xffff],
    [0x08b4, 0x0b64, 0x0920, 0x09a2, 0xffff],
    [0x0364, 0x0834, 0x03a6, 0x0930, 0x039a],
    [0x0649, 0x0369, 0x0139, 0x063b, 0xffff],
    [0x0491, 0x0241, 0x0642, 0xffff, 0xffff],
    [0x039a, 0x023a, 0x0349, 0x063b, 0x0364],
    [0x049a, 0x04a6, 0xffff, 0xffff, 0xffff],
    [0x06a5, 0x0874, 0xffff, 0xffff, 0xffff],
    [0x02b3, 0x0487, 0x056a, 0xffff, 0xffff],
    [0x0216, 0x0156, 0x0874, 0xffff, 0xffff],
    [0x0748, 0x05b3, 0x0153, 0x06b5, 0xffff],
    [0x0034, 0x0374, 0x0a56, 0xffff, 0xffff],
    [0x06a5, 0x0274, 0x0024, 0x0b72, 0xffff],
    [0x0521, 0x0625, 0x0403, 0x0743, 0xffff],
    [0x0b15, 0x06b5, 0x0b01, 0x04b7, 0x0b40],
    [0x0091, 0x06a5, 0x0748, 0xffff, 0xffff],
    [0x0910, 0x0874, 0x0b32, 0x06a5, 0xffff],
    [0x0748, 0x0509, 0x0560, 0x0620, 0xffff],
    [0x0950, 0x0560, 0x0630, 0x036b, 0x0748],
    [0x056a, 0x0791, 0x0371, 0x0497, 0xffff],
    [0x0129, 0x02b9, 0x0b49, 0x04b7, 0x06a5],
    [0x0937, 0x0497, 0x0923, 0x0695, 0x0962],
    [0x0956, 0x0b96, 0x0974, 0x09b7, 0xffff],
    [0x0485, 0x08a5, 0x08ba, 0xffff, 0xffff],
    [0x0a52, 0x0253, 0x0543, 0x0483, 0xffff],
    [0x0152, 0x0582, 0x08b2, 0x0854, 0xffff],
    [0x0548, 0x0358, 0x0153, 0xffff, 0xffff],
    [0x0405, 0x00b5, 0x0ba5, 0x003b, 0xffff],
    [0x02a5, 0x0425, 0x0024, 0xffff, 0xffff],
    [0x0b40, 0x03b0, 0x0b54, 0x01b2, 0x0b15],
    [0x0540, 0x0501, 0xffff, 0xffff, 0xffff],
    [0x0910, 0x0a48, 0x0ba8, 0x054a, 0xffff],
    [0x02a3, 0x0a53, 0x0583, 0x0854, 0x0910],
    [0x0520, 0x0950, 0x05b2, 0x0854, 0x058b],
    [0x0548, 0x0358, 0x0509, 0x0530, 0xffff],
    [0x04ba, 0x054a, 0x043b, 0x0149, 0x0413],
    [0x02a5, 0x0425, 0x0291, 0x0249, 0xffff],
    [0x0549, 0x03b2, 0xffff, 0xffff, 0xffff],
    [0x0549, 0xffff, 0xffff, 0xffff, 0xffff],
    [0x0459, 0xffff, 0xffff, 0xffff, 0xffff],
    [0x0459, 0x0b32, 0xffff, 0xffff, 0xffff],
    [0x0a21, 0x0459, 0xffff, 0xffff, 0xffff],
    [0x0b3a, 0x031a, 0x0459, 0xffff, 0xffff],
    [0x0459, 0x0380, 0xffff, 0xffff, 0xffff],
    [0x02b0, 0x0b80, 0x0594, 0xffff, 0xffff],
    [0x0803, 0x0a21, 0x0594, 0xffff, 0xffff],
    [0x0594, 0x0180, 0x01a8, 0x0ab8, 0xffff],
    [0x0450, 0x0051, 0xffff, 0xffff, 0xffff],
    [0x0450, 0x0510, 0x0b32, 0xffff, 0xffff],
    [0x0a25, 0x0245, 0x0204, 0xffff, 0xffff],
    [0x0045, 0x0b05, 0x0ab5, 0x030b, 0xffff],
    [0x0458, 0x0538, 0x0513, 0xffff, 0xffff],
    [0x0512, 0x0852, 0x0b82, 0x0584, 0xffff],
    [0x05a2, 0x0523, 0x0453, 0x0843, 0xffff],
    [0x0845, 0x0a85, 0x0b8a, 0xffff, 0xffff],
    [0x0594, 0x0b67, 0xffff, 0xffff, 0xffff],
    [0x0327, 0x0267, 0x0945, 0xffff, 0xffff],
    [0x0459, 0x021a, 0x0b67, 0xffff, 0xffff],
    [0x0459, 0x061a, 0x0671, 0x0731, 0xffff],
    [0x0380, 0x0594, 0x067b, 0xffff, 0xffff],
    [0x0459, 0x0680, 0x0260, 0x0786, 0xffff],
    [0x07b6, 0x0a21, 0x0380, 0x0594, 0xffff],
    [0x0a61, 0x0671, 0x0701, 0x0078, 0x0459],
    [0x0105, 0x0045, 0x0b67, 0xffff, 0xffff],
    [0x0263, 0x0673, 0x0051, 0x0045, 0xffff],
    [0x0b67, 0x0a45, 0x0a24, 0x0204, 0xffff],
    [0x0a04, 0x05a4, 0x0a30, 0x07a6, 0x0a73],
    [0x067b, 0x0438, 0x0453, 0x0513, 0xffff],
    [0x0826, 0x0786, 0x0812, 0x0584, 0x0851],
    [0x0843, 0x0453, 0x0523, 0x025a, 0x067b],
    [0x0a67, 0x08a7, 0x0a45, 0x0a84, 0xffff],
    [0x094a, 0x0a46, 0xffff, 0xffff, 0xffff],
    [0x094a, 0x046a, 0x032b, 0xffff, 0xffff],
    [0x0941, 0x0421, 0x0462, 0xffff, 0xffff],
    [0x0469, 0x0639, 0x0319, 0x036b, 0xffff],
    [0x06a4, 0x0a94, 0x0380, 0xffff, 0xffff],
    [0x0280, 0x0b82, 0x0a94, 0x06a4, 0xffff],
    [0x0803, 0x0921, 0x0942, 0x0462, 0xffff],
    [0x01b8, 0x0018, 0x016b, 0x0419, 0x0146],
    [0x010a, 0x006a, 0x0046, 0xffff, 0xffff],
    [0x02b3, 0x0610, 0x0460, 0x0a16, 0xffff],
    [0x0420, 0x0624, 0xffff, 0xffff, 0xffff],
    [0x06b3, 0x0063, 0x0460, 0xffff, 0xffff],
    [0x0138, 0x0618, 0x0468, 0x0a16, 0xffff],
    [0x0146, 0x0a16, 0x0184, 0x0b12, 0x01b8],
    [0x0238, 0x0428, 0x0624, 0xffff, 0xffff],
    [0x0846, 0x086b, 0xffff, 0xffff, 0xffff],
    [0x07b4, 0x0b94, 0x0ba9, 0xffff, 0xffff],
    [0x0a92, 0x0972, 0x0732, 0x0947, 0xffff],
    [0x07b4, 0x04b9, 0x0b29, 0x0219, 0xffff],
    [0x0194, 0x0714, 0x0317, 0xffff, 0xffff],
    [0x0380, 0x0794, 0x07b9, 0x0ba9, 0xffff],
    [0x07a9, 0x0479, 0x072a, 0x0078, 0x0702],
    [0x0479, 0x07b9, 0x0b19, 0x01b2, 0x0380],
    [0x0194, 0x0714, 0x0180, 0x0178, 0xffff],
    [0x0ba1, 0x04b1, 0x0041, 0x0b47, 0xffff],
    [0x0a73, 0x02a3, 0x0a47, 0x00a1, 0x0a04],
    [0x047b, 0x024b, 0x0042, 0xffff, 0xffff],
    [0x0304, 0x0347, 0xffff, 0xffff, 0xffff],
    [0x0413, 0x0843, 0x04a1, 0x0b47, 0x04ba],
    [0x02a1, 0x0478, 0xffff, 0xffff, 0xffff],
    [0x047b, 0x024b, 0x0438, 0x0423, 0xffff],
    [0x0784, 0xffff, 0xffff, 0xffff, 0xffff],
    [0x0879, 0x0975, 0xffff, 0xffff, 0xffff],
    [0x0597, 0x0987, 0x02b3, 0xffff, 0xffff],
    [0x0879, 0x0759, 0x021a, 0xffff, 0xffff],
    [0x0859, 0x0758, 0x031a, 0x0b3a, 0xffff],
    [0x0039, 0x0359, 0x0375, 0xffff, 0xffff],
    [0x0759, 0x0279, 0x0029, 0x0b72, 0xffff],
    [0x021a, 0x0059, 0x0035, 0x0375, 0xffff],
    [0x0075, 0x0905, 0x00b7, 0x0a01, 0x00ab],
    [0x0870, 0x0710, 0x0751, 0xffff, 0xffff],
    [0x0b32, 0x0810, 0x0871, 0x0751, 0xffff],
    [0x0208, 0x0528, 0x0758, 0x025a, 0xffff],
    [0x00ab, 0x030b, 0x005a, 0x0708, 0x0075],
    [0x0351, 0x0753, 0xffff, 0xffff, 0xffff],
    [0x012b, 0x071b, 0x0517, 0xffff, 0xffff],
    [0x05a2, 0x0352, 0x0753, 0xffff, 0xffff],
    [0x05ab, 0x05b7, 0xffff, 0xffff, 0xffff],
    [0x0596, 0x09b6, 0x098b, 0xffff, 0xffff],
    [0x0985, 0x0825, 0x0265, 0x0283, 0xffff],
    [0x0a21, 0x0b59, 0x08b9, 0x065b, 0xffff],
    [0x0631, 0x0a61, 0x0683, 0x0965, 0x0698],
    [0x0b63, 0x0360, 0x0650, 0x0590, 0xffff],
    [0x0659, 0x0069, 0x0260, 0xffff, 0xffff],
    [0x03b0, 0x0b60, 0x0690, 0x0965, 0x0a21],
    [0x001a, 0x060a, 0x0059, 0x0065, 0xffff],
    [0x08b0, 0x0b50, 0x0510, 0x0b65, 0xffff],
    [0x0851, 0x0081, 0x0865, 0x0283, 0x0826],
    [0x058b, 0x065b, 0x0508, 0x025a, 0x0520],
    [0x0830, 0x0a65, 0xffff, 0xffff, 0xffff],
    [0x03b6, 0x0536, 0x0135, 0xffff, 0xffff],
    [0x0651, 0x0612, 0xffff, 0xffff, 0xffff],
    [0x03b6, 0x0536, 0x03a2, 0x035a, 0xffff],
    [0x065a, 0xffff, 0xffff, 0xffff, 0xffff],
    [0x06a7, 0x0a87, 0x0a98, 0xffff, 0xffff],
    [0x0b32, 0x086a, 0x098a, 0x0768, 0xffff],
    [0x0621, 0x0861, 0x0981, 0x0768, 0xffff],
    [0x0698, 0x0768, 0x0619, 0x036b, 0x0631],
    [0x0370, 0x07a0, 0x0a90, 0x0a76, 0xffff],
    [0x0702, 0x0b72, 0x0790, 0x0a76, 0x07a9],
    [0x0962, 0x0192, 0x0976, 0x0390, 0x0937],
    [0x0190, 0x076b, 0xffff, 0xffff, 0xffff],
    [0x076a, 0x07a1, 0x0871, 0x0081, 0xffff],
    [0x0081, 0x0871, 0x07a1, 0x0a76, 0x0b32],
    [0x0087, 0x0607, 0x0206, 0xffff, 0xffff],
    [0x0087, 0x0607, 0x00b3, 0x006b, 0xffff],
    [0x076a, 0x017a, 0x0371, 0xffff, 0xffff],
    [0x012b, 0x071b, 0x016a, 0x0176, 0xffff],
    [0x0237, 0x0276, 0xffff, 0xffff, 0xffff],
    [0x06b7, 0xffff, 0xffff, 0xffff, 0xffff],
    [0x08a9, 0x08ba, 0xffff, 0xffff, 0xffff],
    [0x0832, 0x0a82, 0x098a, 0xffff, 0xffff],
    [0x0b21, 0x09b1, 0x08b9, 0xffff, 0xffff],
    [0x0831, 0x0819, 0xffff, 0xffff, 0xffff],
    [0x0903, 0x0b93, 0x0a9b, 0xffff, 0xffff],
    [0x02a9, 0x0290, 0xffff, 0xffff, 0xffff],
    [0x0903, 0x0b93, 0x0921, 0x09b2, 0xffff],
    [0x0190, 0xffff, 0xffff, 0xffff, 0xffff],
    [0x0a10, 0x08a0, 0x0ba8, 0xffff, 0xffff],
    [0x0832, 0x0a82, 0x0810, 0x08a1, 0xffff],
    [0x0b20, 0x0b08, 0xffff, 0xffff, 0xffff],
    [0x0830, 0xffff, 0xffff, 0xffff, 0xffff],
    [0x0a13, 0x0a3b, 0xffff, 0xffff, 0xffff],
    [0x02a1, 0xffff, 0xffff, 0xffff, 0xffff],
    [0x0b23, 0xffff, 0xffff, 0xffff, 0xffff],
    [0xffff, 0xffff, 0xffff, 0xffff, 0xffff],
];

pub struct MeshGenerator<'a> {
    chunk: &'a Chunk,
    pub dual_cells: Vec<DirectionMapper<Voxel<'a>>>,

    vertices: Vec<Position>,
    normal: Vec<Normal>,
    texcoords: Vec<TexCoord>,
    indices: Vec<u16>,

    current: u16,
    pub size: f32,

    pub count: usize,
}

impl<'a> MeshGenerator<'a> {
    pub fn new(chunk: &'a Chunk, size: f32) -> Self {
        Self {
            chunk,
            dual_cells: Vec::new(),
            vertices: Vec::new(),
            normal: Vec::new(),
            texcoords: Vec::new(),
            indices: Vec::new(),
            current: 0,
            size,
            count: 0,
        }
    }
    pub fn create_dualgrid(&mut self) {
        let root = self.chunk.get_root();
        self.node_proc(&root);
    }

    fn node_proc(&mut self, node: &Voxel<'a>) {
        if node.is_leaf() {
            return;
        }

        // Unwrap, because we've asserted that node is subdivided so it must have child
        let children = Direction::map(|dir| node.get_child(dir));

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
    fn face_proc_children<T: Dimension>(&mut self, children: &DirectionMapper<Voxel<'a>>) {
        for (dir1, dir2) in T::FACE_PROC_DIR_GROUPS.iter() {
            self.face_proc::<T>([
                &children[*dir1],
                &children[*dir2]
            ]);
        }
    }
    fn face_proc<T: Dimension>(&mut self, nodes: [&Voxel<'a>; 2]) {
        if nodes.iter().all(|n| n.is_leaf()) {
            return;
        }
        let tuples = T::FACE_PROC_DIR_TUPLES;
        // Can unwrap because we've asserted that all nodes are subdivided
        let children = DirectionMapper::new([
            nodes[tuples[0].0].get_child(tuples[0].1),
            nodes[tuples[1].0].get_child(tuples[1].1),
            nodes[tuples[2].0].get_child(tuples[2].1),
            nodes[tuples[3].0].get_child(tuples[3].1),
            nodes[tuples[4].0].get_child(tuples[4].1),
            nodes[tuples[5].0].get_child(tuples[5].1),
            nodes[tuples[6].0].get_child(tuples[6].1),
            nodes[tuples[7].0].get_child(tuples[7].1),
        ]);

        self.face_proc_children::<T>(&children);
        self.edge_proc_children::<T::FaceEdges1>(&children);
        self.edge_proc_children::<T::FaceEdges2>(&children);
        self.vert_proc(children.data);
    }
    fn edge_proc_children<T>(&mut self, children: &DirectionMapper<Voxel<'a>>)
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
    fn edge_proc<T>(&mut self, nodes: [&Voxel<'a>; 4])
        where T: Dimension {
        if nodes.iter().all(|n| n.is_leaf()) {
            return;
        }

        let t = T::EDGE_PROC_DIR_TUPLES;

        let children = DirectionMapper::new([
            nodes[t[0].0].get_child(t[0].1),
            nodes[t[1].0].get_child(t[1].1),
            nodes[t[2].0].get_child(t[2].1),
            nodes[t[3].0].get_child(t[3].1),
            nodes[t[4].0].get_child(t[4].1),
            nodes[t[5].0].get_child(t[5].1),
            nodes[t[6].0].get_child(t[6].1),
            nodes[t[7].0].get_child(t[7].1),
        ]);
        self.edge_proc_children::<T>(&children);
        self.vert_proc(children.data);
    }
    fn vert_proc(&mut self, mut nodes: [Voxel<'a>; 8]) {
        loop {
            let mut has_subdivided = false;
            for (index, node) in nodes.iter_mut().enumerate() {
                if node.is_subdivided() {
                    has_subdivided = true;
                    let dir = Direction::from(index as u8);
                    let opposite_dir_node = node.get_child(dir.opposite());
                    *node = opposite_dir_node;
                }
            }
            if !has_subdivided {
                break;
            }
        }
        // Now all nodes are leaf node
        self.add_dualcell(DirectionMapper::new(nodes));
        // TODO add dual cell
    }
    fn add_dualcell(&mut self, nodes: DirectionMapper<Voxel<'a>>) {
        let mut edge_index: u8 = 0;
        for node in nodes.iter().rev() {
            edge_index <<= 1;
            if *node.get_value() == VoxelData::EMPTY {
                edge_index |= 1;
            }
        }

        let edge_bin = EDGE_TABLE[edge_index as usize];
        for edges in edge_bin.iter() {
            let edges = *edges;
            if edges == std::u16::MAX {
                break;
            }
            debug_assert_eq!(edges >> 12, 0); // Highest 4 bits are always 0
            let edge1: Edge = ((edges & 0b1111) as u8).into();
            let edge2: Edge = (((edges >> 4) & 0b1111) as u8).into();
            let edge3: Edge = ((edges >> 8) as u8).into();

            self.add_triangle([edge1, edge2, edge3], &nodes);
        }

        self.dual_cells.push(nodes);
    }

    fn add_triangle(&mut self, edges: [Edge; 3], nodes: &DirectionMapper<Voxel>) {
        for edge in edges.iter() {
            let (v1, v2) = edge.vertices();
            let node1 = nodes[v1].get_bounds().center();
            let node2 = nodes[v2].get_bounds().center();
            let pos: Vector3<f32> = (node1.coords + node2.coords) * (self.size * 0.5);
            self.vertices.push(pos.into());
            self.texcoords.push(TexCoord([0.0, 0.0]));
            self.normal.push(Normal([0.0, 0.0, 0.0]));
            self.current += 1;
        }
        // Making faces visible from both sides
        self.indices.push(self.current - 1);
        self.indices.push(self.current - 2);
        self.indices.push(self.current - 3);
        self.count += 1;
        println!("added a triangle {}", self.count);
    }

    pub fn into_mesh_builder(self) -> MeshBuilder<'static> {
        MeshBuilder::new()
            .with_vertices(self.vertices)
            .with_vertices(self.normal)
            .with_vertices(self.texcoords)
            .with_indices(Indices::U16(self.indices.into()))
    }


    pub fn gen_wireframe(&self) -> DebugLinesComponent {
        let mut wireframe = DebugLinesComponent::with_capacity(100);
        for node in self.chunk.iter_leaf() {
            let bounds = node.get_bounds();
            let position = bounds.get_position();
            let width = bounds.get_width();

            wireframe.add_sphere(bounds.center(), 0.01,
                                 8,
                                 8,
                                 if node.get_value().is_empty() {
                                     Srgba::new(1.0, 1.0, 1.0, 1.0)
                                 } else {

                                     Srgba::new(1.0, 0.5, 0.23, 1.0)
                                 }) ;

            for i in 0..3 {
                let mut dir: [f32; 3] = [0.0, 0.0, 0.0];
                dir[i] = width;
                wireframe.add_direction(
                    position,
                    dir.into(),
                    Srgba::new(1.0, 0.5, 0.23, 1.0),
                );
            }
        }
        for cell in &self.dual_cells {
            let origin = cell[Direction::RearRightTop].get_bounds().center() * self.size;
            for dir in &[Direction::FrontRightTop, Direction::RearRightBottom, Direction::RearLeftTop] {
                wireframe.add_line(
                    origin,
                    cell[*dir].get_bounds().center() * self.size,
                    Srgba::new(1.0, 0.2, 1.0, 1.8),
                );
            }
        }

        for mut indices in &self.indices.iter().chunks(3) {
            let x = *indices.next().unwrap() as usize;
            let y = *indices.next().unwrap() as usize;
            let z = *indices.next().unwrap() as usize;
            debug_assert!(indices.next().is_none());
            let x_vert: [f32; 3] = self.vertices[x].0;
            let y_vert: [f32; 3] = self.vertices[y].0;
            let z_vert: [f32; 3] = self.vertices[z].0;

            wireframe.add_line(
                x_vert.into(),
                y_vert.into(),
                Srgba::new(1.0, 1.0, 1.0, 1.8),
            );
            wireframe.add_line(
                y_vert.into(),
                z_vert.into(),
                Srgba::new(1.0, 1.0, 1.0, 1.8),
            );
            wireframe.add_line(
                z_vert.into(),
                x_vert.into(),
                Srgba::new(1.0, 1.0, 1.0, 1.8),
            );
        }

        wireframe
    }
}
