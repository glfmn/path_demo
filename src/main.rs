extern crate game_lib;
extern crate rand;
extern crate tcod;

mod draw;

use game_lib::actor::Monster;
use game_lib::map::{generate, Map, Tile};
use game_lib::Position;
use rand::thread_rng;
use tcod::colors::{self, Color};
use tcod::console::*;
use tcod::input::{self, Event, Key};

/// Screen width in number of vertical columns of text
const SCREEN_WIDTH: u32 = 100;

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

fn main() {
    let mut root = Root::initializer()
        .font("consolas12x12_gs_tc.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32)
        .title("Pathfinding")
        .init();

    let mut map_layer = Offscreen::new(MAP_WIDTH as i32, MAP_HEIGHT as i32);

    let mut monster: Option<Monster> = None;
    let mut player: Option<Position> = None;

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

        use tcod::input::KeyCode::{Backspace, Escape};
        match key {
            Key { code: Escape, .. } => break,
            Key { code: Backspace, .. } => {
                map = generate(&mut map_rng, MAP_WIDTH, MAP_HEIGHT);
                monster = None;
                player = None;
            }
            _ => (),
        };

        draw_map(&mut root, &mut map_layer, &map);

        if let Some(tile) = map.get(mouse.cx as u32, mouse.cy as u32) {
            root.put_char(mouse.cx as i32, mouse.cy as i32, 'X', BackgroundFlag::None);
            let color =
                if *tile == Tile::FLOOR { COLOR_CURSOR } else { colors::DESATURATED_FLAME };
            root.set_char_foreground(mouse.cx as i32, mouse.cy as i32, color);

            if mouse.lbutton && *tile == Tile::FLOOR {
                monster = if let Some(mut monster) = monster {
                    monster.pos.x = mouse.cx as u32;
                    monster.pos.y = mouse.cy as u32;
                    Some(monster)
                } else {
                    Some(Monster::new(mouse.cx as u32, mouse.cy as u32, 100, 100))
                }
            }

            if mouse.rbutton && *tile == Tile::FLOOR {
                player = if let Some(mut player) = player {
                    player.x = mouse.cx as u32;
                    player.y = mouse.cy as u32;
                    Some(player)
                } else {
                    Some(Position { x: mouse.cx as u32, y: mouse.cy as u32 })
                }
            }
        }

        if let Some(monster) = &monster {
            let (x, y) = (monster.pos.x as i32, monster.pos.y as i32);
            root.put_char(x, y, 'M', BackgroundFlag::None);
            root.set_char_foreground(x, y, COLOR_MONSTER);
        }

        if let Some(player) = &player {
            let (x, y) = (player.x as i32, player.y as i32);
            root.put_char(x, y, '@', BackgroundFlag::None);
            root.set_char_foreground(x, y, COLOR_PLAYER);
        }

        root.flush();
    }
}
