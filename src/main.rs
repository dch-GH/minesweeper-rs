mod constants;
mod minefield;

use crate::constants::*;
use crate::minefield::*;

use raylib::prelude::*;

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
    let mine_sprite = rl.load_texture(&thread, "sprites/sad32x32.png").unwrap();

    let width = rl.get_screen_width();
    let height = rl.get_screen_height();
    println!("Screen width: {} height: {}", width, height);

    let mut mine_field = MineField::new(width, height);
    let mut game_state = GameState::PreGame;

    while !rl.window_should_close() {
        // Tick
        let mouse_pos = rl.get_mouse_position();
        let left_click_released = rl.is_mouse_button_released(MouseButton::MOUSE_LEFT_BUTTON);
        let right_click_released = rl.is_mouse_button_released(MouseButton::MOUSE_RIGHT_BUTTON);

        match game_state {
            GameState::PreGame => {
                if left_click_released {
                    match mine_field
                        .tiles
                        .iter_mut()
                        .filter(|x| x.rect.check_collision_point_rec(mouse_pos))
                        .nth(0)
                    {
                        Some(tile) => {
                            // We do this here to prevent the player from clicking on a mine
                            // with their first guess.
                            tile.revealed = true;
                            game_state = GameState::Playing;
                            mine_field.populate_mines();
                        }
                        None => {}
                    }
                }
            }

            GameState::Playing => {
                // Dig up a tile.
                if left_click_released {
                    match mine_field
                        .tiles
                        .iter_mut()
                        .filter(|x| x.rect.check_collision_point_rec(mouse_pos))
                        .nth(0)
                    {
                        Some(tile) => {
                            // Don't dig up flagged tiles.
                            if !tile.revealed && !tile.flagged {
                                tile.revealed = true;
                                println!("{}", mine_field.required_num_to_clear);
                                if tile.has_mine {
                                    game_state = GameState::GameOver;
                                }
                            }
                        }
                        None => {}
                    }
                }

                if right_click_released {
                    let tile = mine_field
                        .tiles
                        .iter_mut()
                        .filter(|x| x.rect.check_collision_point_rec(mouse_pos))
                        .nth(0)
                        .unwrap();

                    if !tile.revealed {
                        tile.flagged = !tile.flagged;
                    }
                }
            }

            GameState::GameOver => {
                if rl.is_key_released(KeyboardKey::KEY_SPACE) {
                    game_state = GameState::PreGame;
                    mine_field = MineField::new(width, height);
                }
            }
        }

        // Draw
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);

        for tile in mine_field.tiles.iter() {
            let tile_color = match tile.revealed {
                true => Color::DARKGREEN,
                false => tile.color,
            };

            let tile_x = tile.pixel_position.0;
            let tile_y = tile.pixel_position.1;

            d.draw_rectangle(tile_x, tile_y, TILE_SIZE - 2, TILE_SIZE - 2, tile_color);

            if game_state == GameState::GameOver && tile.has_mine {
                d.draw_texture(&mine_sprite, tile_x, tile_y, Color::RED);
            }

            if tile.flagged {
                d.draw_texture(&flag_sprite, tile_x, tile_y, Color::BLUE);
            }

            if !tile.has_mine && tile.revealed && tile.mine_neighbor_count > 0 {
                // Draw number of neighbor tiles which have a mine.
                d.draw_text(
                    &format!("{}", tile.mine_neighbor_count),
                    tile_x + TILE_SIZE / 4,
                    tile_y + TILE_SIZE / 4,
                    18,
                    Color::BLACK,
                );
            }
        }

        if game_state == GameState::GameOver {
            d.draw_text(
                "Game Over!\nPress space to restart.",
                width / 4,
                height / 4,
                28,
                Color::WHITE,
            );
        }
    }
}
