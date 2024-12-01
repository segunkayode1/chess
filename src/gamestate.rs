use std::collections::HashSet;

use crate::{util::*, BOARD_LENGTH};

#[derive(Clone)]
struct Selection {
    starting_tile: Point,
    current_point: Point,
    piece: Piece,
    held_down: bool,
}

#[derive(Clone)]
pub struct GameState {
    pub board: Board,
    pub last_move: Option<Move>,
    selected: Option<Selection>,
    players_turn: Color,
    prev_game_state: Option<Box<GameState>>,
    white_pieces: HashSet<Point>,
    black_pieces: HashSet<Point>,
    moves_since: i32,
}

impl GameState {
    pub fn new() -> Self {
        let mut game_state = Self {
            board: Vec::new(),
            last_move: None,
            selected: None,
            players_turn: Color::White,
            prev_game_state: None,
            white_pieces: HashSet::new(),
            black_pieces: HashSet::new(),
            moves_since: 0,
        };
        game_state.intialise_new_board();
        game_state
    }
    pub fn slected_piece_coord(&self) -> Option<Point> {
        self.selected.as_ref().map(|s| s.starting_tile)
    }

    pub fn is_empty(&self, (x, y): Point) -> bool {
        Tile::Empty == self.board[x as usize][y as usize]
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) {
        let (board_x, board_y) = get_board_position((x, y));
        if let Some(ref selected) = self.selected {
            if self.is_valid_tile(board_x, board_y) {
                self.move_piece(Move::new(selected.starting_tile, (board_x, board_y)));
                return;
            }
        }
        self.select_tile((x, y));
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) {
        let (board_x, board_y) = get_board_position((x, y));
        if let Some(
            ref selected @ Selection {
                held_down: true, ..
            },
        ) = self.selected
        {
            if self.is_valid_tile(board_x, board_y) {
                self.move_piece(Move::new(selected.starting_tile, (board_x, board_y)));
            } else {
                self.selected
                    .as_mut()
                    .expect("checked in if statement")
                    .held_down = false;
            }
        }
    }

    pub fn mouse_move(&mut self, x: i32, y: i32) {
        if let Some(ref mut selected) = self.selected {
            selected.current_point = (x, y);
        }
    }

    pub fn legal_moves(&self) -> HashSet<Point> {
        if let Some(ref selected) = self.selected {
            let selected_tile = MovingPiece::new(selected.piece, selected.starting_tile);
            self.valid_piece_moves(&selected_tile)
        } else {
            HashSet::new()
        }
    }

    pub fn select_tile(&mut self, (pos_x, pos_y): (i32, i32)) {
        let (board_x, board_y) = get_board_position((pos_x, pos_y));
        if !in_bounds((board_x, board_y)) {
            return;
        }
        if let Tile::Piece(piece) = self.board[board_x as usize][board_y as usize] {
            if piece.color == self.players_turn {
                self.selected = Some(Selection {
                    starting_tile: (board_x, board_y),
                    current_point: (pos_x, pos_y),
                    piece,
                    held_down: true,
                });
            }
        }
    }

    pub fn get_moving_piece(&self) -> Option<(Piece, Point)> {
        if let Some(Selection {
            held_down,
            piece,
            current_point,
            ..
        }) = self.selected
        {
            if held_down {
                return Some((piece, current_point));
            }
        }
        None
    }

    pub fn intialise_new_board(&mut self) {
        self.board = vec![vec![Tile::Empty; BOARD_LENGTH as usize]; BOARD_LENGTH as usize];
        for black in [true, false] {
            let back_rank = flip_rank(0, black);
            let pawn_rank = flip_rank(1, black);

            for file in 0..BOARD_LENGTH {
                self.add_tile(
                    back_rank,
                    file,
                    Tile::Piece(Piece::new(file_to_piece(file), black)),
                );
                self.add_tile(
                    pawn_rank,
                    file,
                    Tile::Piece(Piece::new(PieceState::Pawn, black)),
                );
            }
        }
    }

    pub fn end_game(&self) -> PlayStatus {
        use PlayStatus::*;
        if self.in_check_mate() {
            return Win(if self.players_turn == Color::Black {
                Color::White
            } else {
                Color::Black
            });
        }

        if self.fifty_move_rule() {
            return Draw;
        }
        if self.in_stalemate() {
            return Draw;
        }

        Continue
    }
    pub fn in_check(&self) -> bool {
        self.in_check_color(self.players_turn)
    }

    pub fn get_king(&self) -> Point {
        self.get_king_color(self.players_turn)
    }

