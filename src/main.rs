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

impl Into<(u32, u32)> for Cursor {
    #[inline]
    fn into(self) -> (u32, u32) {
        self.as_tuple()
    }
}

use crate::ui::widgets::Visualization;

struct Settings {
    tabs: Vec<String>,
    selected: usize,
}

// pub enum AppState {
//     Planning {},
// }

struct App {
    // pub state: AppState,
    pub map_pos: Pos,
    pub map: Map,
    pub settings: Settings,
    pub monster: Option<Actor>,
    pub player: Option<Actor>,
    pub astar: AStar<TurnOptimal>,
    pub trajectory: PathResult<TurnOptimal>,
}

impl App {
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
        self.astar.clear();
        self.trajectory = PathResult::Intermediate(Trajectory::default());
    }

    pub fn trajectory(&self) -> Vec<Pos> {
        match &self.trajectory {
            PathResult::Intermediate(t) => t,
            PathResult::Final(t) => t,
            _ => return Vec::new(),
        }
        .trajectory
        .iter()
        .map(|(s, _)| s.pos.clone())
        .collect()
    }

    pub fn visualization(&self) -> Visualization {
        Visualization {
            queue: self.astar.inspect_queue().map(|(s, _)| (s.pos.clone(), 0)).collect(),
            visited: self.astar.inspect_discovered().cloned().collect(),
            trajectory: self.trajectory(),
        }
    }

    pub fn step(mut self) -> Self {
        if let (Some(ref player), Some(ref monster)) = (&self.player, &self.monster) {
            if let PathResult::Intermediate(_) = &self.trajectory {
                let mut model = TurnOptimal::new(self.map);
                model.set_heuristic(Heuristic::Diagonal);
                let mut goal = player.clone();
                let mut sampler = WalkSampler::new();
                self.trajectory =
                    self.astar.next_trajectory(&mut model, &monster, &goal, &mut sampler);
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
                let mut sampler = WalkSampler::new();
                self.trajectory =
                    self.astar.next_trajectory(&mut model, &monster, &goal, &mut sampler);
                self.map = model.return_map();
            }
        }

        self
    }

    pub fn update(mut self, event: Key) -> Self {
        use tcod::input::KeyCode::{Down, Enter, Left, Right, Tab, Up};

        match event {
            Key { code: Right, .. } => self.map_pos.x = self.map_pos.x + 1,
            Key { code: Left, .. } => self.map_pos.x = self.map_pos.x.max(1) - 1,
            Key { code: Up, .. } => self.map_pos.y = self.map_pos.y.max(1) - 1,
            Key { code: Down, .. } => self.map_pos.y = self.map_pos.y + 1,
            Key { code: Tab, .. } => self.settings.selected = (self.settings.selected + 1) % 3,
            Key { code: Enter, .. } => self = self.step(),
            Key { code: Enter, shift: true, .. } => self = self.complete_plan(),
            _ => (),
        };

        self
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

    let mut app = App {
        map_pos: Pos::zero(),
        map: generate(&mut map_rng, MAP_WIDTH, MAP_HEIGHT),
        settings: Settings {
            tabs: vec!["Visualization".to_string(), "Model".to_string(), "Map".to_string()],
            selected: 0,
        },
        monster: None,
        player: None,
        astar: AStar::default(),
        trajectory: PathResult::Intermediate(Trajectory::default()),
    };

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
                    .constraints(
                        [Constraint::Length(2), Constraint::Min(80), Constraint::Min(0)]
                            .as_ref(),
                    )
                    .split(size);

                let map_layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(
                        [Constraint::Percentage(70), Constraint::Percentage(30)].as_ref(),
                    )
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
                            player = p.map(|Pos { x, y }| Actor::new(x, y, 100, 100));
                        } else if cursor.mouse.rbutton {
                            monster = p.map(|Pos { x, y }| Actor::new(x, y, 0, 0));
                        }
                    })
                    .render(&mut f, map_layout[0]);

                app.update_player(player);
                app.update_monster(monster);

                let mut settings = Block::default().title("Settings").borders(Borders::ALL);
                let settings_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(2), Constraint::Min(0)].as_ref())
                    .split(settings.inner(map_layout[1]));
                settings.render(&mut f, map_layout[1]);

                Tabs::default()
                    .titles(&app.settings.tabs)
                    .select(app.settings.selected)
                    .block(Block::default().borders(Borders::BOTTOM))
                    .highlight_style(Style::default().fg(Color::Yellow))
                    .render(&mut f, settings_layout[0]);

                Block::default().title("Log").borders(Borders::ALL).render(&mut f, layout[2]);
            })
            .unwrap();

        use tcod::input::KeyCode::Escape;
        match key {
            Key { code: Escape, .. } => break,
            key => app = app.update(key),
        };
    }
}
