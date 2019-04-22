use std::io;

use tcod::chars;
use tcod::console::{BackgroundFlag, Console, Root};
use tcod::{self, colors};

use tui::backend::Backend;
use tui::buffer::Cell;
use tui::layout::Rect;
use tui::style::Style;

use tcod::colors::Color as TCodColor;
use tui::style::Color as TuiColor;

/// Use `tcod-rs` as backend for terminal UI
///
/// Since `tcod-rs` is _not_ a true text terminal emulator, it does not support true
/// unicode and all of the expected characters from tui.  Efforts have been taken
/// to support as many features as possible by finding the closest possible
/// replacement character that `tcod-rs` supports.
pub struct TCodBackend {
    console: Root,
    reset_colors: (TCodColor, TCodColor),
}

impl TCodBackend {
    /// Create a new backend with the specified foreground and background style
    pub fn new(mut console: Root, style: Style) -> Self {
        let (fg, bg) = (
            tui_to_tcod_color(style.fg, colors::WHITE),
            tui_to_tcod_color(style.bg, colors::BLACK),
        );
        console.set_default_background(bg);
        console.rect(0, 0, console.width(), console.height(), true, BackgroundFlag::Set);

        TCodBackend { console, reset_colors: (fg, bg) }
    }

    /// Change the foreground and background colors
    pub fn style(mut self, style: Style) -> Self {
        let (fg, bg) = (
            tui_to_tcod_color(style.fg, colors::WHITE),
            tui_to_tcod_color(style.bg, colors::BLACK),
        );
        self.reset_colors = (fg, bg);
        self.console.set_default_background(bg);
        let width = self.console.width();
        let height = self.console.height();
        self.console.rect(0, 0, width, height, true, BackgroundFlag::Set);
        self
    }
}

impl Backend for TCodBackend {
    fn draw<'a, I>(&mut self, content: I) -> Result<(), io::Error>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        for (x, y, cell) in content {
            let symbol = match cell.symbol.as_str() {
                "─" => chars::HLINE,
                "│" => chars::VLINE,
                "┘" => chars::SE,
                "┐" => chars::NE,
                "┌" => chars::NW,
                "└" => chars::SW,
                "⢀" | "⠄" | "⠠" | "⡀" => '.',
                "⠐" | "⠈" => '`',
                "⠂" | "⠁" => '`',
                "•" => '*',
                "▃" => chars::BLOCK1,
                "▄" => chars::BLOCK2,
                "█" => chars::BLOCK3,
                symbol => {
                    if symbol != " " {
                        println!("Content");
                        println!("{:?}", cell);
                    }
                    symbol.chars().next().unwrap()
                }
            };
            let fg = tui_to_tcod_color(cell.style.fg, self.reset_colors.0);
            let bg = tui_to_tcod_color(cell.style.bg, self.reset_colors.1);
            self.console.put_char_ex(x as i32, y as i32, symbol, fg, bg);
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
        let width = self.console.width();
        let height = self.console.height();
        self.console.rect(0, 0, width, height, false, BackgroundFlag::Set);
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

fn tui_to_tcod_color(color: TuiColor, default: TCodColor) -> TCodColor {
    use TuiColor::*;
    match color {
        Reset => default,
        Indexed(_) => unimplemented!("No support for indexed color {:?}", color),
        Black => colors::BLACK,
        Red => colors::RED,
        Green => colors::GREEN,
        Yellow => colors::YELLOW,
        Blue => colors::BLUE,
        Magenta => colors::MAGENTA,
        Cyan => colors::CYAN,
        Gray => colors::GREY,
        DarkGray => colors::DARK_GREY,
        LightRed => colors::LIGHT_RED,
        LightGreen => colors::LIGHT_GREEN,
        LightYellow => colors::LIGHT_YELLOW,
        LightBlue => colors::LIGHT_BLUE,
        LightMagenta => colors::LIGHT_MAGENTA,
        LightCyan => colors::LIGHT_CYAN,
        White => colors::WHITE,
        Rgb(r, g, b) => TCodColor { r, g, b },
    }
}
