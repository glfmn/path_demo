use std::io;

use tcod::chars;
use tcod::console::{Console, Root};
use tcod::{self, colors};

use tui::backend::Backend;
use tui::buffer::Cell;
use tui::layout::Rect;
use tui::style::Color as TuiColor;
use tui::style::Style;

pub struct TCodBackend {
    console: Root,
    default_style: Style,
}

impl TCodBackend {
    pub fn new(mut console: Root) -> Self {
        console.set_default_background(colors::RED);
        TCodBackend { console, default_style: Default::default() }
    }
}

impl Backend for TCodBackend {
    fn draw<'a, I>(&mut self, content: I) -> Result<(), io::Error>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        for (x, y, cell) in content {
            let symbol = match cell.symbol.chars().next().unwrap() {
                '─' => chars::HLINE,
                '│' => chars::VLINE,
                '┘' => chars::SE,
                '┐' => chars::NE,
                '┌' => chars::NW,
                '└' => chars::SW,
                '⢀' | '⠄' | '⠠' | '⡀' => '.',
                '⠐' | '⠈' => '`',
                '⠂' | '⠁' => '`',
                '•' => '*',
                symbol => {
                    if symbol != ' ' {
                        println!("Content");
                        println!("{:?}", cell);
                    }
                    symbol
                }
            };
            self.console.put_char_ex(
                x as i32,
                y as i32,
                symbol,
                tcod::colors::WHITE,
                tcod::colors::BLACK,
            );
        }
        Ok(())
    }

    fn hide_cursor(&mut self) -> Result<(), io::Error> {
        tcod::input::show_cursor(false);

        Ok(())
    }

    fn show_cursor(&mut self) -> Result<(), io::Error> {
        tcod::input::show_cursor(true);

        Ok(())
    }

    fn set_cursor(&mut self, x: u16, y: u16) -> Result<(), io::Error> {
        unimplemented!()
    }

    fn get_cursor(&mut self) -> Result<(u16, u16), io::Error> {
        unimplemented!()
    }

    fn clear(&mut self) -> Result<(), io::Error> {
        self.console.clear();
        Ok(())
    }

    fn size(&self) -> Result<Rect, io::Error> {
        Ok(Rect::new(0, 0, self.console.width() as u16, self.console.height() as u16))
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        self.console.flush();
        Ok(())
    }
}
