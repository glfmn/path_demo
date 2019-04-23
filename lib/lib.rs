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

    pub fn zero() -> Self {
        Position { x: 0, y: 0 }
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
            #[allow(clippy::cast_lossless)]
            impl Into<($num, $num)> for Position {
                fn into(self) -> ($num, $num) {
                    (self.x as $num, self.y as $num)
                }
            }

            #[allow(clippy::cast_lossless)]
            impl From<($num, $num)> for Position {
                fn from((x, y): ($num, $num)) -> Self {
                    Position { x: x as u32, y: y as u32 }
                }
            }
        )+
    };
}

impl_conversion!(u8, u16, u32, u64, usize, i16, i32, isize, i64);

/// A rectangular area
///
/// Useful to create relative tansforms, converting positions relative to the area
/// of the rectangle to positions in the parent space of the rectangle.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Rect {
    /// The top left position of the rectangle
    pub pos: Position,
    /// The width of the rectangle
    pub w: u32,
    /// The height of the rectangle
    pub h: u32,
}

impl Rect {
    pub fn new<P: Into<Position>>(pos: P, w: u32, h: u32) -> Self {
        Rect { pos: pos.into(), w, h }
    }

    /// Create a rectangle whose top-left is at the origin
    pub fn origin(w: u32, h: u32) -> Self {
        Rect { pos: Position::new(0, 0), w, h }
    }

    /// Calculate the global position of a position inside the rectangle
    ///
    /// Returns `None` when the position falls outside of the Rectangle's area.
    ///
    /// A position in the rectangle at `(0, 0)` is just the top-left coordinate of
    /// the rectangle.
    ///
    /// ```
    /// # use game_lib::{ Position, Rect };
    /// let pos = Position::new(0, 0);
    /// let rect = Rect::new(Position::new(10, 10), 10, 10);
    /// assert_eq!(rect.transform(&pos), Some(rect.pos));
    /// ````
    ///
    /// A rectangle at the origin does not change positions transformed to its
    /// coordinates.
    ///
    /// ```
    /// # use game_lib::{ Position, Rect };
    /// let zero = Rect::origin(10, 10);
    ///
    /// let pos = Position::new(1, 1);
    /// assert_eq!(zero.transform(&pos), Some(pos));
    ///
    /// // Positions out of bounds are still None
    /// let out_of_bounds = Position::new(11, 11);
    /// assert_eq!(zero.transform(&out_of_bounds), None);
    /// ```
    pub fn transform(&self, pos: &Position) -> Option<Position> {
        if pos.x > self.w || pos.y > self.h {
            None
        } else {
            Some(self.pos.clone() + pos.clone())
        }
    }
}
