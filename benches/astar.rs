use std::marker::PhantomData;

use criterion::{criterion_group, criterion_main, Criterion};

use game_lib::actor::Direction;
use game_lib::path::{self, astar, HeuristicModel, Model, Optimizer, Sampler};
use game_lib::Position;

#[derive(Copy, Clone, Debug)]
enum Tile {
    // A blocked tile, or Wall
    W,
    // An unblocked tile, or Floor
    O,
}

trait Heuristic {
    fn calc(current: (isize, isize), goal: (isize, isize)) -> usize;
}

macro_rules! make_heuristic {
    ($($name:ident: ($x1:ident, $y1:ident), ($x2:ident, $y2:ident)  => $func:tt),*) => {
        $(
            #[derive(Debug, Copy, Clone)]
            struct $name;

            impl Heuristic for $name {
                #[inline(always)]
                fn calc(($x1, $y1): (isize, isize), ($x2, $y2): (isize, isize)) -> usize {
                    let c = $func;
                    c as usize
                }
            }
        )*
    }
}

make_heuristic! {
    Diagonal: (x1, y1), (x2, y2) => {
        let (dx, dy) = ((x1 - x2).abs(), (y1 - y2).abs());

        2 * (dx + dy) - dx.min(dy)
    },
    Manhattan: (x1, y1), (x2, y2) => {
        let (dx, dy) = ((x1 - x2).abs(), (y1 - y2).abs());

        2 * (dx + dy)
    },
    Zero: (_x1, _y1), (_x2, _y2) => 0
}

#[derive(Clone, Debug)]
struct BenchModel<H>
where
    H: Heuristic,
{
    width: u32,
    height: u32,
    map: Vec<Tile>,
    heuristic: PhantomData<H>,
}

impl<H: Heuristic> BenchModel<H> {
    #[inline(always)]
    fn pos2ind<T: Into<(u32, u32)>>(&self, pos: T) -> usize {
        let (x, y) = pos.into();
        x as usize + y as usize * self.width as usize
    }

    #[inline]
    fn get<T: Into<(u32, u32)>>(&self, pos: T) -> Option<&Tile> {
        let index = self.pos2ind(pos);
        self.map.get(index)
    }

    fn is_blocked<T: Into<(u32, u32)>>(&self, pos: T) -> bool {
        match self.get(pos) {
            Some(Tile::W) => true,
            Some(Tile::O) => false,
            None => true,
        }
    }

    pub fn new(width: u32, height: u32, map: Vec<Tile>) -> Self {
        debug_assert_eq!(map.len(), height as usize * width as usize);

        BenchModel { width, height, map, heuristic: PhantomData }
    }
}

impl<H: Heuristic> Model for BenchModel<H> {
    type Control = Direction;
    type State = Position;
    type Cost = usize;

    fn cost(&self, _: &Self::State, action: &Self::Control, _: &Self::State) -> Self::Cost {
        use Direction::*;
        match action {
            NorthEast | NorthWest | SouthEast | SouthWest => 3,
            _ => 2,
        }
    }

    /// Convergence occurs adjacent to the goal, not on the goal in this case
    fn converge(&self, current: &Self::State, goal: &Self::State) -> bool {
        let (x, y) = (current.x as isize, current.y as isize);
        let (gx, gy) = (goal.x as isize, goal.y as isize);

        (x - gx).abs() <= 1 && (y - gy).abs() <= 1
    }

    /// Nothing to do on initialization
    #[inline(always)]
    fn init(&mut self, _: &Self::State) {}

    fn integrate(
        &self,
        previous: &Self::State,
        control: &Self::Control,
    ) -> Option<Self::State> {
        let next = control.step_from(previous.x, previous.y);
        if !self.is_blocked(next) {
            Some(next.into())
        } else {
            None
        }
    }
}

impl<H: Heuristic> HeuristicModel for BenchModel<H> {
    fn heuristic(&self, c: &Self::State, n: &Self::State) -> Self::Cost {
        let c = (c.x as isize, c.y as isize);
        let n = (n.x as isize, n.y as isize);
        H::calc(c, n)
    }
}

#[derive(Debug, Clone)]
struct Cardinal;

impl Sampler<BenchModel<Manhattan>> for Cardinal {
    #[inline(always)]
    fn sample(&mut self, _: &BenchModel<Manhattan>, _: &Position) -> &[Direction] {
        use Direction::*;
        &[North, South, East, West]
    }
}

