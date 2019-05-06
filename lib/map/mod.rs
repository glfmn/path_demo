use std::collections::HashSet;
use std::ops::{Index, IndexMut};

use super::{Position, Rect};

/// A Tile on the map
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tile {
    explored: bool,
    blocked: bool,
    wall: bool,
}

impl Tile {
    /// An impassable wall in the game world
    pub const WALL: Self = Tile { explored: false, blocked: true, wall: true };

    /// A tile that entities can be placed in and freely move through
    pub const FLOOR: Self = Tile { explored: false, blocked: false, wall: false };

    /// A tile which blocks movement but is not a wall
    pub const BLOCK: Self = Tile { explored: false, blocked: true, wall: false };

    pub fn is_blocking(&self) -> bool {
        self.blocked
    }

    pub fn is_wall(&self) -> bool {
        self.wall
    }

    pub fn is_explored(&self) -> bool {
        self.explored
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MapError {
    InfiniteLoop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Map {
    tiles: Vec<Tile>,
    width: u32,
    height: u32,
}

impl Map {
    /// Create a new Map of blocking tiles
    ///
    /// The default map is impossible to traverse, with the assumption that areas will be
    /// carved out of the map.
    pub fn new(width: u32, height: u32) -> Self {
        let tiles = vec![Tile::WALL; (width * height) as usize];
        Map { tiles, width, height }
    }

    /// The width and height of the map
    ///
    /// ```
    /// # use game_lib::map::Map;
    /// # let width = 10;
    /// # let height = 10;
    /// let map = Map::new(width, height);
    /// assert_eq!((width, height), map.dimensions());
    /// ```
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Convert two values from a subscript into an index to the tile vector
    #[inline(always)]
    fn sub2ind(&self, x: u32, y: u32) -> usize {
        x as usize + y as usize * self.width as usize
    }

    pub fn pos(&self, pos: &Position) -> Option<&Tile> {
        let index = self.sub2ind(pos.x, pos.y);
        self.tiles.get(index)
    }

    pub fn pos_mut(&mut self, pos: &Position) -> Option<&mut Tile> {
        let index = self.sub2ind(pos.x, pos.y);
        self.tiles.get_mut(index)
    }

    /// Get a reference to a tile, if it exists in the Map
    pub fn get(&self, x: u32, y: u32) -> Option<&Tile> {
        let index = self.sub2ind(x, y);
        self.tiles.get(index)
    }

    /// Get a mutable reference to a tile, if it exists in the Map
    pub fn get_mut(&mut self, x: u32, y: u32) -> Option<&mut Tile> {
        let index = self.sub2ind(x, y);
        self.tiles.get_mut(index)
    }

    /// Run some operation on tiles adjacent to `(x, y)`
    ///
    /// Iterates over the tiles in the order they are laid out in memory.
    ///
    /// |       | `x - 1` | `x` | `x + 1` |
    /// |:-----:|---------|-----|---------|
    /// |`y - 1`| 1       | 2   | 3       |
    /// |`y`    | 4       | 5   | 6       |
    /// |`y + 1`| 7       | 8   | 9       |
    ///
    /// If any values are outside the range of the map, they are simply skipped.
    #[inline]
    pub fn fold_adjacent<B, F>(&self, x: u32, y: u32, range: u32, mut acc: B, op: F) -> B
    where
        F: Fn(&Tile, B) -> B,
    {
        let x_range = x.max(range) - range..=(x + range).min(self.width);
        for y in y.max(range) - range..=(y + range).min(self.height) {
            for x in x_range.clone() {
                if let Some(tile) = self.get(x, y) {
                    acc = op(tile, acc);
                }
            }
        }

        acc
    }

    #[inline]
    pub fn map_adjacent<F>(&self, x: u32, y: u32, range: u32, mut op: F)
    where
        F: FnMut(&Tile),
    {
        for y in y.max(range) - range..=(y + range).min(self.height - 1) {
            for x in x.max(range) - range..=(x + range).min(self.width - 1) {
                if let Some(tile) = self.get(x, y) {
                    op(tile);
                }
            }
        }
    }

    pub fn count_adjacent<F>(&self, x: u32, y: u32, range: u32, pred: F) -> usize
    where
        F: Fn(&Tile) -> bool,
    {
        self.fold_adjacent(x, y, range, 0, |tile, sum| if pred(tile) { sum + 1 } else { sum })
    }

    /// Return a set of adjacent tiles which satisfy a predicate
    ///
    /// If the first tile does not match the predicate, then the selection exits early and
    /// returns an empty set.
    pub fn flood_select<F>(&self, x: u32, y: u32, predicate: F) -> HashSet<(u32, u32)>
    where
        F: Fn(&Tile) -> bool,
    {
        let mut set = HashSet::new();

        // Return early if the first tile in the cluster does not satisfy the predicate
        if let Some(tile) = self.get(x, y) {
            if !predicate(tile) {
                return set;
            }
        }

        let mut queue = vec![(x, y)];
        while !queue.is_empty() {
            // Queue should never be empty, so pop the last element without checking
            let (x, y) = queue.pop().unwrap();

            // Clamp lower bounds to zero to prevent underflow
            for y in y.max(1) - 1..y + 2 {
                for x in x.max(1) - 1..x + 2 {
                    // If a tile exists and has not been seen before, check the predicate
                    if let Some(tile) = self.get(x, y) {
                        if !set.contains(&(x, y)) && predicate(tile) {
                            set.insert((x, y));
                            // search around the tile in a future iteration
                            queue.push((x, y));
                        }
                    }
                }
            }
        }

        set
    }

    /// Replace the set of adjacent elements matching a predicate with a new tile
    ///
    /// The replacement `Tile` _must_ not match the predicate, otherwise this would cause an
    /// infinite loop.
    ///
    /// ```rust
    /// # use game_lib::map::*;
    /// # let mut map = Map::new(5, 5);
    /// let replace = Tile::WALL;
    /// if replace.is_wall() {
    ///     assert_eq!(
    ///         Err(MapError::InfiniteLoop),
    ///         map.flood_replace(1, 1, Tile::is_wall, replace),
    ///     );
    /// }
    /// # else { unreachable!() }
    /// ```
    pub fn flood_replace<F>(
        &mut self,
        x: u32,
        y: u32,
        predicate: F,
        replacement: Tile,
    ) -> Result<usize, MapError>
    where
        F: Fn(&Tile) -> bool,
    {
        use self::MapError::*;

        if predicate(&replacement) {
            return Err(InfiniteLoop);
        }

        let mut size = 0;
        let mut queue = vec![(x, y)];
        while queue.is_empty() {
            let (x, y) = queue.pop().unwrap();
            // prevent underflow
            for y in y.max(1) - 1..y + 2 {
                for x in x.max(1) - 1..x + 2 {
                    if let Some(tile) = self.get_mut(x, y) {
                        if !predicate(tile) {
                            continue;
                        } else {
                            *tile = replacement.clone();
                            queue.push((x, y));
                            size += 1;
                        }
                    }
                }
            }
        }

        Ok(size)
    }

    /// Iterate over the tiles inside a rectangular area contained in the map
    pub fn iter_rect(&self, area: Rect) -> MapArea<'_> {
        MapArea { x: 0, y: 0, area, map: self }
    }
}

impl Index<(u32, u32)> for Map {
    type Output = Tile;

    fn index(&self, (x, y): (u32, u32)) -> &Self::Output {
        if y >= self.height || x >= self.width {
            panic!("Index ({}, {}) out of bounds ({}, {})", x, y, self.width, self.height);
        }

        let index = self.sub2ind(x, y);
        &self.tiles[index]
    }
}

impl IndexMut<(u32, u32)> for Map {
    fn index_mut(&mut self, (x, y): (u32, u32)) -> &mut Tile {
        if y >= self.height || x >= self.width {
            panic!("Index ({}, {}) out of bounds ({}, {})", x, y, self.width, self.height);
        }

        let index = self.sub2ind(x, y);
        &mut self.tiles[index]
    }
}

impl Index<Position> for Map {
    type Output = Tile;

    fn index(&self, Position { x, y }: Position) -> &Self::Output {
        if y >= self.height || x >= self.width {
            panic!("Index ({}, {}) out of bounds ({}, {})", x, y, self.width, self.height);
        }

        let index = self.sub2ind(x, y);
        &self.tiles[index]
    }
}

impl IndexMut<Position> for Map {
    fn index_mut(&mut self, Position { x, y }: Position) -> &mut Tile {
        if y >= self.height || x >= self.width {
            panic!("Index ({}, {}) out of bounds ({}, {})", x, y, self.width, self.height);
        }

        let index = self.sub2ind(x, y);
        &mut self.tiles[index]
    }
}

/// Iterate over the tiles inside a rectangular area contained in the map
///
/// See `Map::iter_rect`
pub struct MapArea<'a> {
    x: u32,
    y: u32,
    area: Rect,
    map: &'a Map,
}

impl<'a> Iterator for MapArea<'a> {
    type Item = (Position, &'a Tile);

    fn next(&mut self) -> Option<Self::Item> {
        let pos = (self.x, self.y).into();
        if self.y < self.area.h {
            if self.x < self.area.w {
                self.x += 1;
            } else {
                self.x = 0;
                self.y += 1;
            }
        } else {
            return None;
        }

        self.area
            .transform(&pos)
            .and_then(|map_pos| self.map.pos(&map_pos).map(|tile| (pos, tile)))
    }
}

pub fn generate<R>(rng: &mut R, width: u32, height: u32) -> Map
where
    R: rand::Rng,
{
    let mut map = Map::new(width, height);

    let mut fill = 0.0;
    while fill < 0.45 {
        // on first pass, fill the floors with a certain density
        for y in 1..(height - 1) {
            for x in 1..(width - 1) {
                map[(x, y)] = if rng.gen::<f32>() < 0.52 { Tile::FLOOR } else { Tile::WALL }
            }
        }

        let mut next = map.clone();

        // For a set number of generations,
        for _ in 0..5 {
            // use a celular automata algorithm to smooth the map
            for y in 1..(height - 1) {
                for x in 1..(width - 1) {
                    let adjacency_1 = map.count_adjacent(x, y, 1, Tile::is_wall);
                    let adjacency_2 = map.count_adjacent(x, y, 2, Tile::is_wall);

                    next[(x, y)] = if adjacency_1 >= 5 || adjacency_2 == 0 {
                        Tile::WALL
                    } else {
                        Tile::FLOOR
                    }
                }
            }
            map = next.clone();
        }

        // For a set number of generations,
        for _ in 0..1 {
            // use a celular automata algorithm to smooth the map
            for y in 1..(height - 1) {
                for x in 1..(width - 1) {
                    let count = map.count_adjacent(x, y, 1, Tile::is_wall);
                    next[(x, y)] = if count >= 4 { Tile::WALL } else { Tile::FLOOR }
                }
            }
            map = next.clone();
        }

        let mut clusters: Vec<(u32, u32, usize)> = Vec::new();
        let mut cluster_map = map.clone();
        for y in 1..height - 1 {
            for x in 1..width - 1 {
                if cluster_map[(x, y)].is_wall() {
                    continue;
                }

                if let Ok(size) =
                    cluster_map.flood_replace(x, y, |tile| !tile.is_wall(), Tile::WALL)
                {
                    clusters.push((x, y, size));
                }
            }
        }

        clusters.sort_by(|c1, c2| c1.2.cmp(&c2.2));
        clusters.pop();

        for (x, y, _) in clusters {
            match map.flood_replace(x, y, |tile| !tile.is_wall(), Tile::WALL) {
                Ok(_) => continue,
                Err(MapError::InfiniteLoop) => continue,
            }
        }

        let mut count = 0.0;
        for tile in map.tiles.iter() {
            if !tile.is_wall() {
                count += 1.0;
            }
        }
        fill = count / (f64::from(width) * f64::from(height));
    }
    map
}
