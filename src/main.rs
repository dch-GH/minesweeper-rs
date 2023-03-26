mod minefield;

use crate::minefield::*;
use raylib::prelude::*;

pub(crate) const TILE_SIZE: i32 = 32;
pub(crate) const TILE_SIZE_F: f32 = TILE_SIZE as f32;
pub(crate) const TILE_COLOR_PALETTE_HEX: [&str; 3] = ["69B578", "D0DB97", "3A7D44"];

const BACKGROUND_COLOR_HEX: &str = "181D27";

#[derive(Debug, PartialEq)]
enum GameState {
    PreGame,
    Playing,
    GameOver,
    Victory,
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(512, 512)
        .vsync()
        .title("Minesweeper")
        .build();

    let bg_color = Color::from_hex(BACKGROUND_COLOR_HEX).unwrap();

    // Load assets
    let flag_sprite = rl.load_texture(&thread, "sprites/flag32x32.png").unwrap();
    let font_bold = rl.load_font(&thread, "fonts/OpenSans-Bold.ttf").unwrap();

    let width = rl.get_screen_width();
    let height = rl.get_screen_height();
    println!("Screen width: {} height: {}", width, height);

    let mut mine_field = MineField::new(width, height);
    let mut game_state = GameState::PreGame;

    while !rl.window_should_close() {
        // Tick
        {
            let mouse_pos = rl.get_mouse_position();
            let left_click_released = rl.is_mouse_button_released(MouseButton::MOUSE_LEFT_BUTTON);
            let right_click_released = rl.is_mouse_button_released(MouseButton::MOUSE_RIGHT_BUTTON);
            let paint_left_click = rl.is_mouse_button_down(MouseButton::MOUSE_LEFT_BUTTON)
                && rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT);

            match game_state {
                GameState::PreGame => {
                    // Pre-Game starting tile.
                    if left_click_released {
                        let clone_tiles = mine_field.tiles.clone();
                        if let Some(clicked_tile) = clone_tiles
                            .iter()
                            .find(|x| x.rect.check_collision_point_rec(mouse_pos))
                        {
                            // We do this here to prevent the player from clicking on a mine
                            // with their first guess.
                            mine_field.reveal_tile(clicked_tile.index);

                            game_state = GameState::Playing;
                            mine_field.populate_mines();
                            mine_field.update_neighbors();
                            mine_field.flood_reveal_from_pos(clicked_tile.coords);
                        }
                    }
                }

                GameState::Playing => {
                    // Dig up a tile.
                    if left_click_released || paint_left_click {
                        let clone_tiles = mine_field.tiles.clone();
                        if let Some(clicked_tile) = clone_tiles
                            .iter()
                            .find(|x| x.rect.check_collision_point_rec(mouse_pos))
                        {
                            if !clicked_tile.revealed && !clicked_tile.flagged {
                                mine_field.reveal_tile(clicked_tile.index);
                                if clicked_tile.has_mine {
                                    game_state = GameState::GameOver;
                                } else if clicked_tile.adjacent_mines <= 0 {
                                    mine_field.flood_reveal_from_pos(clicked_tile.coords);
                                }
                            }

                            if mine_field.required_num_to_clear == 0 {
                                game_state = GameState::Victory;
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
                GameState::Victory => {}
            }

            // Handle retrying.
            if game_state != GameState::PreGame && rl.is_key_released(KeyboardKey::KEY_SPACE) {
                game_state = GameState::PreGame;
                mine_field = MineField::new(width, height);
            }
        }

        // Draw
        {
            let mut d = rl.begin_drawing(&thread);
            d.clear_background(bg_color);

            for tile in mine_field.tiles.iter() {
                let tile_color = match tile.revealed {
                    true => bg_color,
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
                    d.draw_circle(
                        tile_x + TILE_SIZE / 2 - 1,
                        tile_y + TILE_SIZE / 2 - 1,
                        9.0,
                        Color::RED,
                    );
                }

                if tile.flagged {
                    d.draw_texture(&flag_sprite, tile_x, tile_y, Color::BLUE);
                }

                if !tile.has_mine && tile.revealed && tile.adjacent_mines > 0 {
                    // Draw number of neighbor tiles which have a mine.
                    d.draw_text_ex(
                        &font_bold,
                        &format!("{}", tile.adjacent_mines),
                        Vector2 {
                            x: tile_x as f32 + TILE_SIZE_F / 4.0,
                            y: tile_y as f32 + TILE_SIZE_F / 4.0,
                        },
                        24.0,
                        1.0,
                        get_danger_color(tile.adjacent_mines),
                    );
                }
            }

            match game_state {
                GameState::PreGame => {}
                GameState::Playing => {}
                GameState::GameOver => {
                    d.draw_text_ex(
                        &font_bold,
                        "Game Over. Press SPACE to try again.",
                        Vector2 { x: 0.1, y: 0.2 },
                        24.0,
                        1.0,
                        Color::BLACK,
                    );
                    d.draw_text_ex(
                        &font_bold,
                        "Game Over. Press SPACE to try again.",
                        Vector2 { x: 1.0, y: 1.0 },
                        24.0,
                        1.0,
                        Color::WHITE,
                    );
                }
                GameState::Victory => {
                    d.draw_text_ex(
                        &font_bold,
                        "You Win! Press SPACE to play again.",
                        Vector2 { x: 0.1, y: 0.2 },
                        24.0,
                        1.0,
                        Color::BLACK,
                    );

                    d.draw_text_ex(
                        &font_bold,
                        "You Win! Press SPACE to play again.",
                        Vector2 { x: 1.0, y: 1.0 },
                        24.0,
                        1.0,
                        Color::WHITE,
                    );
                }
            }
        }
    }
}
