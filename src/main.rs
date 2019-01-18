extern crate game_lib;
extern crate rand;
extern crate tcod;

mod draw;

use game_lib::map::{generate, Map, Tile};
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
const COLOR_GROUND_FG: Color = Color { r: 198, g: 197, b: 195 };
const COLOR_GROUND_BG: Color = Color { r: 233, g: 232, b: 232 };
const COLOR_WALL_FG: Color = Color { r: 85, g: 81, b: 79 };
const COLOR_WALL_BG: Color = Color { r: 28, g: 22, b: 20 };

// Color of the cursor and other UI elements
const COLOR_CURSOR: Color = Color { r: 200, g: 180, b: 50 };

fn draw_map(root: &mut Root, map_layer: &mut Offscreen, map: &Map) {
    map_layer.clear();
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let (char, fg_color, bg_color) = if map[(x, y)].is_wall() {
                ('%', COLOR_WALL_FG, COLOR_WALL_BG)
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
        .font("prestige12x12_gs_tc.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32)
        .title("Pathfinding")
        .init();

    let mut map_layer = Offscreen::new(MAP_WIDTH as i32, MAP_HEIGHT as i32);

    tcod::system::set_fps(30);

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
            Key { code: Backspace, .. } => map = generate(&mut map_rng, MAP_WIDTH, MAP_HEIGHT),
            _ => (),
        };

        draw_map(&mut root, &mut map_layer, &map);

        if let Some(&Tile::FLOOR) = map.get(mouse.cx as u32, mouse.cy as u32) {
            root.put_char(mouse.cx as i32, mouse.cy as i32, 'X', BackgroundFlag::None);
            root.set_char_foreground(mouse.cx as i32, mouse.cy as i32, COLOR_CURSOR);
        }

        root.flush();
    }
}
