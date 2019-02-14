extern crate game_lib;
extern crate rand;
extern crate tcod;

#[macro_use]
extern crate slog;
extern crate slog_term;

use slog::{Drain, Logger};

mod draw;

use game_lib::actor::{Actor, Heuristic, TurnOptimal, WalkSampler};
use game_lib::map::{generate, Map, Tile};
use game_lib::path::astar::AStar;
use game_lib::path::{Optimizer, PathResult, State, Trajectory};
use game_lib::Position;

use rand::thread_rng;

use tcod::colors::{self, Color};
use tcod::console::*;
use tcod::input::{self, Event, Key, Mouse};

/// Screen width in number of vertical columns of text
const SCREEN_WIDTH: u32 = 120;

/// Screen height in number of horizontal rows of text
const SCREEN_HEIGHT: u32 = 80;

const TOP_BAR_HEIGHT: u32 = 2;
const PANEL_HEIGHT: u32 = 10;

// Have the map consume the space not consumed by the GUI
const MAP_WIDTH: u32 = SCREEN_WIDTH;
const MAP_HEIGHT: u32 = SCREEN_HEIGHT - TOP_BAR_HEIGHT - PANEL_HEIGHT;
const MAP_AREA: (i32, i32) = (0, TOP_BAR_HEIGHT as i32);

const COLOR_CANVAS_BG: Color = Color { r: 94, g: 86, b: 76 };

// Color of map tiles
const COLOR_WALL_BG: Color = Color { r: 209, g: 178, b: 138 };
const COLOR_WALL_FG: Color = Color { r: 130, g: 118, b: 101 };
const COLOR_GROUND_FG: Color = Color { r: 254, g: 241, b: 224 };
const COLOR_GROUND_BG: Color = Color { r: 246, g: 230, b: 206 };

// Color of the cursor and other UI elements
const COLOR_CURSOR: Color = colors::DARK_GREEN;
const COLOR_MONSTER: Color = Color { r: 44, g: 200, b: 247 };
const COLOR_PLAYER: Color = Color { r: 188, g: 7, b: 98 };

fn draw_map(root: &mut Root, map_layer: &mut Offscreen, map: &Map) {
    map_layer.clear();
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let (char, fg_color, bg_color) = if map[(x, y)].is_wall() {
                let count = map.count_adjacent(x, y, 1, |tile| !tile.is_wall());
                if count == 0 {
                    (' ', COLOR_CANVAS_BG, COLOR_CANVAS_BG)
                } else {
                    ('#', COLOR_WALL_FG, COLOR_WALL_BG)
                }
            } else {
                ('.', COLOR_GROUND_FG, COLOR_GROUND_BG)
            };
            map_layer.put_char_ex(x as i32, y as i32, char, fg_color, bg_color);
        }
    }

    blit(map_layer, (0, 0), (MAP_WIDTH as i32, MAP_HEIGHT as i32), root, MAP_AREA, 1f32, 1f32);
}

fn draw_vis(
    root: &mut Root,
    vis_layer: &mut Offscreen,
    planner: &AStar<TurnOptimal>,
    trajectory: &Trajectory<TurnOptimal>,
) {
    vis_layer.clear();

    for Position { x, y } in planner.inspect_discovered() {
        vis_layer.put_char_ex(*x as i32, *y as i32, 177 as char, colors::RED, COLOR_GROUND_BG);
    }

    for (state, _) in planner.inspect_queue() {
        let Position { x, y } = state.grid_position();
        vis_layer.put_char_ex(x as i32, y as i32, 178 as char, colors::GREEN, COLOR_GROUND_BG);
    }

    for (state, _) in trajectory.trajectory.iter() {
        let Position { x, y } = state.grid_position();
        vis_layer.put_char_ex(x as i32, y as i32, '+', colors::LIGHT_SKY, colors::BLUE);
    }

    vis_layer.set_key_color(colors::BLACK);
    blit(vis_layer, (0, 0), (MAP_WIDTH as i32, MAP_HEIGHT as i32), root, MAP_AREA, 1f32, 1f32);
}

