extern crate sdl2;

use crate::util::*;
use gamestate::GameState;
use images::Images;
use renderer::Renderer;
use sdl2::event::Event;
use sdl2::image::InitFlag;
use sdl2::keyboard::Keycode;

const BOARD_LENGTH: i32 = 8;
const TILE_SIZE: i32 = 96;

mod images;
mod renderer;
mod util;

mod gamestate;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let _image_context = sdl2::image::init(InitFlag::PNG);

    let window_length = (BOARD_LENGTH * TILE_SIZE) as u32;

    let window = video_subsystem
        .window("chess", window_length, window_length)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let canvas = window
        .into_canvas()
        .software()
        .build()
        .map_err(|e| e.to_string())?;

    let texture_creator = canvas.texture_creator();

    let mut renderer = Renderer::new(canvas, Images::new(&texture_creator)?)?;
    let mut gamestate = GameState::new();

    let mut game_continue = true;

    'mainloop: loop {
        for event in sdl_context.event_pump()?.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Option::Some(Keycode::Escape),
                    ..
                } => break 'mainloop,

                Event::MouseMotion { x, y, .. } if game_continue => {
                    gamestate.mouse_move(x, y);
                }

                Event::MouseButtonDown { x, y, .. } if game_continue => {
                    gamestate.mouse_down(x, y);
                }

                Event::MouseButtonUp { x, y, .. } if game_continue => {
                    gamestate.mouse_up(x, y);
                }
                _ => {}
            }
        }
        renderer.draw(&gamestate)?;
        match gamestate.end_game() {
            PlayStatus::Continue => {}
            PlayStatus::Draw => {
                if game_continue {
                    game_continue = false;
                    println!("Draw!");
                }
            }
            PlayStatus::Win(color) => {
                if game_continue {
                    game_continue = false;
                    let winner = if color == Color::Black {
                        "Black"
                    } else {
                        "White"
                    };
                    println!("{winner} Won!");
                }
            }
        }
    }

    Ok(())
}
