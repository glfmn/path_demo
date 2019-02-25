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
        let dx = f64::from(self.x) - f64::from(other.x);
        let dy = f64::from(self.y) - f64::from(other.y);

        dx * dx + dy * dy
    }

    pub fn dist(&self, other: Position) -> f64 {
        self.square_dist(other).sqrt()
    }
}

impl path::State for Position {
    type Position = Self;
    fn grid_position(&self) -> Self::Position {
        self.clone()
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