fn draw_agents(
    root: &mut Root,
    agent_layer: &mut Offscreen,
    player: &Option<Position>,
    monster: &Option<Actor>,
) {
    agent_layer.clear();

    if let Some(player) = &player {
        let (x, y) = (player.x as i32, player.y as i32);
        agent_layer.set_default_foreground(COLOR_PLAYER);
        agent_layer.put_char(x, y, '@', BackgroundFlag::None);
        agent_layer.horizontal_line(x + 1, y, 1, BackgroundFlag::Add);
        agent_layer.horizontal_line(x - 1, y, 1, BackgroundFlag::Add);
        agent_layer.vertical_line(x, y - 1, 1, BackgroundFlag::Add);
        agent_layer.vertical_line(x, y + 1, 1, BackgroundFlag::Add);
    }

    if let Some(monster) = &monster {
        let (x, y) = (monster.pos.x as i32, monster.pos.y as i32);
        agent_layer.set_default_foreground(COLOR_MONSTER);
        agent_layer.put_char(x, y, 'M', BackgroundFlag::None);
    }

    blit(
        agent_layer,
        (0, 0),
        (MAP_WIDTH as i32, MAP_HEIGHT as i32),
        root,
        MAP_AREA,
        1f32,
        0f32,
    );
}

fn draw_ui(
    root: &mut Root,
    ui_layer: &mut Offscreen,
    map: &Map,
    mouse: &Mouse,
    header: &String,
) {
    use tcod::console::TextAlignment;
    ui_layer.clear();
    ui_layer.set_default_foreground(COLOR_GROUND_FG);
    let color = if mouse.cy >= MAP_AREA.1 as isize {
        if let Some(tile) = map.get(
            (mouse.cx - MAP_AREA.0 as isize) as u32,
            (mouse.cy - MAP_AREA.1 as isize) as u32,
        ) {
            if *tile == Tile::FLOOR {
                COLOR_CURSOR
            } else {
                colors::RED
            }
        } else {
            colors::WHITE
        }
    } else {
        colors::WHITE
    };
    ui_layer.put_char(mouse.cx as i32, mouse.cy as i32, 'X', BackgroundFlag::Screen);
    ui_layer.set_char_foreground(mouse.cx as i32, mouse.cy as i32, color);
    ui_layer.set_alignment(TextAlignment::Center);
    ui_layer.print((SCREEN_WIDTH / 2) as i32, 0, header);
    ui_layer.set_alignment(TextAlignment::Left);
    blit(
        ui_layer,
        (0, 0),
        (SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32),
        root,
        (0, 0),
        1f32,
        0f32,
    );
}

fn overlaps_position(player: &Option<Position>, mouse: &Position) -> bool {
    if let Some(player) = player {
        if player.x == mouse.x && player.y == mouse.y {
            true
        } else {
            false
        }
    } else {
        false
    }
}

