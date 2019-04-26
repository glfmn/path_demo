#![allow(unused)]

use slog::{info, o};
use slog::{Drain, Logger};

use game_lib::actor::{Actor, Heuristic, TeleportSampler, TurnOptimal, WalkSampler};
use game_lib::map::{generate, Map, Tile};
use game_lib::path::astar::{AStar, OptimalAStar};
use game_lib::path::dijkstra::Dijkstra;
use game_lib::path::{Algorithm, HeuristicModel, Optimizer, PathResult, State, Trajectory};
use game_lib::Position as Pos;

use rand::{thread_rng, Rng, SeedableRng};
use rand_xorshift::XorShiftRng;

use tcod::console::*;
use tcod::input::{self, Event, Key, Mouse};
use tui::style::{Color, Style};
use tui::Terminal;

mod ui;

/// Screen width in number of vertical columns of text
const SCREEN_WIDTH: u32 = 125;

/// Screen height in number of horizontal rows of text
const SCREEN_HEIGHT: u32 = 85;

// Have the map consume the space not consumed by the GUI
const MAP_WIDTH: u32 = SCREEN_WIDTH * 2;
const MAP_HEIGHT: u32 = SCREEN_HEIGHT * 2;

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

use crate::ui::widgets::Visualization;

struct Settings {
    items: Vec<(String, &'static Fn(&mut App))>,
    selected: usize,
}

enum Sampler {
    Walk,
    Teleport,
}

impl Sampler {
    fn toggle(&mut self) {
        use Sampler::*;
        *self = match self {
            Walk => Teleport,
            Teleport => Walk,
        }
    }
}

struct App {
    pub map_pos: Pos,
    pub map: Map,
    pub sampler: Sampler,
    pub settings: Settings,
    pub monster: Option<Actor>,
    pub player: Option<Actor>,
    pub algorithm: Algorithm<TurnOptimal>,
    pub trajectory: PathResult<TurnOptimal>,
}

impl Default for App {
    fn default() -> Self {
        let mut map_rng = thread_rng();
        App {
            map_pos: Pos::zero(),
            map: generate(&mut map_rng, MAP_WIDTH, MAP_HEIGHT),
            sampler: Sampler::Walk,
            settings: Settings {
                items: vec![
                    ("Re-Generate Map".to_string(), &|a| {
                        let mut rng = thread_rng();
                        a.clear();
                        a.player = None;
                        a.monster = None;
                        a.map = generate(&mut rng, MAP_WIDTH, MAP_HEIGHT);
                    }),
                    ("Switch Optimizer [A*]".to_string(), &|a| {
                        a.clear();
                        a.algorithm.toggle();
                        let name = match a.algorithm {
                            Algorithm::Dijkstra(_) => "Dijkstra",
                            Algorithm::AStar(_) => "A*",
                            Algorithm::OptimalAStar(_) => "High Performance A*",
                        };
                        a.settings.items[1].0 = format!("Switch Optimizer [{}]", name);
                    }),
                    ("Switch Sampler [Walk]".to_string(), &|a| {
                        a.clear();
                        a.sampler.toggle();
                        let name = match a.sampler {
                            Sampler::Walk => "Walk",
                            Sampler::Teleport => "Teleport",
                        };
                        a.settings.items[2].0 = format!("Switch Sampler [{}]", name);
                    }),
                ],
                selected: 0,
            },
            monster: None,
            player: None,
            algorithm: Algorithm::default(),
            trajectory: PathResult::Intermediate(Trajectory::default()),
        }
    }
}

impl App {
    pub fn settings(&self) -> Vec<&str> {
        self.settings.items.iter().map(|(s, _)| s.as_str()).collect()
    }

    pub fn update_player(&mut self, player: Option<Actor>) {
        if player
            .as_ref()
            .and_then(|p| self.map.pos(&p.pos).map(|t| !t.is_wall()))
            .unwrap_or(false)
        {
            self.clear();
            self.player = player;
        }
    }

