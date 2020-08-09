use amethyst::core::math;
use super::index_path::IndexPath;
use super::direction::Direction;

#[derive(Clone)]
pub struct Bounds {
    x: u32,
    y: u32,
    z: u32,
    width: u32,
}

impl Bounds {
    const MAX_WIDTH: u32 = (1 << 31);
    pub fn new() -> Self {
        Bounds {
            x: 0,
            y: 0,
            z: 0,
            width: Self::MAX_WIDTH,
        }
    }
    pub fn get_position(&self) -> math::Point3<f32> {
        math::Point3::new(
            self.x as f32,
            self.y as f32,
            self.z as f32,
        ) / (Self::MAX_WIDTH as f32)
    }
    pub fn get_width(&self) -> f32 {
        self.width as f32 / Self::MAX_WIDTH as f32
    }
    pub fn center(&self) -> math::Point3<f32> {
        let half_width = self.get_width() / 2.0;
        self.get_position() + math::Vector3::new(half_width, half_width, half_width)
    }

    pub fn half(&self,dir: Direction) -> Bounds {
        let mut bounds = self.clone();
        bounds.width >>= 1; // half the width
        if dir.is_max_x() {
            bounds.x += bounds.width;
        }
        if dir.is_max_y() {
            bounds.y += bounds.width;
        }
        if dir.is_max_z() {
            bounds.z += bounds.width;
        }
        bounds
    }
}


impl From<IndexPath> for Bounds {
    fn from(index_path: IndexPath) -> Self {
        let mut ip_iter = index_path;
        let mut bounds = Bounds::new();
        for dir in ip_iter {
            bounds = bounds.half(dir);
        }
        bounds
    }
}
