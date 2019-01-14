extern crate game_lib;
extern crate tcod;

mod draw;

use game_lib::map::{generate, Map, Tile};
use tcod::colors::{self, Color};
use tcod::console::*;
use tcod::input::{self, Event, Key, Mouse};

/// Screen width in number of vertical columns of text
const SCREEN_WIDTH: u32 = 100;

/// Screen height in number of horizontal rows of text
const SCREEN_HEIGHT: u32 = 80;

// Have the map consume the space not consumed by the GUI
const MAP_WIDTH: u32 = SCREEN_WIDTH;
const MAP_HEIGHT: u32 = SCREEN_HEIGHT;

/// Colors of walls in the game, contrasting lit with unlit
const COLOR_LIGHT_WALL: Color = Color {
    r: 130,
    g: 110,
    b: 50,
};
const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };

/// Ground color in the game, contrasting lit with unlit
const COLOR_LIGHT_GROUND: Color = Color {
    r: 200,
    g: 180,
    b: 50,
};
const COLOR_DARK_GROUND: Color = Color {
    r: 50,
    g: 50,
    b: 150,
};

fn main() {
    let mut root = Root::initializer()
        .font("prestige12x12_gs_tc.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32)
        .title("Rust/libtcod tutorial")
        .init();

    let mut map_console = Offscreen::new(MAP_WIDTH as i32, MAP_HEIGHT as i32);

    tcod::system::set_fps(30);

    let mut map = generate(MAP_WIDTH, MAP_HEIGHT);

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
            Key { code: Backspace, .. } => map = generate(MAP_WIDTH, MAP_HEIGHT),
            _ => (),
        };

        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                let color = if map[(x, y)].is_wall() {
                    COLOR_DARK_WALL
                } else {
                    COLOR_DARK_GROUND
                };
                map_console.set_char_background(x as i32, y as i32, color, BackgroundFlag::Set);
            }
        }

        blit(
            &mut map_console,
            (0, 0),
            (SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32),
            &mut root,
            (0, 0),
            1f32,
            1f32,
        );

        root.flush();
    }
}
