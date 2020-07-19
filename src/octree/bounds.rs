use amethyst::core::math;
use super::index_path::IndexPath;


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
}


impl From<IndexPath> for Bounds {
    fn from(index_path: IndexPath) -> Self {
        let mut ip_iter = index_path;
        let mut bounds = Bounds::new();
        for dir in ip_iter {
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
        }
        bounds
    }
}
