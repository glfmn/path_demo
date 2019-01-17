extern crate game_lib;
extern crate tcod;

mod draw;

use game_lib::map::{generate, Tile};
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
const COLOR_WALL: Color = Color { r: 0, g: 0, b: 100 };
const COLOR_GROUND: Color = Color { r: 50, g: 50, b: 150 };

// Color of the cursor and other UI elements
const COLOR_CURSOR: Color = Color { r: 200, g: 180, b: 50 };

fn main() {
    let mut root = Root::initializer()
        .font("prestige12x12_gs_tc.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32)
        .title("Pathfinding")
        .init();

    let mut map_layer = Offscreen::new(MAP_WIDTH as i32, MAP_HEIGHT as i32);

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

        map_layer.clear();
        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                let color = if map[(x, y)].is_wall() { COLOR_WALL } else { COLOR_GROUND };
                map_layer.set_char_background(x as i32, y as i32, color, BackgroundFlag::Set);
            }
        }

        blit(
            &mut map_layer,
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