    pub fn update_monster(&mut self, monster: Option<Actor>) {
        if monster
            .as_ref()
            .and_then(|p| self.map.pos(&p.pos).map(|t| !t.is_wall()))
            .unwrap_or(false)
        {
            self.clear();
            self.monster = monster;
        }
    }

    pub fn clear(&mut self) {
        self.algorithm.clear();
        self.trajectory = PathResult::Intermediate(Trajectory::default());
    }

    pub fn trajectory(&self) -> Trajectory<TurnOptimal> {
        match &self.trajectory {
            PathResult::Intermediate(t) => t.clone(),
            PathResult::Final(t) => t.clone(),
            _ => Trajectory::default(),
        }
    }

    pub fn visualization(&self) -> Visualization {
        Visualization {
            queue: self.algorithm.inspect_queue().map(|(s, _)| (s.pos.clone(), 0)).collect(),
            visited: self.algorithm.inspect_discovered().cloned().collect(),
            trajectory: self
                .trajectory()
                .trajectory
                .iter()
                .map(|(s, _)| s.pos.clone())
                .collect(),
        }
    }

    pub fn step(mut self) -> Self {
        if let (Some(ref player), Some(ref monster)) = (&self.player, &self.monster) {
            if let PathResult::Intermediate(_) = &self.trajectory {
                let mut model = TurnOptimal::new(self.map);
                model.set_heuristic(Heuristic::Diagonal);
                let mut goal = player.clone();
                match self.sampler {
                    Sampler::Walk => {
                        let mut sampler = WalkSampler::new();
                        self.trajectory = self.algorithm.next_trajectory(
                            &mut model,
                            &monster,
                            &goal,
                            &mut sampler,
                        );
                    }
                    Sampler::Teleport => {
                        let mut sampler = TeleportSampler::new();
                        self.trajectory = self.algorithm.next_trajectory(
                            &mut model,
                            &monster,
                            &goal,
                            &mut sampler,
                        );
                    }
                };
                self.map = model.return_map();
            }
        }

        self
    }

    pub fn complete_plan(mut self) -> Self {
        if let (Some(ref player), Some(ref monster)) = (&self.player, &self.monster) {
            if let PathResult::Intermediate(_) = &self.trajectory {
                let mut model = TurnOptimal::new(self.map);
                model.set_heuristic(Heuristic::Diagonal);
                let mut goal = player.clone();
                match self.sampler {
                    Sampler::Walk => {
                        let mut sampler = WalkSampler::new();
                        self.trajectory =
                            self.algorithm.optimize(&mut model, &monster, &goal, &mut sampler);
                    }
                    Sampler::Teleport => {
                        let mut sampler = TeleportSampler::new();
                        self.trajectory =
                            self.algorithm.optimize(&mut model, &monster, &goal, &mut sampler);
                    }
                };
                self.map = model.return_map();
            }
        }

        self
    }