    pub fn get_selected_tile(&self) -> Option<Point> {
        self.selected.as_ref().map(
            |Selection {
                 current_point: (x, y),
                 ..
             }| get_board_position((*x, *y)),
        )
    }

    fn get_king_color(&self, color: Color) -> Point {
        let pieces = if color == Color::Black {
            &self.black_pieces
        } else {
            &self.white_pieces
        };
        for (x, y) in pieces {
            if let Tile::Piece(Piece {
                state: PieceState::King,
                ..
            }) = self.board[*x as usize][*y as usize]
            {
                return (*x, *y);
            }
        }
        panic!("No king on board!")
    }

    fn is_valid_move(&self, selected_move: Move) -> bool {
        let mut simulated_game = self.clone();
        simulated_game.move_piece(selected_move);
        !simulated_game.in_check_color(self.players_turn)
    }

    fn in_check_color(&self, color: Color) -> bool {
        let opp_color = if color == Color::Black {
            Color::White
        } else {
            Color::Black
        };

        self.all_pieces_moves(opp_color)
            .contains(&self.get_king_color(color))
    }

    fn in_check_mate(&self) -> bool {
        if self.in_check_color(self.players_turn) {
            let pieces = if self.players_turn == Color::Black {
                &self.black_pieces
            } else {
                &self.white_pieces
            };
            for (src_x, src_y) in pieces {
                let (src_x, src_y) = (*src_x, *src_y);
                if let Tile::Piece(piece) = self.board[src_x as usize][src_y as usize] {
                    for (dst_x, dst_y) in
                        self.all_piece_moves(&MovingPiece::new(piece, (src_x, src_y)))
                    {
                        if self.is_valid_move(Move::new((src_x, src_y), (dst_x, dst_y))) {
                            return false;
                        }
                    }
                }
            }

            true
        } else {
            false
        }
    }

    fn in_stalemate(&self) -> bool {
        self.all_valid_pieces_moves(self.players_turn).is_empty()
    }

    fn fifty_move_rule(&self) -> bool {
        self.moves_since >= 50
    }

    fn is_valid_tile(&self, x: i32, y: i32) -> bool {
        let tile = (x, y);
        self.legal_moves().contains(&tile)
    }

    fn make_tile_empty(&mut self, x: i32, y: i32) -> bool {
        let not_empty = if let Tile::Piece(Piece { color, .. }) = self.board[x as usize][y as usize]
        {
            if color == Color::Black {
                self.black_pieces.remove(&(x, y));
            } else {
                self.white_pieces.remove(&(x, y));
            }
            false
        } else {
            true
        };

        self.board[x as usize][y as usize] = Tile::Empty;
        not_empty
    }

    fn add_tile(&mut self, x: i32, y: i32, tile: Tile) -> bool {
        let not_empty = self.make_tile_empty(x, y);
        if let Tile::Piece(Piece { color, .. }) = tile {
            if color == Color::Black {
                self.black_pieces.insert((x, y));
            } else {
                self.white_pieces.insert((x, y));
            }
        }
        self.board[x as usize][y as usize] = tile;
        not_empty
    }

    fn move_piece(
        &mut self,
        selected_move @ Move {
            src: (src_x, src_y),
            dst: dst @ (dst_x, dst_y),
        }: Move,
    ) {
        if let Tile::Piece(piece) = self.board[src_x as usize][src_y as usize] {
            let prev_state = Some({
                let mut prev_state = Box::new(self.clone());
                prev_state.prev_game_state = None;
                prev_state
            });
            self.last_move = Some(selected_move);

            self.moves_since += 1;

            if let Piece {
                state: PieceState::Pawn,
                ..
            } = piece
            {
                if (dst_y - src_y).abs() == 1 && self.is_empty(dst) {
                    let direction = if self.players_turn == Color::Black {
                        -1
                    } else {
                        1
                    };
                    let new_dst_x = dst_x + direction;
                    self.make_tile_empty(new_dst_x, dst_y);
                    self.moves_since = 0;
                }
            }

            if let Piece {
                state: PieceState::King,
                ..
            } = piece
            {
                if (dst_y - src_y).abs() > 1 {
                    let file = if dst_y > src_y { 7 } else { 0 };
                    if let Tile::Piece(rook) = self.board[dst_x as usize][file] {
                        let direction = if dst_y > src_y { 1 } else { -1 };

                        self.add_tile(
                            src_x,
                            src_y + direction,
                            Tile::Piece(Piece {
                                has_moved: false,
                                ..rook
                            }),
                        );
                        self.add_tile(
                            src_x,
                            src_y + 2 * direction,
                            Tile::Piece(Piece {
                                has_moved: true,
                                ..piece
                            }),
                        );

                        self.make_tile_empty(src_x, src_y);
                        self.make_tile_empty(dst_x, file as i32);

                        self.change_players_turn();
                        self.selected = None;
                        self.prev_game_state = prev_state;
                        self.last_move = Some(Move::new((src_x, src_y), (dst_x, file as i32)));
                        return;
                    }
                }
            }

            if self.add_tile(
                dst_x,
                dst_y,
                Tile::Piece(Piece {
                    has_moved: true,
                    ..piece
                }),
            ) {
                self.moves_since = 0;
            }

            self.make_tile_empty(src_x, src_y);

            self.change_players_turn();
            self.selected = None;
            self.prev_game_state = prev_state;
        }
    }

