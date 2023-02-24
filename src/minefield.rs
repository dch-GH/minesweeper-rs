use ::core::panic;

use crate::constants::{MAX_FLOOD_TILES, TILE_SIZE};
use raylib::prelude::{Color, *};

#[derive(Debug, Copy, Clone)]
pub(crate) struct MineFieldTile {
    pub(crate) rect: Rectangle,
    pub(crate) coords: (i32, i32),
    pub(crate) revealed: bool,
    pub(crate) has_mine: bool,
    pub(crate) flagged: bool,
    pub(crate) mine_neighbor_count: i32,
    pub(crate) index: usize,
    pub(crate) color: Color,
}

impl MineFieldTile {
    pub(crate) fn danger_color(mines: i32) -> Color {
        match mines {
            0 => Color::BLUE,
            1 => Color::BLUE,
            2 => Color::GREEN,
            _ => Color::RED,
        }
    }
}

#[derive(Clone)]
pub(crate) struct MineField {
    pub(crate) size: Vector2,
    pub(crate) tiles: Vec<MineFieldTile>,
    pub(crate) required_num_to_clear: i32,
}

impl MineField {
    pub(crate) fn new(width: i32, height: i32) -> Self {
        let num_tiles_to_generate = (width / TILE_SIZE) * (height / TILE_SIZE);
        let mut field = Vec::new();

        println!("Generating new minefield!");
        println!("Expected num tiles: {}", num_tiles_to_generate);

        let mut tile_index = 0;
        let mut x_index = 0;
        let mut row = 0;

        for _ in 0..num_tiles_to_generate {
            let mut x_pos = x_index * TILE_SIZE;

            if x_pos >= width {
                x_index = 0;
                x_pos = 0;
                row += 1;
            }

            x_index += 1;

            let y_pos = row * TILE_SIZE;

            let ran_r = get_random_value::<i32>(5, 10) as u8;
            let ran_g = get_random_value::<i32>(180, 255) as u8;
            let ran_b = get_random_value::<i32>(0, 30) as u8;
            let tile_color = rcolor(ran_r, ran_g, ran_b, 255);

            let tile: MineFieldTile = MineFieldTile {
                rect: Rectangle {
                    x: x_pos as f32,
                    y: y_pos as f32,
                    width: TILE_SIZE as f32,
                    height: TILE_SIZE as f32,
                },
                coords: (x_pos, y_pos),
                revealed: false,
                has_mine: false,
                flagged: false,
                mine_neighbor_count: (0),
                index: tile_index,
                color: tile_color,
            };

            field.push(tile);
            tile_index += 1;
        }

        println!("Total tiles generated: {}", field.len());

        Self {
            size: Vector2 {
                x: width as f32,
                y: height as f32,
            },
            tiles: field,
            required_num_to_clear: 0,
        }
    }

    pub(crate) fn update_neighbors(&mut self) {
        for mut t in self.tiles.iter_mut() {
            t.mine_neighbor_count = 0;
        }

        for (index, tile) in self.tiles.clone().iter().enumerate() {
            for n in self.get_neighbors(tile.coords.0, tile.coords.1).iter() {
                match n {
                    Some(neighbor) => {
                        if neighbor.has_mine {
                            self.tiles[index].mine_neighbor_count += 1;
                        }
                    }
                    None => {}
                }
            }
        }
    }

    pub(crate) fn populate_mines(&mut self) {
        // Plant the mines
        for tile in self.tiles.iter_mut() {
            if tile.revealed {
                continue;
            }

            if get_random_value::<i32>(0, 10) == 2 {
                tile.has_mine = true;
            } else {
                self.required_num_to_clear += 1;
            }
        }

        self.update_neighbors();
    }

    pub(crate) fn flood_reveal_from_pos(&mut self, pos: (i32, i32)) {
        let mut flood_queue: Vec<MineFieldTile> = Vec::new();
        let origin_tile = match self.get_tile(pos.0, pos.1) {
            None => {
                panic!("Origin tile is None!");
            }
            Some(tile) => tile,
        };

        let mut flood_count: i32 = 0;
        flood_queue.push(origin_tile);
        while !flood_queue.is_empty() {
            let tile = match flood_queue.pop() {
                None => {
                    panic!("Couldn't pop from flood queue vector!");
                }
                Some(tile) => tile,
            };

            if flood_count >= MAX_FLOOD_TILES {
                break;
            }

            for neighbor in self
                .get_neighbors(tile.coords.0, tile.coords.1)
                .iter()
                .filter(|x| {
                    if x.is_none() {
                        return false;
                    }

                    let n = x.unwrap();
                    !n.has_mine && n.mine_neighbor_count <= 0 && !n.revealed
                })
            {
                match neighbor {
                    None => {
                        panic!("Neighbor was None in flood reveal get_neighbors!");
                    }
                    Some(neighbor) => {
                        self.tiles[neighbor.index].revealed = true;
                        flood_queue.push(*neighbor);
                    }
                }
            }

            flood_count += 1;
            assert!(flood_queue.len() <= 800);
        }
    }

    pub(crate) fn get_tile(&self, x: i32, y: i32) -> Option<MineFieldTile> {
        if self.tiles.is_empty()
            || x > self.size.x as i32
            || y > self.size.y as i32
            || x < 0
            || y < 0
        {
            return None;
        }

        let index: usize =
            (x / TILE_SIZE + (self.size.x as i32 / TILE_SIZE as i32 * y / TILE_SIZE)) as usize;
        self.tiles
            .iter()
            .filter(|x| x.index == index)
            .nth(0)
            .copied()
    }

    pub(crate) fn get_neighbors(&self, x: i32, y: i32) -> Vec<Option<MineFieldTile>> {
        let mut neighbors: Vec<Option<MineFieldTile>> = Vec::new();

        // Clockwise order from NW -> W;

        // Northwest
        neighbors.push(self.get_tile(x - TILE_SIZE, y + TILE_SIZE));

        // North
        neighbors.push(self.get_tile(x, y + TILE_SIZE));

        // Northeast
        neighbors.push(self.get_tile(x + TILE_SIZE, y + TILE_SIZE));

        // East
        neighbors.push(self.get_tile(x + TILE_SIZE, y));

        // Southeast
        neighbors.push(self.get_tile(x + TILE_SIZE, y - TILE_SIZE));

        // South
        neighbors.push(self.get_tile(x, y - TILE_SIZE));

        // Southwest
        neighbors.push(self.get_tile(x - TILE_SIZE, y - TILE_SIZE));

        // West
        neighbors.push(self.get_tile(x - TILE_SIZE, y));

        return neighbors;
    }
}
