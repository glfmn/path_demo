use game_lib::map::{Map, Tile};

use tui::buffer::Buffer;
use tui::layout::{Layout, Rect};
use tui::style::Style;
use tui::widgets::{Block, Widget};

pub struct MapView<'a, F>
where
    F: Fn(usize, &Tile) -> (char, Style),
{
    map: &'a Map,
    block: Option<Block<'a>>,
    style_fn: F,
}

impl<'a, F> MapView<'a, F>
where
    F: Fn(usize, &Tile) -> (char, Style),
{
    pub fn new(map: &'a Map, style_fn: F) -> Self {
        MapView { map, style_fn, block: None }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn style(mut self, style: F) -> Self {
        self.style_fn = style;
        self
    }
}

impl<'a, F> Widget for MapView<'a, F>
where
    F: Fn(usize, &Tile) -> (char, Style),
{
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        let map_area = match self.block {
            Some(ref mut b) => {
                b.draw(area, buf);
                b.inner(area)
            }
            None => area,
        };

        let mut sym_buff = [0, 0, 0, 0];
        let (w, h) = self.map.dimensions();
        for y in 0..map_area.height.min(h as u16) {
            for x in 0..map_area.width.min(w as u16) {
                let count =
                    self.map.count_adjacent(x as u32, y as u32, 1, |tile| !tile.is_wall());
                let (glyph, style) = (self.style_fn)(count, &self.map[(x as u32, y as u32)]);
                let symbol = glyph.encode_utf8(&mut sym_buff);
                let (x, y) = (map_area.left() + x, map_area.top() + y);
                buf.get_mut(x, y).set_symbol(symbol).set_style(style);
            }
        }
    }
}