    fn change_players_turn(&mut self) {
        self.players_turn = if self.players_turn == Color::Black {
            Color::White
        } else {
            Color::Black
        };
    }

    fn all_valid_pieces_moves(&self, color: Color) -> HashSet<Point> {
        let mut moves = HashSet::new();
        let pieces = if color == Color::Black {
            &self.black_pieces
        } else {
            &self.white_pieces
        };
        for (x, y) in pieces {
            if let Tile::Piece(piece) = self.board[*x as usize][*y as usize] {
                moves.extend(self.valid_piece_moves(&MovingPiece::new(piece, (*x, *y))))
            }
        }
        moves
    }

    fn all_pieces_moves(&self, color: Color) -> HashSet<Point> {
        let mut moves = HashSet::new();
        let pieces = if color == Color::Black {
            &self.black_pieces
        } else {
            &self.white_pieces
        };
        for (x, y) in pieces {
            if let Tile::Piece(piece) = self.board[*x as usize][*y as usize] {
                moves.extend(self.all_piece_moves(&MovingPiece::new(piece, (*x, *y))))
            }
        }
        moves
    }

    fn valid_piece_moves(&self, selected_piece: &MovingPiece) -> HashSet<Point> {
        let moves = self.all_piece_moves(selected_piece);
        moves
            .into_iter()
            .filter(|x| {
                let selected_move = Move::new(selected_piece.point.to_owned(), x.to_owned());
                self.is_valid_move(selected_move)
            })
            .collect()
    }

    fn all_piece_moves(&self, selected_piece: &MovingPiece) -> HashSet<Point> {
        let mut moves = HashSet::new();
        moves.extend(&self.piece_moves(selected_piece));
        moves.extend(&self.piece_attack_moves(selected_piece));
        moves
    }

    fn piece_moves(&self, selected_piece: &MovingPiece) -> HashSet<Point> {
        use PieceState::*;
        match selected_piece.piece.state {
            King => self.king_moves(selected_piece.point),
            Queen => self.queen_moves(selected_piece.point),
            Rook => self.rook_moves(selected_piece.point),
            Bishop => self.bishop_moves(selected_piece.point),
            Knight => self.knight_moves(selected_piece.point),
            Pawn => self.pawn_moves(selected_piece.point),
        }
    }

    fn piece_attack_moves(&self, selected_piece: &MovingPiece) -> HashSet<Point> {
        use PieceState::*;
        match selected_piece.piece.state {
            King => self.attack_king_moves(selected_piece.point),
            Queen => self.attack_queen_moves(selected_piece.point),
            Rook => self.attack_rook_moves(selected_piece.point),
            Bishop => self.attack_bishop_moves(selected_piece.point),
            Knight => self.attack_knight_moves(selected_piece.point),
            Pawn => self.attack_pawn_moves(selected_piece.point),
        }
    }

    fn get_tile_color(&self, (x, y): Point) -> Option<Color> {
        match self.board[x as usize][y as usize] {
            Tile::Empty => None,
            Tile::Piece(Piece { color, .. }) => Some(color),
        }
    }

