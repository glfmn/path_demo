use game_lib::actor::Actor;
use game_lib::map::{Map, Tile};
use game_lib::Position;
use game_lib::Rect as Area;

use tui::buffer::Buffer;
use tui::layout::{Layout, Rect};
use tui::style::Style;
use tui::widgets::{Block, Widget};

use std::collections::{HashMap, HashSet};

pub struct MapView<'a, F, M>
where
    F: Fn(usize, &Tile) -> (char, Style),
    M: FnMut(Option<Position>),
{
    map: &'a Map,
    map_pos: Position,
    block: Option<Block<'a>>,
    visualization: Option<Visualization>,
    style_fn: F,
    mouse_position: Option<Position>,
    mouse_callback: Option<M>,
    player: Option<(Actor, &'a str, Style)>,
    monster: Option<(Actor, &'a str, Style)>,
}

impl<'a, F, M> MapView<'a, F, M>
where
    F: Fn(usize, &Tile) -> (char, Style),
    M: FnMut(Option<Position>),
{
    pub fn new(map: &'a Map, style_fn: F) -> Self {
        MapView {
            map,
            style_fn,
            block: None,
            map_pos: Position::zero(),
            visualization: None,
            mouse_position: None,
            mouse_callback: None,
            player: None,
            monster: None,
        }
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

    pub fn visualization(mut self, vis: Visualization) -> Self {
        self.visualization = Some(vis);
        self
    }

    pub fn player(mut self, p: Actor, s: &'a str, st: Style) -> Self {
        self.player = Some((p, s, st));
        self
    }

    pub fn monster(mut self, p: Actor, s: &'a str, st: Style) -> Self {
        self.monster = Some((p, s, st));
        self
    }

    pub fn position_callback(mut self, mouse_position: Position, callback: M) -> Self {
        self.mouse_callback = Some(callback);
        self.mouse_position = Some(mouse_position);
        self
    }
}

impl<'a, F, M> Widget for MapView<'a, F, M>
where
    F: Fn(usize, &Tile) -> (char, Style),
    M: FnMut(Option<Position>),
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

        use tui::layout::{Constraint, Direction, Layout};
        use tui::widgets::*;

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5 + 2), Constraint::Min(0)].as_ref())
            .split(map_area);

        let legend_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
            .split(layout[0])[1];

        if let (Some(mouse_pos), Some(callback)) =
            (&self.mouse_position, &mut self.mouse_callback)
        {
            (callback)(
                screen_coord
                    .transform_to_local(mouse_pos)
                    .and_then(|p| map_coord.transform(&p)),
            );
        }

        if let Some((monster, symbol, style)) = &self.monster {
            let pos = monster.pos.clone() - self.map_pos.clone();
            if let Some(Position { x, y }) = screen_coord.transform(&pos) {
                buf.get_mut(x as u16, y as u16).set_symbol(symbol).set_style(*style);
            }
        }

        if let Some((player, symbol, style)) = &self.player {
            let pos = player.pos.clone() - self.map_pos.clone();
            if let Some(Position { x, y }) = screen_coord.transform(&pos) {
                buf.get_mut(x as u16, y as u16).set_symbol(symbol).set_style(*style);
            }
        }

        let mut legend = Block::default().title("Legend").borders(Borders::ALL);
        legend.draw(legend_area, buf);
        let legend_area = legend.inner(legend_area);
        for y in legend_area.top()..legend_area.bottom() {
            buf.get_mut(legend_area.left(), y).set_symbol("#");
        }
    }
}

pub struct Visualization {
    queue: HashMap<Position, usize>,
    visited: HashSet<Position>,
    trajectory: Vec<Position>,
}