impl Sampler<BenchModel<Zero>> for Cardinal {
    #[inline(always)]
    fn sample(&mut self, _: &BenchModel<Zero>, _: &Position) -> &[Direction] {
        use Direction::*;
        &[North, South, East, West]
    }
}

#[derive(Debug, Clone)]
struct Octile;

impl<H: Heuristic> Sampler<BenchModel<H>> for Octile {
    #[inline(always)]
    fn sample(&mut self, _: &BenchModel<H>, _: &Position) -> &[Direction] {
        use Direction::*;
        &[North, NorthWest, West, SouthWest, South, SouthEast, East, NorthEast]
    }
}

fn map<H: Heuristic>() -> BenchModel<H> {
    use Tile::*;
    let width = 32;
    let height = 24;
    let map = vec![
        O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O,
        O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O,
        O, O, O, O, O, W, W, W, W, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O,
        O, O, O, O, O, W, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O,
        O, O, O, O, O, W, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O,
        O, O, O, O, O, W, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O,
        O, O, O, O, O, W, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O,
        O, O, O, O, O, W, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O,
        O, O, O, O, O, W, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O,
        O, O, O, O, O, W, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O,
        O, O, O, O, O, W, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O,
        O, O, O, O, O, W, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O,
        O, O, O, O, O, W, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O,
        O, O, O, O, O, W, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O,
        O, O, O, O, O, W, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O,
        O, O, O, O, O, W, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O,
        O, O, O, O, O, W, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O,
        O, O, O, O, O, W, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O,
        O, O, O, O, O, W, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O,
        O, O, O, O, O, W, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O,
        O, O, O, O, O, W, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O,
        O, O, O, O, O, W, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O,
        O, O, O, O, O, W, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O,
        O, O, O, O, O, W, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O,
        O, O, O, O, O, W, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O,
        O, O, O, O, O, W, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O,
        O, O, O, O, O, W, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O,
        O, O, O, O, O, W, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O,
        O, O, O, O, O, W, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O,
        O, O, O, O, O, W, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O,
        O, O, O, O, O, W, W, W, W, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O,
        O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O,
        O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O,
    ];

    BenchModel::new(width, height, map)
}

macro_rules! full_path_bench {
    ($($name:ident, $title:expr, $samp:expr, $heuristic:ty {$start:expr => $goal:expr}),*) => {
        $(
            fn $name(c: &mut Criterion) {
                let mut map = map();

                let start = $start;
                let goal = $goal;
                let mut sampler = $samp;

                c.bench_function($title, move |b| {
                    b.iter(|| {
                        let mut planner: astar::AStar<BenchModel<$heuristic>> =
                            astar::AStar::new();
                        planner.optimize(&mut map, &start, &goal, &mut sampler);
                    });
                });
            }
        )*
    };
}

full_path_bench! {
    full_octile, "Full Admissable Octile Path", Octile, Diagonal {
        Position::new(30, 12) => Position::new(0, 15)
    },
    full_cardinal, "Full Admissable Cardinal Path", Cardinal, Manhattan {
        Position::new(30, 12) => Position::new(0, 15)
    },
    full_dijkstra_octile, "Zero Heuristic on Octile grid", Octile, Zero {
        Position::new(30, 12) => Position::new(0, 15)
    },
    full_dijkstra_cardinal, "Zero Heuristic on Cardinal grid", Cardinal, Zero {
        Position::new(30, 12) => Position::new(0, 15)
    }
}

fn single_iter(c: &mut Criterion) {
    let mut map = map();
    let start = Position::new(31, 15);
    let goal = Position::new(0, 20);
    let mut sampler = Octile;

    let mut planner: astar::AStar<BenchModel<Diagonal>> = astar::AStar::new();
    c.bench_function("Single octile iteration", move |b| {
        b.iter(|| {
            if let path::PathResult::Final(_) =
                planner.next_trajectory(&mut map, &start, &goal, &mut sampler)
            {
                planner.clear();
            }
        });
    });
}

criterion_group!(octile, full_octile, full_dijkstra_octile);
criterion_group!(cardinal, full_cardinal, full_dijkstra_cardinal);
criterion_group!(single_path, single_iter);
criterion_main!(octile, cardinal, single_path);
