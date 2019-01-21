use super::Position;
use crate::map::Map;
use crate::path::{HeuristicModel, Model, Sampler, State};

pub type ActionResult = Result<(), String>;

pub trait Action<A: Actor> {
    fn execute(&self, map: &Map, actor: &mut A) -> ActionResult;
}

pub trait Actor {
    fn take_turn(&mut self, map: &Map) -> Box<dyn Action<Self>>;
}

#[derive(Debug, Clone)]
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
        Box::new(Movement::None)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Movement {
    Walk(Direction),
    None,
}

impl Default for Movement {
    fn default() -> Self {
        Movement::None
    }
}

pub struct WalkSampler {
    movements: [Movement; 9],
}

impl WalkSampler {
    pub fn new() -> Self {
        use Direction::*;
        use Movement::*;

        WalkSampler {
            movements: [
                Walk(North),
                Walk(NorthEast),
                Walk(East),
                Walk(SouthEast),
                Walk(South),
                Walk(SouthWest),
                Walk(West),
                Walk(NorthWest),
                None,
            ],
        }
    }
}

impl Sampler<TurnOptimal> for WalkSampler {
    #[inline]
    fn sample(&mut self, _: &TurnOptimal, _: &Monster) -> &[Movement] {
        &self.movements
    }
}

impl Action<Monster> for Movement {
    fn execute(&self, map: &Map, actor: &mut Monster) -> ActionResult {
        use Movement::*;

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

#[derive(Clone, Debug)]
pub struct TurnOptimal {
    map: Map,
}

impl TurnOptimal {
    pub fn new(map: Map) -> Self {
        TurnOptimal { map }
    }

    pub fn return_map(self) -> Map {
        self.map
    }
}

impl Model for TurnOptimal {
    type Control = Movement;
    type State = Monster;
    type Cost = usize;

    /// Convergence occurs adjacent to the goal, not on the goal in this case
    fn converge(&self, current: &Self::State, goal: &Self::State) -> bool {
        let (x, y) = (current.pos.x as i64, current.pos.y as i64);
        let (gx, gy) = (goal.pos.x as i64, goal.pos.y as i64);

        (x - gx).abs() <= 1 && (y - gy).abs() <= 1
    }

    fn integrate(
        &self,
        previous: &Self::State,
        control: &Self::Control,
    ) -> Option<Self::State> {
        let mut next = previous.clone();

        if control.execute(&self.map, &mut next).is_ok() {
            Some(next)
        } else {
            None
        }
    }

    /// Nothing to do on initialization
    #[inline(always)]
    fn init(&mut self, _: &Self::State) {}

    #[inline(always)]
    fn cost(&self, _current: &Self::State, _next: &Self::State) -> Self::Cost {
        1
    }
}

impl HeuristicModel for TurnOptimal {
    /// Reasonable estimate for the number of turns required to reach the player
    fn heuristic(&self, current: &Self::State, goal: &Self::State) -> Self::Cost {
        let Position { x, y } = current.pos;
        let Position { x: gx, y: gy } = goal.pos;

        ((gx as isize - x as isize).abs() + (gy as isize - y as isize).abs()) as usize
    }
}
