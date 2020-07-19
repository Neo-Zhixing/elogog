#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Direction {
    FrontLeftBottom = 0,
    FrontRightBottom = 1,
    RearLeftBottom = 2,
    RearRightBottom = 3,
    FrontLeftTop = 4,
    FrontRightTop = 5,
    RearLeftTop = 6,
    RearRightTop = 7,
}

impl Direction {
    #[inline]
    pub fn is_max_x(&self) -> bool {
        *self as u8 & 0b001 != 0
    }

    #[inline]
    pub fn is_min_x(&self) -> bool {
        *self as u8 & 0b001 == 0
    }

    #[inline]
    pub fn is_max_y(&self) -> bool {
        *self as u8 & 0b010 != 0
    }

    #[inline]
    pub fn is_min_y(&self) -> bool {
        *self as u8 & 0b010 == 0
    }

    #[inline]
    pub fn is_max_z(&self) -> bool {
        *self as u8 & 0b100 != 0
    }

    #[inline]
    pub fn is_min_z(&self) -> bool {
        *self as u8 & 0b100 == 0
    }
}

impl From<u8> for Direction {
    fn from(val: u8) -> Self {
        let val = val & 0b111;
        match val {
            0 => Direction::FrontLeftBottom,
            1 => Direction::FrontRightBottom,
            2 => Direction::RearLeftBottom,
            3 => Direction::RearRightBottom,
            4 => Direction::FrontLeftTop,
            5 => Direction::FrontRightTop,
            6 => Direction::RearLeftTop,
            7 => Direction::RearRightTop,
            _ => unreachable!()
        }
    }
}