    fn king_moves(&self, (x, y): Point) -> HashSet<Point> {
        let mut moves = HashSet::new();
        let dir = [1, -1, 0];
        for rank in dir {
            for file in dir {
                if rank == 0 && file == 0 {
                    continue;
                }
                let rank = rank + x;
                let file = file + y;
                if in_bounds((rank, file)) && self.is_empty((rank, file)) {
                    moves.insert((rank, file));
                }
            }
        }
        if let Tile::Piece(Piece {
            has_moved: true, ..
        }) = self.board[x as usize][y as usize]
        {
            return moves;
        }

        {
            let mut all_empty = true;
            for i in (y + 1)..(BOARD_LENGTH - 1) {
                if !self.is_empty((x, i)) {
                    all_empty = false;
                    break;
                }
            }
            if all_empty {
                if let Tile::Piece(Piece {
                    has_moved: false, ..
                }) = self.board[x as usize][7]
                {
                    moves.insert((x, 7));
                    moves.insert((x, y + 2));
                }
            }
        }

        {
            let mut all_empty = true;
            for i in 1..(y - 1) {
                if !self.is_empty((x, i)) {
                    all_empty = false;
                    break;
                }
            }
            if all_empty {
                if let Tile::Piece(Piece {
                    has_moved: false, ..
                }) = self.board[x as usize][0]
                {
                    moves.insert((x, 0));
                    moves.insert((x, y - 2));
                }
            }
        }

        moves
    }

    fn attack_king_moves(&self, point @ (x, y): Point) -> HashSet<Point> {
        let mut moves = HashSet::new();
        let dir = [1, -1, 0];
        for rank in dir {
            for file in dir {
                if rank == 0 && file == 0 {
                    continue;
                }
                let rank = rank + x;
                let file = file + y;
                if in_bounds((rank, file))
                    && self.is_enemy(
                        (rank, file),
                        self.get_tile_color(point).expect("should never panic"),
                    )
                {
                    moves.insert((rank, file));
                }
            }
        }
        moves
    }

    fn queen_moves(&self, starting_point: Point) -> HashSet<Point> {
        let mut moves = HashSet::new();
        moves.extend(&self.rook_moves(starting_point));
        moves.extend(&self.bishop_moves(starting_point));
        moves
    }

    fn attack_queen_moves(&self, starting_point: Point) -> HashSet<Point> {
        let mut moves = HashSet::new();
        moves.extend(&self.attack_rook_moves(starting_point));
        moves.extend(&self.attack_bishop_moves(starting_point));
        moves
    }

    fn rook_moves(&self, (x, y): Point) -> HashSet<Point> {
        let mut moves = HashSet::new();
        let directions = [vec![-1, 1], vec![0]];
        for xs in directions.iter() {
            for ys in directions.iter() {
                if xs == ys {
                    continue;
                }
                for x_dir in xs {
                    for y_dir in ys {
                        moves.extend(&self.add_till((x, y), (*x_dir, *y_dir)));
                    }
                }
            }
        }
        moves
    }

    fn attack_rook_moves(&self, point @ (x, y): Point) -> HashSet<Point> {
        let mut moves = HashSet::new();
        let directions = [vec![-1, 1], vec![0]];
        for xs in directions.iter() {
            for ys in directions.iter() {
                if xs == ys {
                    continue;
                }
                for x_dir in xs {
                    for y_dir in ys {
                        moves.extend(&self.add_when(
                            (x, y),
                            (*x_dir, *y_dir),
                            self.get_tile_color(point).expect("should never panic"),
                        ));
                    }
                }
            }
        }
        moves
    }

    fn bishop_moves(&self, (x, y): Point) -> HashSet<Point> {
        let mut moves = HashSet::new();
        let directions = [-1, 1];
        for x_dir in directions.iter() {
            for y_dir in directions.iter() {
                moves.extend(&self.add_till((x, y), (*x_dir, *y_dir)));
            }
        }
        moves
    }

    fn attack_bishop_moves(&self, point @ (x, y): Point) -> HashSet<Point> {
        let mut moves = HashSet::new();
        let directions = [-1, 1];
        for x_dir in directions.iter() {
            for y_dir in directions.iter() {
                moves.extend(&self.add_when(
                    (x, y),
                    (*x_dir, *y_dir),
                    self.get_tile_color(point).expect("should never panic"),
                ));
            }
        }
        moves
    }

    fn knight_moves(&self, (x, y): Point) -> HashSet<Point> {
        let mut moves = HashSet::new();
        let dir = [[1, -1], [2, -2]];
        for xs in dir {
            for ys in dir {
                if xs == ys {
                    continue;
                }

                for rank in xs {
                    for file in ys {
                        let rank = x + rank;
                        let file = y + file;
                        if in_bounds((rank, file)) && self.is_empty((rank, file)) {
                            moves.insert((rank, file));
                        }
                    }
                }
            }
        }
        moves
    }

