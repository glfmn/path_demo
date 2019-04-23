use game_lib::map::{Map, Tile};
use game_lib::Position;
use game_lib::Rect as Area;

use tui::buffer::Buffer;
use tui::layout::{Layout, Rect};
use tui::style::Style;
use tui::widgets::{Block, Widget};

pub struct MapView<'a, F>
where
    F: Fn(usize, &Tile) -> (char, Style),
{
    map: &'a Map,
    map_pos: Position,
    block: Option<Block<'a>>,
    style_fn: F,
}

impl<'a, F> MapView<'a, F>
where
    F: Fn(usize, &Tile) -> (char, Style),
{
    pub fn new(map: &'a Map, style_fn: F) -> Self {
        MapView { map, style_fn, block: None, map_pos: Position::zero() }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn style(mut self, style: F) -> Self {
        self.style_fn = style;
        self
    }

    pub fn map_position(mut self, map_pos: Position) -> Self {
        self.map_pos = map_pos.clone();
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

        let (w, h) = self.map.dimensions();
        let map_coord =
            Area::new(self.map_pos.clone(), map_area.width as u32 - 1, map_area.height as u32);
        let screen_coord = Area::new(
            (map_area.left(), map_area.top()),
            map_area.width as u32,
            map_area.height as u32,
        );

        let mut sym_buff = [0, 0, 0, 0];
        for (pos, tile) in self.map.iter_rect(map_coord.clone()) {
            let count = map_coord
                .transform(&pos)
                .map(|map_pos| {
                    self.map.count_adjacent(map_pos.x, map_pos.y, 1, |tile| !tile.is_wall())
                })
                .unwrap_or(0);
            let (glyph, style) = (self.style_fn)(count, tile);
            let symbol = glyph.encode_utf8(&mut sym_buff);
            if let Some(Position { x, y }) = screen_coord.transform(&pos).into() {
                buf.get_mut(x as u16, y as u16).set_symbol(symbol).set_style(style);
            }
        }
    }
}
