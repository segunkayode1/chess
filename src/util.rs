use crate::BOARD_LENGTH;
use crate::TILE_SIZE;
use std::hash::Hash;

pub struct MovingPiece {
    pub piece: Piece,
    pub point: Point,
}

impl MovingPiece {
    pub fn new(piece: Piece, point: Point) -> Self {
        Self { piece, point }
    }
}
pub enum PlayStatus {
    Continue,
    Win(Color),
    Draw,
}

pub type Point = (i32, i32);

pub fn in_bounds((x, y): Point) -> bool {
    x >= 0 && y >= 0 && x < BOARD_LENGTH && y < BOARD_LENGTH
}

#[derive(Clone)]
pub struct Move {
    pub src: Point,
    pub dst: Point,
}

impl Move {
    pub fn new(src: Point, dst: Point) -> Self {
        Self { src, dst }
    }
}
#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub enum Color {
    Black,
    White,
}

pub fn get_board_position((x, y): (i32, i32)) -> (i32, i32) {
    (y / TILE_SIZE, x / TILE_SIZE)
}

#[derive(Eq, Hash, Debug, Copy, Clone, PartialEq)]
pub enum PieceState {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

#[derive(Eq, Copy, Clone, PartialEq, Debug)]
pub struct Piece {
    pub state: PieceState,
    pub color: Color,
    pub has_moved: bool,
}

impl Piece {
    pub fn new(state: PieceState, black: bool) -> Self {
        Self {
            state,
            color: if black { Color::Black } else { Color::White },
            has_moved: false,
        }
    }
}

#[derive(Eq, Copy, Clone, PartialEq, Debug)]
pub enum Tile {
    Empty,
    Piece(Piece),
}

pub type Board = Vec<Vec<Tile>>;

pub const fn flip(x: i32) -> i32 {
    7 - x
}

pub fn flip_rank(rank: i32, black: bool) -> i32 {
    if black {
        rank
    } else {
        flip(rank)
    }
}

pub fn file_to_piece(file: i32) -> PieceState {
    use PieceState::*;

    const FLIP_ZERO: i32 = flip(0);
    const FLIP_ONE: i32 = flip(1);
    const FLIP_TWO: i32 = flip(2);

    match file {
        0 | FLIP_ZERO => Rook,
        1 | FLIP_ONE => Knight,
        2 | FLIP_TWO => Bishop,
        3 => Queen,
        4 => King,
        _ => panic!("Should never be any other number"),
    }
}
