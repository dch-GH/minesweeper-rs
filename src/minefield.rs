use crate::constants::TILE_SIZE;
use raylib::prelude::*;

#[derive(Debug, Copy, Clone)]
pub(crate) struct MineFieldTile {
    pub(crate) rect: Rectangle,
    pub(crate) pixel_position: (i32, i32),
    pub(crate) revealed: bool,
    pub(crate) has_mine: bool,
    pub(crate) flagged: bool,
    pub(crate) mine_neighbor_count: u8,
    pub(crate) index: i32,
    pub(crate) color: Color,
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
                pixel_position: (x_pos, y_pos),
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

    pub(crate) fn populate_mines(&mut self) {
        for tile in self.tiles.iter_mut() {
            if tile.revealed {
                continue;
            }

            tile.has_mine = get_random_value::<i32>(0, 10) == 2;
        }

        for (index, tile) in self.tiles.clone().iter().enumerate() {
            if tile.has_mine {
                continue;
            }

            let (_, neighbors) = self.get_neighbors(tile.pixel_position.0, tile.pixel_position.1);

            for n in neighbors {
                if n.is_some() && n.unwrap().has_mine {
                    self.tiles[index].mine_neighbor_count += 1;
                }
            }

            self.required_num_to_clear += 1;
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

        for tile in self.tiles.iter() {
            if tile.pixel_position.0 == x && tile.pixel_position.1 == y {
                return Some(*tile);
            }
        }

        return None;
    }

    pub(crate) fn get_neighbors(&self, x: i32, y: i32) -> (i32, Vec<Option<MineFieldTile>>) {
        // How many neighbors are Some();
        let mut count_valid_neighbors = 0;
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

        for n in neighbors.iter() {
            if n.is_some() {
                count_valid_neighbors += 1;
            }
        }

        return (count_valid_neighbors, neighbors);
    }
}
