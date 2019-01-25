use super::Position;
use crate::map::Map;
use crate::path::{self, HeuristicModel, Model, Optimizer, PathResult, Sampler, State};

pub type ActionResult = Result<(), String>;

pub trait Action {
    fn execute(&self, map: &Map, actor: &mut Actor) -> ActionResult;
}

#[derive(Debug, Clone)]
pub struct Actor {
    pub pos: Position,
    mana: usize,
    max_mana: usize,
}

pub enum Goal {
    GoTo(Position),
    Do(Box<dyn Action>),
    None,
}

impl Goal {
    pub fn new() -> Self {
        Goal::None
    }

    pub fn go_to<P>(goal: P) -> Self
    where
        P: Into<Position>,
    {
        Goal::GoTo(goal.into())
    }
}

impl Actor {
    pub fn new(x: u32, y: u32, mana: usize, max_mana: usize) -> Self {
        Actor { pos: Position { x, y }, mana, max_mana }
    }

    pub fn take_turn(&mut self, goal: Goal, map: &Map) -> Box<dyn Action> {
        let map = map.clone();

        match goal {
            Goal::GoTo(position) => {
                // Create a goal to go to the defined position
                let mut goal = self.clone();
                goal.pos = position;
                let mut planner = path::astar::AStar::new();
                let mut walker = WalkSampler::new();
                let mut model = TurnOptimal::new(map);
                let trajectory = planner.optimize(&mut model, self.clone(), goal, &mut walker);

                if let PathResult::Final(trajectory) = trajectory {
                    if let Some((_, action)) = trajectory.trajectory.first() {
                        Box::new(action.clone())
                    } else {
                        Box::new(Movement::None)
                    }
                } else {
                    Box::new(Movement::None)
                }
            }
            Goal::Do(action) => action,
            Goal::None => Box::new(Movement::None),
        }
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
    fn sample(&mut self, _: &TurnOptimal, _: &Actor) -> &[Movement] {
        &self.movements
    }
}

impl Action for Movement {
    fn execute(&self, map: &Map, actor: &mut Actor) -> ActionResult {
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

impl State for Actor {
    type Position = Position;

    fn grid_position(&self) -> Self::Position {
        self.pos.clone()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Heuristic {
    Manhattan,
    Chebyshev,
}

impl Heuristic {
    #[inline(always)]
    pub fn calculate(&self, (cx, cy): (isize, isize), (gx, gy): (isize, isize)) -> usize {
        use Heuristic::*;

        let (dx, dy) = ((cx - gx).abs(), (cy - gy).abs());

        let estimate = match self {
            Manhattan => dx + dy,
            Chebyshev => (dx + dy) - 1 * dx.min(dy),
        };

        (estimate + 1) as usize
    }
}

#[derive(Clone, Debug)]
pub struct TurnOptimal {
    heurisitc: Heuristic,
    map: Map,
}

impl TurnOptimal {
    pub fn new(map: Map) -> Self {
        TurnOptimal { map, heurisitc: Heuristic::Manhattan }
    }

    pub fn set_heuristic(&mut self, heuristic: Heuristic) {
        self.heurisitc = heuristic
    }

    pub fn use_chebyshev(&mut self) {
        self.heurisitc = Heuristic::Chebyshev
    }

    pub fn use_manhattan(&mut self) {
        self.heurisitc = Heuristic::Manhattan
    }

    pub fn return_map(self) -> Map {
        self.map
    }
}

impl Model for TurnOptimal {
    type Control = Movement;
    type State = Actor;
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
        self.heurisitc.calculate(current.pos.clone().into(), goal.pos.clone().into())
    }
}
