pub mod actor;
pub mod map;
pub mod path;

use std::ops::{Add, Mul, Sub};

/// An (x,y) position in the game world
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

impl Position {
    pub fn new(x: u32, y: u32) -> Self {
        Position { x, y }
    }

    pub fn square_dist(&self, other: Position) -> f64 {
        let dx = self.x as f64 - other.x as f64;
        let dy = self.y as f64 - other.y as f64;

        dx * dx + dy * dy
    }

    pub fn dist(&self, other: Position) -> f64 {
        self.square_dist(other).sqrt()
    }
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

/// Dot product of positions
impl Mul for Position {
    type Output = u32;

    fn mul(self, rhs: Position) -> Self::Output {
        self.x * rhs.x + self.y * rhs.y
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
