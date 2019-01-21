pub mod actor;
pub mod map;
pub mod path;

use std::ops::{Add, Sub};

/// An (x,y) position in the game world
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

impl Add for Position {
    type Output = Position;

    fn add(self, other: Position) -> Self::Output {
        Position { x: self.x + other.x, y: self.y + other.y }
    }
}

impl Sub for Position {
    type Output = Position;

    fn sub(self, other: Position) -> Self::Output {
        Position { x: self.x - other.x, y: self.y - other.y }
    }
}

macro_rules! impl_conversion {
    ($($num:ty),+) => {
        $(
            impl Into<($num, $num)> for Position {
                fn into(self) -> ($num, $num) {
                    (self.x as $num, self.y as $num)
                }
            }

            impl From<($num, $num)> for Position {
                fn from((x, y): ($num, $num)) -> Self {
                    Position { x: x as u32, y: y as u32 }
                }
            }
        )+
    };
}

impl_conversion!(u32, u64, usize, i32, isize, i64);
