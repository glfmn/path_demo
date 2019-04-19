#![allow(unused)]

use slog::{info, o};
use slog::{Drain, Logger};

use game_lib::actor::{Actor, Heuristic, TurnOptimal, WalkSampler};
use game_lib::map::{generate, Map, Tile};
use game_lib::path::astar::AStar;
use game_lib::path::{Optimizer, PathResult, State, Trajectory};
use game_lib::Position as Pos;

use rand::{thread_rng, Rng, SeedableRng};
use rand_xorshift::XorShiftRng;

use tcod::console::*;
use tcod::input::{self, Event, Key, Mouse};
use tui::style::{Color, Style};
use tui::Terminal;

mod ui;

/// Screen width in number of vertical columns of text
const SCREEN_WIDTH: u32 = 120;

/// Screen height in number of horizontal rows of text
const SCREEN_HEIGHT: u32 = 80;

const TOP_BAR_HEIGHT: u32 = 2;
const PANEL_HEIGHT: u32 = 10;

// Have the map consume the space not consumed by the GUI
const MAP_WIDTH: u32 = SCREEN_WIDTH;
const MAP_HEIGHT: u32 = SCREEN_HEIGHT - TOP_BAR_HEIGHT - PANEL_HEIGHT - 1;
const MAP_AREA: (i32, i32) = (0, TOP_BAR_HEIGHT as i32);

const COLOR_CANVAS_BG: Color = Color::Rgb(94, 86, 76);

// Color of map tiles
const COLOR_WALL_BG: Color = Color::Rgb(209, 178, 138);
const COLOR_WALL_FG: Color = Color::Rgb(130, 118, 101);
const COLOR_GROUND_FG: Color = Color::Rgb(254, 241, 224);
const COLOR_GROUND_BG: Color = Color::Rgb(246, 230, 206);

// Color of the cursor and other UI elements
const COLOR_CURSOR: Color = Color::Green;
const COLOR_MONSTER: Color = Color::Rgb(44, 200, 247);
const COLOR_PLAYER: Color = Color::Rgb(188, 7, 98);

#[derive(PartialEq, Default)]
struct Cursor {
    mouse: Mouse,
}

impl Cursor {
    pub fn update_mouse(&mut self, m: Mouse) {
        self.mouse = m;
    }

    pub fn draw<C: Console>(&self, _: &mut C, _: &Map) {}

    #[inline]
    pub fn as_tuple(&self) -> (u32, u32) {
        (self.mouse.cx as u32, self.mouse.cy as u32)
    }

    #[inline]
    pub fn as_position(&self) -> Pos {
        Pos::new(self.mouse.cx as u32, self.mouse.cy as u32)
    }
}

impl Into<Pos> for Cursor {
    #[inline]
    fn into(self) -> Pos {
        self.as_position()
    }
}

impl Into<(u32, u32)> for Cursor {
    #[inline]
    fn into(self) -> (u32, u32) {
        self.as_tuple()
    }
}

fn main() {
    let mut root = Root::initializer()
        .font("consolas12x12_gs_tc.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32)
        .title("Pathfinding")
        .init();

    let seed = thread_rng().gen();
    let mut map_rng = XorShiftRng::from_seed(seed);

    let term = slog_term::TermDecorator::new().force_color().build();
    let decorator = slog_term::CompactFormat::new(term).build();
    let drain = std::sync::Mutex::new(decorator).fuse();
    let logger = Logger::root(drain, o!());

    info!(logger, "Starting vis"; "seed" => format!("{:?}", seed));

    tcod::system::set_fps(30);
    tcod::input::show_cursor(true);

    let backend = ui::TCodBackend::new(root, Style::default().bg(COLOR_CANVAS_BG));
    let mut terminal = Terminal::new(backend).unwrap();

    let mut map = generate(&mut map_rng, MAP_WIDTH, MAP_HEIGHT);
    let mut render_map = true;

    let mut cursor: Cursor = Default::default();
    let mut key = Default::default();

    let mut offset = 0.0;
    loop {
        match input::check_for_event(input::MOUSE | input::KEY_PRESS) {
            Some((_, Event::Mouse(m))) => cursor.update_mouse(m),
            Some((_, Event::Key(k))) => key = k,
            _ => key = Default::default(),
        }

        use tcod::input::KeyCode::Escape;
        match key {
            Key { code: Escape, .. } => break,
            _ => (),
        };

        use tui::widgets::*;
        terminal
            .draw(|mut f| {
                let size = f.size();
                offset += 0.025;
                Chart::default()
                    .block(Block::default().title("Chart").borders(Borders::ALL))
                    .x_axis(Axis::default().title("X Axis").bounds([1., 4.]).labels(&[
                        &format!("1"),
                        &format!("2"),
                        &format!("3"),
                    ]))
                    .y_axis(
                        Axis::default()
                            .title("Y Axis")
                            .bounds([-20.0, 20.0])
                            .labels(&["-20", "0", "20"]),
                    )
                    .datasets(&[
                        Dataset::default()
                            .name("data2")
                            .marker(Marker::Dot)
                            .style(Style::default().fg(Color::Magenta))
                            .data(
                                (1..100)
                                    .map(|x| {
                                        let x = x as f64 / 100. * 4.;
                                        (x, 12. * (x + offset).sin())
                                    })
                                    .collect::<Vec<_>>()
                                    .as_slice(),
                            ),
                        Dataset::default()
                            .name("data3")
                            .marker(Marker::Braille)
                            .style(Style::default().fg(Color::Cyan))
                            .data(
                                (1..100)
                                    .map(|x| {
                                        let x = x as f64 / 100. * 4.;
                                        (x, 3. * x.sin() + 12. * (x + offset).cos())
                                    })
                                    .collect::<Vec<_>>()
                                    .as_slice(),
                            ),
                    ])
                    .render(&mut f, size);
            })
            .unwrap();
    }
}