fn main() {
    let mut root = Root::initializer()
        .font("consolas12x12_gs_tc.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32)
        .title("Pathfinding")
        .init();

    let mut map_rng = thread_rng();

    let term = slog_term::TermDecorator::new().force_color().build();
    let decorator = slog_term::CompactFormat::new(term).build();
    let drain = std::sync::Mutex::new(decorator).fuse();
    let logger = Logger::root(drain, o!());

    println!("\nVisulaizaiton controls:\n");
    println!("   ESC - quit");
    println!("   LEFT CLICK - place monster");
    println!("   RIGHT CLICK - place player/goal");
    println!("   ENTER - step forward once in path-finding");
    println!("   SHIFT + ENTER - plan all the way to the goal");
    println!("   BACKSPACE - restart planning");
    println!("   DELETE - generate a new map");
    println!("   F1 - toggle heuristic functions");

    info!(logger, "Starting vis");

    let mut map_layer = Offscreen::new(MAP_WIDTH as i32, MAP_HEIGHT as i32);
    let mut vis_layer = Offscreen::new(MAP_WIDTH as i32, MAP_HEIGHT as i32);
    let mut agent_layer = Offscreen::new(MAP_WIDTH as i32, MAP_HEIGHT as i32);
    let mut ui_layer = Offscreen::new(SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32);

    let mut monster: Option<Actor> = None;
    let mut player: Option<Position> = None;

    let mut astar = AStar::<TurnOptimal>::new();
    let mut sampler = WalkSampler::new();
    let mut trajectory = Trajectory::<TurnOptimal>::default();
    let mut converged = false;
    let mut heuristic = Heuristic::Manhattan;

    tcod::system::set_fps(30);
    tcod::input::show_cursor(false);

    let mut map = generate(&mut map_rng, MAP_WIDTH, MAP_HEIGHT);
    let mut render_map = true;

    let mut mouse = Default::default();
    let mut key = Default::default();
    'main_loop: while !root.window_closed() {
        match input::check_for_event(input::MOUSE | input::KEY_PRESS) {
            Some((_, Event::Mouse(m))) => mouse = m,
            Some((_, Event::Key(k))) => key = k,
            _ => key = Default::default(),
        }

        use tcod::input::KeyCode::{Backspace, Delete, Enter, Escape, F1, F2};
        match key {
            Key { code: Escape, .. } => break,
            Key { code: Delete, .. } => {
                astar = AStar::new();
                trajectory = Default::default();
                map = generate(&mut map_rng, MAP_WIDTH, MAP_HEIGHT);
                render_map = true;
                monster = None;
                player = None;
                info!(logger, "New map generated");
            }
            Key { code: Backspace, .. } => {
                astar = AStar::new();
                trajectory = Default::default();
                converged = false;
            }
            Key { code: Enter, shift, .. } => {
                if !converged {
                    if let (Some(player), Some(monster)) = (&player, &monster) {
                        let mut model = TurnOptimal::new(map);
                        model.set_heuristic(heuristic.clone());
                        let mut goal = monster.clone();
                        goal.pos = player.clone();
                        let result = if shift {
                            astar.optimize(&mut model, monster.clone(), goal, &mut sampler)
                        } else {
                            astar.next_trajectory(&mut model, &monster, &goal, &mut sampler)
                        };
                        if let PathResult::Intermediate(traj) = result {
                            trajectory = traj;
                        } else if let PathResult::Final(traj) = result {
                            trajectory = traj;
                            converged = true;
                            info!(
                                logger,
                                "Converged";
                                "heuristic" => format!("{}", heuristic),
                                "cost" => trajectory.cost,
                            );
                        }
                        map = model.return_map();
                    }
                }
            }
            Key { code: F1, .. } => {
                astar = AStar::new();
                trajectory = Default::default();
                converged = false;
                match &heuristic {
                    &Heuristic::Chebyshev => heuristic = Heuristic::Manhattan,
                    &Heuristic::Manhattan => heuristic = Heuristic::DoubleManhattan,
                    &Heuristic::DoubleManhattan => heuristic = Heuristic::Chebyshev,
                }
            }
            Key { code: F2, .. } => {
                render_map = !render_map;
            }
            _ => (),
        };

        let mouse_pos = Position::new(
            (mouse.cx - MAP_AREA.0 as isize) as u32,
            (mouse.cy - MAP_AREA.1 as isize) as u32,
        );
        if let Some(tile) = map.get(mouse_pos.x, mouse_pos.y) {
            if *tile == Tile::FLOOR {
                if mouse.lbutton && !overlaps_position(&player, &mouse_pos) {
                    astar = AStar::new();
                    trajectory = Default::default();
                    converged = false;
                    monster = if let Some(mut monster) = monster {
                        monster.pos.x = mouse_pos.x;
                        monster.pos.y = mouse_pos.y;
                        Some(monster)
                    } else {
                        Some(Actor::new(mouse_pos.x, mouse_pos.y, 100, 100))
                    }
                }

                if mouse.rbutton
                    && !overlaps_position(&monster.clone().map(|m| m.pos), &mouse_pos)
                {
                    astar = AStar::new();
                    trajectory = Default::default();
                    converged = false;
                    player = if let Some(mut player) = player {
                        player.x = mouse.cx as u32;
                        player.y = mouse_pos.y;
                        Some(player)
                    } else {
                        Some(Position { x: mouse_pos.x, y: mouse_pos.y })
                    }
                }
            }
        }

        let header = if player.is_none() || monster.is_none() {
            "L-Click to place a monster R-Click to place a goal".into()
        } else {
            format!("Trajectory of cost {} with {} heuristic", trajectory.cost, heuristic)
        };

        root.clear();
        root.set_default_background(COLOR_CANVAS_BG);
        if render_map {
            draw_map(&mut root, &mut map_layer, &map);
        }
        draw_vis(&mut root, &mut vis_layer, &astar, &trajectory);
        draw_ui(&mut root, &mut ui_layer, &map, &mouse, &header);
        draw_agents(&mut root, &mut agent_layer, &player, &monster);
        root.flush();
    }
}
