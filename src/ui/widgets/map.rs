use game_lib::actor::Actor;
use game_lib::map::{Map, Tile};
use game_lib::Position;
use game_lib::Rect as Area;

use tui::buffer::Buffer;
use tui::layout::Rect;
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
    visited_style: Option<Style>,
    queue_style: Option<Style>,
    trajectory_style: Option<Style>,
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
            visited_style: None,
            queue_style: None,
            trajectory_style: None,
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
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
    pub fn visited_style(mut self, s: Style) -> Self {
        self.visited_style = Some(s);
        self
    }

    pub fn queue_style(mut self, s: Style) -> Self {
        self.queue_style = Some(s);
        self
    }

    pub fn trajectory_style(mut self, s: Style) -> Self {
        self.trajectory_style = Some(s);
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

        let map_coord = Area::new(
            self.map_pos.clone(),
            u32::from(map_area.width) - 1,
            u32::from(map_area.height),
        );
        let screen_coord = Area::new(
            (map_area.left(), map_area.top()),
            u32::from(map_area.width),
            u32::from(map_area.height),
        );

        let mut sym_buff = [0, 0, 0, 0];
        let mut legend_entries: Vec<(String, Style, String)> =
            vec![(Tile::WALL, "wall".to_string()), (Tile::FLOOR, "floor".to_string())]
                .iter()
                .map(|(tile, name)| {
                    let (glyph, sty) = (self.style_fn)(1, &tile);
                    let sym = glyph.encode_utf8(&mut sym_buff).to_string();
                    (sym, sty, name.clone())
                })
                .collect();

        for (pos, tile) in self.map.iter_rect(map_coord.clone()) {
            let count = map_coord
                .transform(&pos)
                .map(|map_pos| {
                    self.map.count_adjacent(map_pos.x, map_pos.y, 1, |tile| !tile.is_wall())
                })
                .unwrap_or(0);
            let (glyph, style) = (self.style_fn)(count, tile);
            let symbol = glyph.encode_utf8(&mut sym_buff);
            if let Some(Position { x, y }) = screen_coord.transform(&pos) {
                buf.get_mut(x as u16, y as u16).set_symbol(symbol).set_style(style);
            }
        }

        if let Some(Visualization { queue, visited, trajectory }) = &self.visualization {
            use tui::symbols;
            let style = &self.visited_style.unwrap_or_default();
            legend_entries.push((
                symbols::bar::HALF.to_string(),
                *style,
                "visited".to_string(),
            ));
            for pos in visited.iter() {
                if let Some(Position { x, y }) = map_coord
                    .transform_to_local(pos)
                    .and_then(|pos| screen_coord.transform(&pos))
                {
                    buf.get_mut(x as u16, y as u16)
                        .set_symbol(symbols::bar::HALF)
                        .set_style(*style);
                }
            }

            let style = &self.queue_style.unwrap_or_default();
            legend_entries.push((
                symbols::bar::HALF.to_string(),
                *style,
                "queued".to_string(),
            ));
            for pos in queue.keys() {
                if let Some(Position { x, y }) = map_coord
                    .transform_to_local(pos)
                    .and_then(|pos| screen_coord.transform(&pos))
                {
                    buf.get_mut(x as u16, y as u16)
                        .set_symbol(symbols::bar::HALF)
                        .set_style(*style);
                }
            }

            let style = &self.trajectory_style.unwrap_or_default();
            legend_entries.push(("+".to_string(), *style, "trajectory".to_string()));
            for pos in trajectory.iter() {
                if let Some(Position { x, y }) = map_coord
                    .transform_to_local(pos)
                    .and_then(|pos| screen_coord.transform(&pos))
                {
                    buf.get_mut(x as u16, y as u16).set_symbol("+").set_style(*style);
                }
            }
        }

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
            legend_entries.push((
                symbol.to_string(),
                *style,
                format!("start {},{}", monster.pos.x, monster.pos.y),
            ));
            let pos = map_coord
                .transform_to_local(&monster.pos)
                .and_then(|p| screen_coord.transform(&p));
            if let Some(Position { x, y }) = pos {
                buf.get_mut(x as u16, y as u16).set_symbol(symbol).set_style(*style);
            }
        }

        if let Some((player, symbol, style)) = &self.player {
            legend_entries.push((
                symbol.to_string(),
                *style,
                format!("goal  {},{}", player.pos.x, player.pos.y),
            ));
            let pos = map_coord
                .transform_to_local(&player.pos)
                .and_then(|p| screen_coord.transform(&p));
            if let Some(Position { x, y }) = pos {
                buf.get_mut(x as u16, y as u16).set_symbol(symbol).set_style(*style);
            }
        }

        use tui::layout::{Constraint, Direction, Layout};
        use tui::widgets::*;

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [Constraint::Length(legend_entries.len() as u16 + 2), Constraint::Min(0)]
                    .as_ref(),
            )
            .split(map_area);

        let legend_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(75), Constraint::Percentage(25)].as_ref())
            .split(layout[0])[1];

        let mut legend = Block::default().title("Legend").borders(Borders::ALL);
        legend.draw(legend_area, buf);
        let legend_area = legend.inner(legend_area);
        for y in legend_area.top()..legend_area.bottom() {
            for x in legend_area.left()..legend_area.right() {
                buf.get_mut(x, y).set_symbol(" ").set_style(Style::default());
            }
        }
        for (y, (symbol, style, text)) in
            (legend_area.top()..legend_area.bottom()).zip(legend_entries.iter())
        {
            let x = legend_area.left();
            buf.get_mut(x, y).set_symbol(symbol).set_style(*style);
            buf.set_string(x + 2, y, text, Style::default())
        }
    }
}

pub struct Visualization {
    pub queue: HashMap<Position, usize>,
    pub visited: HashSet<Position>,
    pub trajectory: Vec<Position>,
}
