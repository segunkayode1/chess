use super::images::Images;
use crate::gamestate::GameState;
use crate::util::*;
use crate::{BOARD_LENGTH, TILE_SIZE};
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::pixels::Color;
use sdl2::rect;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

const TRANSPARENCY: f64 = 0.6;

pub struct Renderer<'a> {
    canvas: Canvas<Window>,
    images: Images<'a>,
}

impl<'a> Renderer<'a> {
    pub fn new(canvas: Canvas<Window>, images: Images<'a>) -> Result<Self, String> {
        Ok(Self { canvas, images })
    }

    pub fn draw(&mut self, gamestate: &GameState) -> Result<(), String> {
        self.canvas.copy(self.images.get_background(), None, None)?;

        self.draw_selected_piece(gamestate.slected_piece_coord())?;

        if gamestate.in_check() {
            let king_position = gamestate.get_king();
            self.draw_king_threatened(king_position)?;
        }

        let mut moves = gamestate.legal_moves();

        if let Some(point) = gamestate.get_selected_tile() {
            if moves.contains(&point) {
                moves.remove(&point);
                self.draw_selected_tile(point)?;
            }
        }

        if let Some(Move { src, dst }) = gamestate.last_move {
            self.draw_yellow_tile(src)?;
            self.draw_yellow_tile(dst)?;
        }

        for (y, x) in moves {
            if gamestate.is_empty((y, x)) {
                self.draw_empty_move_tile((x, y))?;
            } else {
                self.draw_occupied_move_tile((x, y))?;
            }
        }
        self.draw_tiles(gamestate)?;

        self.draw_moving_piece(gamestate.get_moving_piece())?;
        self.canvas.present();
        Ok(())
    }

    fn draw_tiles(&mut self, gamestate: &GameState) -> Result<(), String> {
        for i in 0..BOARD_LENGTH {
            for j in 0..BOARD_LENGTH {
                self.draw_tile((j, i), gamestate.board[i as usize][j as usize])?;
            }
        }
        Ok(())
    }

    fn draw_selected_piece(&mut self, coord: Option<Point>) -> Result<(), String> {
        if let Some((y, x)) = coord {
            let alpha = 0.95 * 255.0;
            let alpha = alpha as u8;
            self.draw_green_square((x, y), alpha)?;
        }

        Ok(())
    }

    fn draw_yellow_tile(&mut self, (y, x): Point) -> Result<(), String> {
        let alpha = TRANSPARENCY * 255.0;
        let alpha = alpha as u8;
        let color = Color::RGBA(170, 162, 86, alpha);
        self.draw_square((x, y), color)?;
        Ok(())
    }

    fn draw_selected_tile(&mut self, (y, x): Point) -> Result<(), String> {
        let alpha = TRANSPARENCY * 255.0;
        let alpha = alpha as u8;
        self.draw_green_square((x, y), alpha)?;

        Ok(())
    }

    fn draw_green_square(&mut self, point: Point, alpha: u8) -> Result<(), String> {
        self.draw_square(point, Color::RGBA(107, 111, 70, alpha))?;
        Ok(())
    }

    fn draw_square(&mut self, (x, y): Point, color: Color) -> Result<(), String> {
        let x = x * TILE_SIZE;
        let y = y * TILE_SIZE;

        self.canvas.set_draw_color(color);
        self.canvas
            .fill_rect(Rect::new(x, y, TILE_SIZE as u32, TILE_SIZE as u32))?;
        Ok(())
    }

    fn draw_king_threatened(&mut self, (y, x): Point) -> Result<(), String> {
        let x = x * TILE_SIZE;
        let y = y * TILE_SIZE;

        let circle_x = x + TILE_SIZE / 2;
        let circle_y = y + TILE_SIZE / 2;
        let circle_rad = TILE_SIZE as f64 / 2.0;

        self.draw_blury_circle((x, y), (circle_x, circle_y), circle_rad)?;
        Ok(())
    }

    fn draw_blury_circle(
        &mut self,
        (x, y): Point,
        (circle_x, circle_y): Point,
        circle_rad: f64,
    ) -> Result<(), String> {
        for i in x..=(x + TILE_SIZE) {
            for j in y..=(y + TILE_SIZE) {
                let distance = (((i - circle_x).pow(2) + (j - circle_y).pow(2)) as f64).sqrt();
                if distance < circle_rad {
                    let alpha = 255.0 * (1.0 - (distance / circle_rad));
                    let alpha = alpha as u8;
                    self.canvas.set_draw_color(Color::RGBA(255, 0, 0, alpha));
                    self.canvas.draw_point(rect::Point::new(i, j))?;
                }
            }
        }

        Ok(())
    }

    fn draw_occupied_move_tile(&mut self, (x, y): (i32, i32)) -> Result<(), String> {
        let x = x * TILE_SIZE;
        let y = y * TILE_SIZE;
        let color = Color::RGBA(110, 110, 70, (u8::MAX as f64 * TRANSPARENCY) as u8);

        let circle_x = x + TILE_SIZE / 2;
        let circle_y = y + TILE_SIZE / 2;
        let circle_rad = TILE_SIZE as f64 / 1.75;
        self.canvas.set_draw_color(color);
        for i in x..=(x + TILE_SIZE) {
            for j in y..=(y + TILE_SIZE) {
                if (((i - circle_x).pow(2) + (j - circle_y).pow(2)) as f64).sqrt() > circle_rad {
                    self.canvas.draw_point(rect::Point::new(i, j))?;
                }
            }
        }

        Ok(())
    }

    fn draw_empty_move_tile(&mut self, (x, y): (i32, i32)) -> Result<(), String> {
        let x = x * TILE_SIZE;
        let y = y * TILE_SIZE;

        let circle_x = x + TILE_SIZE / 2;
        let circle_y = y + TILE_SIZE / 2;
        let circle_rad = TILE_SIZE / 8;

        self.canvas.filled_circle(
            circle_x as i16,
            circle_y as i16,
            circle_rad as i16,
            Color::RGBA(110, 110, 70, (u8::MAX as f64 * TRANSPARENCY) as u8),
        )?;
        Ok(())
    }

    fn draw_tile(&mut self, (x, y): (i32, i32), tile: Tile) -> Result<(), String> {
        if let Tile::Piece(piece) = tile {
            let x = x * TILE_SIZE;
            let y = y * TILE_SIZE;
            self.canvas.copy(
                self.images.get(piece),
                None,
                Rect::new(x, y, TILE_SIZE as u32, TILE_SIZE as u32),
            )?;
        }
        Ok(())
    }

    fn draw_moving_piece(&mut self, piece: Option<(Piece, Point)>) -> Result<(), String> {
        if let Some((piece, (x, y))) = piece {
            let half_tile_size = TILE_SIZE / 2;
            self.canvas.copy(
                self.images.get(piece),
                None,
                Rect::new(
                    x - half_tile_size,
                    y - half_tile_size,
                    TILE_SIZE as u32,
                    TILE_SIZE as u32,
                ),
            )?;
        }
        Ok(())
    }
}