    fn attack_knight_moves(&self, point @ (x, y): Point) -> HashSet<Point> {
        let mut moves = HashSet::new();
        let dir = [[1, -1], [2, -2]];
        for xs in dir {
            for ys in dir {
                if xs == ys {
                    continue;
                }

                for rank in xs {
                    for file in ys {
                        let rank = x + rank;
                        let file = y + file;
                        if in_bounds((rank, file))
                            && self.is_enemy(
                                (rank, file),
                                self.get_tile_color(point).expect("should never panic"),
                            )
                        {
                            moves.insert((rank, file));
                        }
                    }
                }
            }
        }
        moves
    }

    fn pawn_moves(&self, (x, y): Point) -> HashSet<Point> {
        let mut moves = HashSet::new();
        if let Tile::Piece(pawn) = self.board[x as usize][y as usize] {
            let mut direction = if pawn.color == Color::Black { 1 } else { -1 };
            if in_bounds((x + direction, y)) && self.is_empty((x + direction, y)) {
                moves.insert((x + direction, y));
            }
            if in_bounds((x + direction, y)) && self.is_empty((x + direction, y)) && !pawn.has_moved
            {
                direction *= 2;

                if in_bounds((x + direction, y)) && self.is_empty((x + direction, y)) {
                    moves.insert((x + direction, y));
                }
            }

            moves
        } else {
            moves
        }
    }

    fn was_there_enemy_pawn_move_ago(&self, (x, y): Point, color: Color) -> bool {
        self.prev_game_state.as_ref().map_or(false, |prev| {
            if let Tile::Piece(
                piece @ Piece {
                    state: PieceState::Pawn,
                    ..
                },
            ) = prev.board[x as usize][y as usize]
            {
                piece.color != color
            } else {
                false
            }
        })
    }

    fn attack_pawn_moves(&self, point @ (x, y): Point) -> HashSet<Point> {
        let mut moves = HashSet::new();
        if let Tile::Piece(pawn) = self.board[x as usize][y as usize] {
            let tile_color = self.get_tile_color(point).expect("should not panic");
            let direction = if pawn.color == Color::Black { 1 } else { -1 };
            if in_bounds((x + direction, y + 1))
                && self.is_enemy((x + direction, y + 1), tile_color)
            {
                moves.insert((x + direction, y + 1));
            }

            if in_bounds((x + direction, y - 1))
                && self.is_enemy((x + direction, y - 1), tile_color)
            {
                moves.insert((x + direction, y - 1));
            }

            if self.prev_game_state.is_some() {
                if in_bounds((x, y + 1))
                    && self.is_enemy((x, y + 1), tile_color)
                    && in_bounds((x + 2 * direction, y + 1))
                    && self.was_there_enemy_pawn_move_ago((x + 2 * direction, y + 1), tile_color)
                {
                    moves.insert((x + direction, y + 1));
                }

                if in_bounds((x, y - 1))
                    && self.is_enemy((x, y - 1), tile_color)
                    && in_bounds((x + 2 * direction, y - 1))
                    && self.was_there_enemy_pawn_move_ago((x + 2 * direction, y - 1), tile_color)
                {
                    moves.insert((x + direction, y - 1));
                }
            }

            moves
        } else {
            moves
        }
    }

    fn is_enemy(&self, (x, y): Point, color: Color) -> bool {
        if let Tile::Piece(piece) = self.board[x as usize][y as usize] {
            return piece.color != color;
        }
        false
    }

    fn add_till(&self, (start_x, start_y): Point, (x_dir, y_dir): Point) -> HashSet<Point> {
        let mut points = HashSet::new();
        let (mut curr_x, mut curr_y) = (start_x + x_dir, start_y + y_dir);
        while in_bounds((curr_x, curr_y)) && self.is_empty((curr_x, curr_y)) {
            points.insert((curr_x, curr_y));
            (curr_x, curr_y) = (curr_x + x_dir, curr_y + y_dir);
        }
        points
    }

    fn add_when(
        &self,
        (start_x, start_y): Point,
        (x_dir, y_dir): Point,
        color: Color,
    ) -> HashSet<Point> {
        let mut points = HashSet::new();
        let (mut curr_x, mut curr_y) = (start_x + x_dir, start_y + y_dir);
        while in_bounds((curr_x, curr_y)) && self.is_empty((curr_x, curr_y)) {
            (curr_x, curr_y) = (curr_x + x_dir, curr_y + y_dir);
        }
        if in_bounds((curr_x, curr_y)) && self.is_enemy((curr_x, curr_y), color) {
            points.insert((curr_x, curr_y));
        }

        points
    }
}
