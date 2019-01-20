extern crate game_lib;
extern crate rand;
extern crate tcod;

mod draw;

use game_lib::actor::{Monster, Movement, TurnOptimal, WalkSampler};
use game_lib::map::{generate, Map, Tile};
use game_lib::path::astar::AStar;
use game_lib::path::{Model, Optimizer, PathResult, State, Trajectory};
use game_lib::Position;
use rand::thread_rng;
use tcod::colors::{self, Color};
use tcod::console::*;
use tcod::input::{self, Event, Key, Mouse};

/// Screen width in number of vertical columns of text
const SCREEN_WIDTH: u32 = 120;

/// Screen height in number of horizontal rows of text
const SCREEN_HEIGHT: u32 = 80;

// Have the map consume the space not consumed by the GUI
const MAP_WIDTH: u32 = SCREEN_WIDTH;
const MAP_HEIGHT: u32 = SCREEN_HEIGHT;

// Color of map tiles
const COLOR_WALL_FG: Color = Color { r: 198, g: 197, b: 195 };
const COLOR_WALL_BG: Color = Color { r: 142, g: 139, b: 138 };
const COLOR_GROUND_FG: Color = Color { r: 85, g: 81, b: 79 };
const COLOR_GROUND_BG: Color = Color { r: 28, g: 22, b: 20 };

// Color of the cursor and other UI elements
const COLOR_CURSOR: Color = Color { r: 200, g: 180, b: 50 };
const COLOR_MONSTER: Color = Color { r: 0, g: 223, b: 252 };
// 190,242,2
const COLOR_PLAYER: Color = Color { r: 190, g: 242, b: 2 };

fn draw_map(root: &mut Root, map_layer: &mut Offscreen, map: &Map) {
    map_layer.clear();
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let (char, fg_color, bg_color) = if map[(x, y)].is_wall() {
                let count = map.count_adjacent(x, y, 1, |tile| !tile.is_wall());
                (if count == 0 { ' ' } else { '#' }, COLOR_WALL_FG, COLOR_WALL_BG)
            } else {
                ('.', COLOR_GROUND_FG, COLOR_GROUND_BG)
            };
            map_layer.put_char_ex(x as i32, y as i32, char, fg_color, bg_color);
        }
    }

    blit(
        map_layer,
        (0, 0),
        (SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32),
        root,
        (0, 0),
        1f32,
        1f32,
    );
}

fn draw_vis(
    root: &mut Root,
    vis_layer: &mut Offscreen,
    planner: &AStar<TurnOptimal>,
    trajectory: &Trajectory<TurnOptimal>,
) {
    vis_layer.clear();

    for Position { x, y } in planner.inspect_discovered() {
        vis_layer.put_char_ex(
            *x as i32,
            *y as i32,
            '.',
            colors::DARKER_RED,
            colors::DARKEST_RED,
        )
    }

    for (state, _) in planner.inspect_queue() {
        let Position { x, y } = state.grid_position();
        vis_layer.put_char_ex(
            x as i32,
            y as i32,
            '.',
            colors::DARKER_GREEN,
            colors::DARKEST_GREEN,
        )
    }

    for (state, _) in trajectory.trajectory.iter() {
        let Position { x, y } = state.grid_position();
        vis_layer.put_char_ex(
            x as i32,
            y as i32,
            '+',
            colors::LIGHT_BLUE,
            colors::DARKEST_BLUE,
        );
    }

    vis_layer.set_key_color(colors::BLACK);
    blit(
        vis_layer,
        (0, 0),
        (SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32),
        root,
        (0, 0),
        1f32,
        1f32,
    );
}

