use std::ops::{Index, IndexMut};

/// A tile on the map
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tile {
    explored: bool,
    blocked: bool,
    wall: bool,
}

impl Tile {
    pub const WALL: Self = Tile { explored: false, blocked: true, wall: true };

    pub const FLOOR: Self = Tile { explored: false, blocked: false, wall: false };

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Map {
    tiles: Vec<Tile>,
    width: u32,
    height: u32,
}

impl Map {
    /// Create a new Map of blocking tiles
    ///
    /// The default map is impossible to traverse, with the assumption that areas will be carved
    /// out of the map.
    pub fn new(width: u32, height: u32) -> Self {
        let tiles = vec![Tile::WALL; (width * height) as usize];
        Map { tiles, width, height }
    }

    /// Convert two values from a subscript into an index to the tile vector
    fn sub2ind(&self, x: u32, y: u32) -> usize {
        x as usize + y as usize * self.width as usize
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

pub fn generate(width: u32, height: u32) -> Map {
    use rand::prelude::*;

    let mut map = Map::new(width, height);

    let mut fill = 0.0;
    while fill < 0.45 {
        // on first pass, fill the floors with a certain density
        for y in 1..(height - 1) {
            for x in 1..(width - 1) {
                map[(x, y)] = if random::<f32>() < 0.52 { Tile::FLOOR } else { Tile::WALL }
            }
        }

        let mut next = map.clone();

        // For a set number of generations,
        for _ in 0..5 {
            // use a celular automata algorithm to smooth the map
            for y in 1..(height - 1) {
                for x in 1..(width - 1) {
                    let mut adjacency_1 = 0;
                    let mut adjacency_2 = 0;

                    for yy in y - 1..=y + 1 {
                        for xx in x - 1..=x + 1 {
                            if map[(xx, yy)].is_wall() {
                                adjacency_1 += 1;
                            }
                        }
                    }

                    for yy in (y.max(2) - 2)..=(y + 2).min(height - 1) {
                        for xx in (x.max(2) - 2)..=(x + 2).min(width - 1) {
                            if map[(xx, yy)].is_wall() {
                                adjacency_2 += 1;
                            }
                        }
                    }

                    next[(x, y)] = if adjacency_1 >= 5 || adjacency_2 <= 0 {
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
                    let mut adjacency_1 = 0;

                    for yy in y - 1..=y + 1 {
                        for xx in x - 1..=x + 1 {
                            if map[(xx, yy)].is_wall() {
                                adjacency_1 += 1;
                            }
                        }
                    }

                    next[(x, y)] = if adjacency_1 >= 4 { Tile::WALL } else { Tile::FLOOR }
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

                let mut size = 0;
                let mut queue = Vec::with_capacity(width as usize * height as usize);
                queue.push((x, y));
                while queue.len() != 0 {
                    let (x, y) = queue.pop().unwrap();
                    for y in y - 1..y + 2 {
                        for x in x - 1..x + 2 {
                            if cluster_map[(x, y)].is_wall() {
                                continue;
                            }
                            cluster_map[(x, y)] = Tile::WALL;
                            queue.push((x, y));
                            size += 1;
                        }
                    }
                }

                clusters.push((x, y, size));
            }
        }

        clusters.sort_by(|c1, c2| c1.2.cmp(&c2.2));
        clusters.pop();

        for (x, y, _) in clusters {
            let mut queue = vec![(x, y)];
            while queue.len() != 0 {
                let (x, y) = queue.pop().unwrap();
                for y in y - 1..y + 2 {
                    for x in x - 1..x + 2 {
                        if map[(x, y)].is_wall() {
                            continue;
                        } else {
                            map[(x, y)] = Tile::WALL;
                            queue.push((x, y));
                        }
                    }
                }
            }
        }

        let mut count = 0.0;
        for tile in map.tiles.iter() {
            if !tile.is_wall() {
                count += 1.0;
            }
        }
        fill = count / (width as f64 * height as f64);
    }
    map
}
