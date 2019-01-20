use super::Position;
use crate::map::Map;
use crate::path::State;

pub type ActionResult = Result<(), String>;

pub trait Action<A: Actor> {
    fn execute(&self, map: &Map, actor: &mut A) -> ActionResult;
}

pub trait Actor {
    fn take_turn(&mut self, map: &Map) -> Box<dyn Action<Self>>;
}

pub struct Monster {
    pub pos: Position,
    mana: usize,
    max_mana: usize,
}

impl Monster {
    pub fn new(x: u32, y: u32, mana: usize, max_mana: usize) -> Self {
        Monster { pos: Position { x, y }, mana, max_mana }
    }
}

impl Actor for Monster {
    fn take_turn(&mut self, map: &Map) -> Box<dyn Action<Self>> {
        Box::new(MovementAction::None)
    }
}

pub enum Direction {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl Direction {
    fn step_from(&self, x: u32, y: u32) -> (u32, u32) {
        use Direction::*;
        match *self {
            North => (x, y + 1),
            NorthEast => (x + 1, y + 1),
            East => (x + 1, y),
            SouthEast => (x + 1, y - 1),
            South => (x, y - 1),
            SouthWest => (x - 1, y - 1),
            West => (x - 1, y),
            NorthWest => (x - 1, y + 1),
        }
    }
}

pub enum MovementAction {
    Walk(Direction),
    None,
}

impl Action<Monster> for MovementAction {
    fn execute(&self, map: &Map, actor: &mut Monster) -> ActionResult {
        use MovementAction::*;

        match self {
            None => Ok(()),
            Walk(direction) => {
                let Position { x, y } = &actor.pos;
                let (nx, ny) = direction.step_from(*x, *y);

                if let Some(tile) = map.get(nx as u32, ny as u32) {
                    if !tile.is_blocking() {
                        actor.pos = Position { x: nx, y: ny };
                        Ok(())
                    } else {
                        Err(format!("Position ({},{}) is blocked", nx, ny))
                    }
                } else {
                    Err(format!("Position ({},{}) does not exist on the map", nx, ny))
                }
            }
        }
    }
}

impl State for Monster {
    type Position = Position;

    fn grid_position(&self) -> Self::Position {
        self.pos.clone()
    }
}