fn draw_ui(
    root: &mut Root,
    ui_layer: &mut Offscreen,
    map: &Map,
    mouse: &Mouse,
    player: &Option<Position>,
    monster: &Option<Monster>,
) {
    ui_layer.clear();
    if let Some(tile) = map.get(mouse.cx as u32, mouse.cy as u32) {
        ui_layer.put_char(mouse.cx as i32, mouse.cy as i32, 'X', BackgroundFlag::None);
        let color =
            if *tile == Tile::FLOOR { COLOR_CURSOR } else { colors::DESATURATED_FLAME };
        ui_layer.set_char_foreground(mouse.cx as i32, mouse.cy as i32, color);
    }

    if let Some(monster) = &monster {
        let (x, y) = (monster.pos.x as i32, monster.pos.y as i32);
        ui_layer.put_char(x, y, 'M', BackgroundFlag::None);
        ui_layer.set_char_foreground(x, y, COLOR_MONSTER);
    }

    if let Some(player) = &player {
        let (x, y) = (player.x as i32, player.y as i32);
        ui_layer.put_char(x, y, '@', BackgroundFlag::None);
        ui_layer.set_char_foreground(x, y, COLOR_PLAYER);
    }
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

fn overlaps_player(player: &Option<Position>, mouse: &Mouse) -> bool {
    if let Some(player) = player {
        if player.x == mouse.cx as u32 && player.y == mouse.cy as u32 {
            true
        } else {
            false
        }
    } else {
        false
    }
}

fn overlaps_monster(monster: &Option<Monster>, mouse: &Mouse) -> bool {
    if let Some(monster) = monster {
        if monster.pos.x == mouse.cx as u32 && monster.pos.y == mouse.cy as u32 {
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

    let mut map_layer = Offscreen::new(MAP_WIDTH as i32, MAP_HEIGHT as i32);
    let mut vis_layer = Offscreen::new(MAP_WIDTH as i32, MAP_HEIGHT as i32);
    let mut ui_layer = Offscreen::new(MAP_WIDTH as i32, MAP_HEIGHT as i32);

    let mut monster: Option<Monster> = None;
    let mut player: Option<Position> = None;

    let mut astar = AStar::<TurnOptimal>::new();
    let mut sampler = WalkSampler::new();
    let mut trajectory = Trajectory::<TurnOptimal>::default();
    let mut converged = false;

    tcod::system::set_fps(30);
    tcod::input::show_cursor(false);

    let mut map_rng = thread_rng();
    let mut map = generate(&mut map_rng, MAP_WIDTH, MAP_HEIGHT);

    let mut mouse = Default::default();
    let mut key = Default::default();
    'main_loop: while !root.window_closed() {
        match input::check_for_event(input::MOUSE | input::KEY_PRESS) {
            Some((_, Event::Mouse(m))) => mouse = m,
            Some((_, Event::Key(k))) => key = k,
            _ => key = Default::default(),
        }

        use tcod::input::KeyCode::{Backspace, Delete, Enter, Escape};
        match key {
            Key { code: Escape, .. } => break,
            Key { code: Delete, .. } => {
                astar = AStar::new();
                trajectory = Default::default();
                map = generate(&mut map_rng, MAP_WIDTH, MAP_HEIGHT);
                monster = None;
                player = None;
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
                        }
                        map = model.return_map();
                    }
                }
            }
            _ => (),
        };

        if let Some(tile) = map.get(mouse.cx as u32, mouse.cy as u32) {
            if mouse.lbutton && *tile == Tile::FLOOR && !overlaps_player(&player, &mouse) {
                astar = AStar::new();
                trajectory = Default::default();
                converged = false;
                monster = if let Some(mut monster) = monster {
                    monster.pos.x = mouse.cx as u32;
                    monster.pos.y = mouse.cy as u32;
                    Some(monster)
                } else {
                    Some(Monster::new(mouse.cx as u32, mouse.cy as u32, 100, 100))
                }
            }

            if mouse.rbutton && *tile == Tile::FLOOR && !overlaps_monster(&monster, &mouse) {
                astar = AStar::new();
                trajectory = Default::default();
                converged = false;
                player = if let Some(mut player) = player {
                    player.x = mouse.cx as u32;
                    player.y = mouse.cy as u32;
                    Some(player)
                } else {
                    Some(Position { x: mouse.cx as u32, y: mouse.cy as u32 })
                }
            }
        }

        draw_map(&mut root, &mut map_layer, &map);
        draw_vis(&mut root, &mut vis_layer, &astar, &trajectory);
        draw_ui(&mut root, &mut ui_layer, &map, &mouse, &player, &monster);
        root.flush();
    }
}
