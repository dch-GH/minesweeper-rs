use ::core::panic;

use crate::{TILE_COLOR_PALETTE_HEX, TILE_SIZE};
use raylib::prelude::{Color, *};

const MAX_FLOOD_TILES: i32 = 100;
const MINE_CHANCE: i32 = 5;

type TileIndex = usize;

// This really doesn't need to be this big of a struct.
// Could be shrunken down to an (x,y) and bitflags.
#[derive(Debug, Copy, Clone)]
pub(crate) struct MineFieldTile {
    pub(crate) rect: Rectangle,
    pub(crate) coords: (i32, i32),
    pub(crate) revealed: bool,
    pub(crate) has_mine: bool,
    pub(crate) flagged: bool,
    pub(crate) adjacent_mines: i32,
    pub(crate) index: TileIndex,
    pub(crate) color: Color,
}

#[derive(Clone)]
pub(crate) struct MineField {
    pub(crate) size: (i32, i32),
    // TODO: This doesn't need to be a Vec at all. [x,y] instead.
    pub(crate) tiles: Vec<MineFieldTile>,
    pub(crate) required_num_to_clear: usize,
}

pub(crate) fn get_danger_color(num_mines: i32) -> Color {
    match num_mines {
        0 => Color::BLUE,
        1 => Color::BLUE,
        2 => Color::GREEN,
        _ => Color::RED,
    }
}

impl MineField {
    pub(crate) fn new(width: i32, height: i32) -> Self {
        let num_tiles_to_generate = (width / TILE_SIZE) * (height / TILE_SIZE);
        let mut field = Vec::new();

        println!("Generating new minefield!");
        println!("Expected num tiles: {}", num_tiles_to_generate);

        let mut x_index = 0;
        let mut row = 0;

        for tile_index in 0..num_tiles_to_generate {
            let mut x_pos = x_index * TILE_SIZE;

            if x_pos + TILE_SIZE > width {
                x_index = 0;
                x_pos = 0;
                row += 1;
            }

            x_index += 1;

            let y_pos = row * TILE_SIZE;

            let max_pallete = TILE_COLOR_PALETTE_HEX.len() - 1;
            let random_tile_color_index = get_random_value::<i32>(0, max_pallete as i32) as usize;
            let tile_color = TILE_COLOR_PALETTE_HEX[random_tile_color_index];

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
                adjacent_mines: (0),
                index: tile_index as TileIndex,
                color: Color::from_hex(tile_color).unwrap(),
            };

            field.push(tile);
        }

        println!("Total tiles generated: {}", field.len());

        Self {
            size: (width, height),
            tiles: field,
            required_num_to_clear: 0,
        }
    }

    pub(crate) fn update_neighbors(&mut self) {
        for mut t in self.tiles.iter_mut() {
            t.adjacent_mines = 0;
        }

        for (index, tile) in self.tiles.clone().iter().enumerate() {
            for n in self.get_neighbors(tile.coords.0, tile.coords.1).iter() {
                match n {
                    Some(neighbor) => {
                        if neighbor.has_mine {
                            self.tiles[index].adjacent_mines += 1;
                        }
                    }
                    None => {}
                }
            }
        }
    }

    /// Set up the minefield, populate mines.
    /// This will also calculate the score the player needs to achieve to win.
    pub(crate) fn populate_mines(&mut self) {
        // Plant the mines
        for tile in self.tiles.iter_mut() {
            // A tile may be revealed here because it
            // may be the PreGame starting tile!
            // No mine on this tile, that would be lame.
            if tile.revealed {
                continue;
            }

            if get_random_value::<i32>(0, MINE_CHANCE) == 1 {
                tile.has_mine = true;
            } else {
                self.required_num_to_clear += 1;
            }
        }

        println!("Player must clear {} tiles.", self.required_num_to_clear);
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

        let mut flood_revealed_tiles: i32 = 0;
        flood_queue.push(origin_tile);

        'flood: loop {
            if flood_queue.is_empty() {
                break 'flood;
            }

            let tile = match flood_queue.pop() {
                None => {
                    panic!("Couldn't pop from flood queue!");
                }
                Some(tile) => tile,
            };

            for neighbor in self
                .get_neighbors(tile.coords.0, tile.coords.1)
                .iter()
                .filter(|x| {
                    if x.is_none() {
                        return false;
                    }

                    let n = x.unwrap();
                    !n.has_mine && !n.revealed
                })
            {
                if flood_revealed_tiles >= MAX_FLOOD_TILES {
                    break 'flood;
                }

                match neighbor {
                    None => {
                        panic!("Neighbor was None in flood reveal get_neighbors!");
                    }
                    Some(neighbor) => {
                        self.reveal_tile(neighbor.index);
                        flood_revealed_tiles += 1;
                        if neighbor.adjacent_mines > 0 {
                            continue;
                        }
                        flood_queue.push(*neighbor);
                    }
                }
            }
        }
    }

    pub(crate) fn get_tile(&self, x: i32, y: i32) -> Option<MineFieldTile> {
        if self.tiles.is_empty() || x > self.size.0 || y > self.size.1 || x < 0 || y < 0 {
            return None;
        }

        self.tiles
            .iter()
            .find(|tile| tile.coords == (x, y))
            .copied()
    }

    pub(crate) fn reveal_tile(&mut self, index: TileIndex) {
        if index >= self.tiles.len() {
            return;
        }

        self.tiles[index].revealed = true;
        if self.required_num_to_clear > 0 {
            self.required_num_to_clear -= 1;
        }
    }

    pub(crate) fn get_neighbors(&self, x: i32, y: i32) -> [Option<MineFieldTile>; 8] {
        // Clockwise order from NW -> W;
        let northwest = self.get_tile(x - TILE_SIZE, y + TILE_SIZE);
        let north = self.get_tile(x, y + TILE_SIZE);
        let northeast = self.get_tile(x + TILE_SIZE, y + TILE_SIZE);
        let east = self.get_tile(x + TILE_SIZE, y);
        let southeast = self.get_tile(x + TILE_SIZE, y - TILE_SIZE);
        let south = self.get_tile(x, y - TILE_SIZE);
        let southwest = self.get_tile(x - TILE_SIZE, y - TILE_SIZE);
        let west = self.get_tile(x - TILE_SIZE, y);

        [
            northwest, north, northeast, east, southeast, south, southwest, west,
        ]
    }
}