    pub fn update(mut self, event: Key) -> Self {
        use tcod::input::KeyCode::*;

        match event {
            Key { code: Right, .. } => self.map_pos.x = self.map_pos.x + 1,
            Key { code: Left, .. } => self.map_pos.x = self.map_pos.x.max(1) - 1,
            Key { code: Up, .. } => self.map_pos.y = self.map_pos.y.max(1) - 1,
            Key { code: Down, .. } => self.map_pos.y = self.map_pos.y + 1,
            Key { code: PageUp, .. } => {
                self.settings.selected =
                    (self.settings.selected - 1) % self.settings.items.len()
            }
            Key { code: PageDown, .. } | Key { code: Tab, .. } => {
                self.settings.selected =
                    (self.settings.selected + 1) % self.settings.items.len()
            }
            Key { code: Enter, shift: true, .. } => self = self.complete_plan(),
            Key { code: Enter, .. } => self = self.step(),
            Key { code: Backspace, .. } => self.clear(),
            Key { code: Spacebar, .. } => {
                (self.settings.items[self.settings.selected].1)(&mut self)
            }
            _ => (),
        };

        self
    }
}

#[derive(PartialEq, Default, Clone)]
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

fn style_map(count: usize, tile: &Tile) -> (char, Style) {
    if count == 0 {
        (' ', Style::default())
    } else {
        if tile.is_wall() {
            ('#', Style::default().fg(COLOR_WALL_FG).bg(COLOR_WALL_BG))
        } else {
            ('.', Style::default().fg(COLOR_GROUND_FG).bg(COLOR_GROUND_BG))
        }
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

    let mut cursor: Cursor = Default::default();
    let mut key = Default::default();

    let mut app = App::default();

    loop {
        match input::check_for_event(input::MOUSE | input::KEY_PRESS) {
            Some((_, Event::Mouse(m))) => cursor.update_mouse(m),
            Some((_, Event::Key(k))) => key = k,
            _ => key = Default::default(),
        }

        terminal
            .draw(|mut f| {
                use crate::ui::widgets::MapView;

                use tui::layout::{Constraint, Direction, Layout};
                use tui::widgets::*;

                let size = f.size();

                let layout = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([Constraint::Length(2), Constraint::Min(80)].as_ref())
                    .split(size);

                let map_layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Min(70), Constraint::Length(32)].as_ref())
                    .split(layout[1]);
                Block::default()
                    .title("Path-finding Visualization")
                    .borders(Borders::TOP)
                    .render(&mut f, layout[0]);

                let mut player = None;
                let mut monster = None;
                let mut map_view = MapView::new(&app.map, style_map)
                    .block(Block::default().title("Map").borders(Borders::ALL))
                    .map_position(app.map_pos.clone())
                    .trajectory_style(Style::default().fg(Color::Cyan).bg(Color::LightBlue))
                    .visited_style(Style::default().fg(Color::Red).bg(COLOR_GROUND_BG))
                    .queue_style(Style::default().fg(Color::Green).bg(COLOR_GROUND_BG))
                    .visualization(app.visualization());

                if let Some(player) = &app.player {
                    map_view = map_view.player(
                        player.clone(),
                        "@",
                        Style::default().fg(COLOR_PLAYER).bg(COLOR_GROUND_BG),
                    );
                }

                if let Some(monster) = &app.monster {
                    map_view = map_view.monster(
                        monster.clone(),
                        "M",
                        Style::default().fg(COLOR_MONSTER).bg(COLOR_GROUND_BG),
                    );
                }

                map_view
                    .position_callback(cursor.clone().into(), |p| {
                        if cursor.mouse.lbutton {
                            player = p.map(|Pos { x, y }| Actor::new(x, y, 0, 20));
                        } else if cursor.mouse.rbutton {
                            monster = p.map(|Pos { x, y }| Actor::new(x, y, 0, 10));
                        }
                    })
                    .render(&mut f, map_layout[0]);

                app.update_player(player);
                app.update_monster(monster);

                let right_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        [
                            Constraint::Length(app.settings.items.len() as u16 + 4),
                            Constraint::Min(10),
                        ]
                        .as_ref(),
                    )
                    .split(map_layout[1]);

                SelectableList::default()
                    .block(Block::default().title("Settings").borders(Borders::ALL))
                    .items(&app.settings())
                    .select(Some(app.settings.selected))
                    .highlight_style(Style::default().fg(Color::Yellow))
                    .highlight_symbol(">")
                    .render(&mut f, right_layout[0]);

                Table::new(
                    ["Position", "Mana", "Action"].into_iter(),
                    app.trajectory().trajectory.iter().map(|(m, a)| {
                        Row::Data(
                            vec![
                                format!("({:3},{:3})", &m.pos.x, &m.pos.y),
                                format!("{:2}/{}", &m.mana, &m.max_mana),
                                format!("{:?}", &a),
                            ]
                            .into_iter(),
                        )
                    }),
                )
                .widths(&[9, 5, 12])
                .header_style(Style::default().fg(Color::Yellow))
                //.column_spacing(2)
                .block(Block::default().title("Trajectory").borders(Borders::ALL))
                .render(&mut f, right_layout[1]);
            })
            .unwrap();

        use tcod::input::KeyCode::Escape;
        match key {
            Key { code: Escape, .. } => break,
            key => app = app.update(key),
        };
    }
}
