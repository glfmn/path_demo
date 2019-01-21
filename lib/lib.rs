pub mod actor;
pub mod map;
pub mod path;

/// An (x,y) position in the game world
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}
