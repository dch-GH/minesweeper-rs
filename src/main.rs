mod minefield;

use crate::minefield::*;

use raylib::prelude::*;
pub(crate) const TILE_SIZE: i32 = 32;
pub(crate) const TILE_SIZE_F: f32 = TILE_SIZE as f32;

#[derive(Debug, PartialEq)]
enum GameState {
    PreGame,
    Playing,
    GameOver,
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(512, 512)
        .vsync()
        .title("Minesweeper")
        .build();

    let flag_sprite = rl.load_texture(&thread, "sprites/flag32x32.png").unwrap();
    let font_regular = rl.load_font(&thread, "fonts/OpenSans-Regular.ttf").unwrap();
    let font_bold = rl.load_font(&thread, "fonts/OpenSans-Bold.ttf").unwrap();

    let width = rl.get_screen_width();
    let height = rl.get_screen_height();
    println!("Screen width: {} height: {}", width, height);

    let mut mine_field = MineField::new(width, height);
    mine_field.populate_mines();
    let mut game_state = GameState::PreGame;

    while !rl.window_should_close() {
        // Tick
        {
            let mouse_pos = rl.get_mouse_position();
            let left_click_released = rl.is_mouse_button_released(MouseButton::MOUSE_LEFT_BUTTON);
            let right_click_released = rl.is_mouse_button_released(MouseButton::MOUSE_RIGHT_BUTTON);

            match game_state {
                GameState::PreGame => {
                    // Pre-Game starting tile.
                    if left_click_released {
                        let clone_tiles = mine_field.tiles.clone();
                        match clone_tiles
                            .iter()
                            .find(|x| x.rect.check_collision_point_rec(mouse_pos))
                        {
                            Some(clicked_tile) => {
                                // We do this here to prevent the player from clicking on a mine
                                // with their first guess.
                                mine_field.tiles[clicked_tile.index].revealed = true;

                                if clicked_tile.has_mine {
                                    mine_field.tiles[clicked_tile.index].has_mine = false;
                                }

                                game_state = GameState::Playing;
                                mine_field.update_neighbors();
                                mine_field.flood_reveal_from_pos(clicked_tile.coords);
                            }

                            None => {
                                // Ignore?
                            }
                        }
                    }
                }

                GameState::Playing => {
                    // Dig up a tile.
                    if left_click_released {
                        let clone_tiles = mine_field.tiles.clone();
                        match clone_tiles
                            .iter()
                            .find(|x| x.rect.check_collision_point_rec(mouse_pos))
                        {
                            Some(clicked_tile) => {
                                if !clicked_tile.revealed && !clicked_tile.flagged {
                                    mine_field.tiles[clicked_tile.index].revealed = true;
                                    if clicked_tile.has_mine {
                                        game_state = GameState::GameOver;
                                    } else if clicked_tile.mine_neighbor_count <= 0 {
                                        mine_field.flood_reveal_from_pos(clicked_tile.coords);
                                    }
                                }
                            }

                            None => {
                                // Ignore?
                            }
                        }
                    }

                    // Flag a tile.
                    if right_click_released {
                        let tile = mine_field
                            .tiles
                            .iter_mut()
                            .find(|x| x.rect.check_collision_point_rec(mouse_pos))
                            .unwrap();

                        if !tile.revealed {
                            tile.flagged = !tile.flagged;
                        }
                    }
                }

                GameState::GameOver => {}
            }

            // Handle retrying.
            if game_state != GameState::PreGame && rl.is_key_released(KeyboardKey::KEY_SPACE) {
                game_state = GameState::PreGame;
                mine_field = MineField::new(width, height);
                mine_field.populate_mines();
            }
        }

        // Draw
        {
            let mut d = rl.begin_drawing(&thread);
            d.clear_background(Color::DARKGREEN);

            for tile in mine_field.tiles.iter() {
                let tile_color = match tile.revealed {
                    true => Color::DARKGREEN,
                    false => tile.color,
                };

                let tile_x = tile.coords.0;
                let tile_y = tile.coords.1;

                d.draw_rectangle_rounded(
                    Rectangle {
                        x: tile_x as f32,
                        y: tile_y as f32,
                        width: TILE_SIZE_F - 2.0,
                        height: TILE_SIZE_F - 2.0,
                    },
                    0.3,
                    4,
                    tile_color,
                );

                if game_state == GameState::GameOver && tile.has_mine {
                    d.draw_circle_gradient(
                        tile_x + TILE_SIZE / 2,
                        tile_y + TILE_SIZE / 2,
                        8.0,
                        Color::RED,
                        Color::DARKPURPLE,
                    );
                }

                if tile.flagged {
                    d.draw_texture(&flag_sprite, tile_x, tile_y, Color::BLUE);
                }

                if !tile.has_mine && tile.revealed && tile.mine_neighbor_count > 0 {
                    // Draw number of neighbor tiles which have a mine.
                    d.draw_text_ex(
                        &font_regular,
                        &format!("{}", tile.mine_neighbor_count),
                        Vector2 {
                            x: tile_x as f32 + TILE_SIZE_F / 4.0,
                            y: tile_y as f32 + TILE_SIZE_F / 4.0,
                        },
                        24.0,
                        1.0,
                        get_danger_color(tile.mine_neighbor_count),
                    );
                }
            }

            if game_state == GameState::GameOver {
                d.draw_text_ex(
                    &font_bold,
                    "Game Over!\nPress space to restart.",
                    Vector2 {
                        x: width as f32 / 4.0,
                        y: height as f32 / 4.0,
                    },
                    34.0,
                    1.0,
                    Color::WHITE,
                );
            }
        }
    }
}
