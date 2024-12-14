#![feature(try_blocks)]
#![feature(let_chains)]

use std::fmt::{Display, Formatter};
use grid::Grid;
use piece::{Color, Piece, PieceKind};
use crate::coordinate::{Coordinate, File, Move, Offset, Rank, Side};

pub mod coordinate;
pub mod piece;
mod grid;

#[derive(Debug, Copy, Clone)]
pub enum GameOutcome {
    Decisive { won: Color, reason: WinReason },
    Draw(DrawReason),
}

#[derive(Debug, Copy, Clone)]
pub enum WinReason {
    Checkmate,
    Resignation,
}

impl Display for WinReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Resignation => write!(f, "resignation"),
            Self::Checkmate => write!(f, "checkmate"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum DrawReason {
    Agreement,
    Stalemate,
    ThreefoldRepetition,
    FivefoldRepetition,
    NoAdvancement,
    InsufficientMaterial,
}

impl Display for DrawReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DrawReason::Agreement => write!(f, "agreement"),
            DrawReason::Stalemate => write!(f, "stalemate"),
            DrawReason::ThreefoldRepetition => write!(f, "threefold repetition"),
            DrawReason::FivefoldRepetition => write!(f, "fivefold repetition"),
            DrawReason::NoAdvancement => write!(f, "lack of advancement in the position (50-move rule)"),
            DrawReason::InsufficientMaterial => write!(f, "there is no sufficient material to checkmate"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MoveError {
    GameHasOutcome(GameOutcome),
    IllegalMove,
    AmbiguousMove,
    DrawPending,
}

#[derive(Debug, Clone)]
pub struct Board {
    pub grid_history: Vec<Grid>,
    pub last_move: Option<Move>,
    pub stale_plies: u8,
    pub white_castle: (bool, bool),
    pub black_castle: (bool, bool),
    pub move_color: Color,
    pub game_outcome: Option<GameOutcome>,
    pub draw_pending: Option<(bool, Color)>,
}

macro_rules! row {
    ($($p:tt)* ) => {
        [$(piece!($p)),*]
    }
}

impl Default for Board {
    fn default() -> Self {
        Self {
            grid_history: vec![Grid([
                row!(R N B Q K B N R),
                row!(P P P P P P P P),
                row!(- - - - - - - -),
                row!(- - - - - - - -),
                row!(- - - - - - - -),
                row!(- - - - - - - -),
                row!(p p p p p p p p),
                row!(r n b q k b n r),
            ])],
            last_move: None,
            stale_plies: 0,
            white_castle: (true, true),
            black_castle: (true, true),
            move_color: Color::White,
            game_outcome: None,
            draw_pending: None,
        }
    }
}

impl Board {
    // fixme proper handling of incorrect FENs and / (separators)
    pub fn from_fen(raw: &str) -> Option<Self> {
        let mut grid = Grid::default();
        let mut white_castle = (false, false);
        let mut black_castle = (false, false);
        let mut move_color = Color::White;
        let mut stale_plies = 0;

        #[derive(Debug, Clone, Copy)]
        enum ParsingState {
            Position, ColorToMove, Useless, Castling, StalePlies
        }

        impl ParsingState {
            pub fn next(self) -> Option<Self> {
                Some(match self {
                    Self::Position => Self::ColorToMove,
                    Self::ColorToMove => Self::Castling,
                    Self::Castling => Self::Useless,
                    Self::Useless => Self::StalePlies,
                    Self::StalePlies => None?,
                })
            }
        }

        let mut state = ParsingState::Position;
        let mut coord_iter = Coordinate::iter().rev();
        let mut last_buf = String::new();
        for c in (0..raw.len()).map(|i| &raw[i..=i]) {
            if c == " " {
                if let Some(next) = state.next() {
                    state = next;
                    continue;
                } else {
                    break;
                };
            };

            match state {
                ParsingState::Position => {
                    if let Ok(skip) = c.parse::<usize>() && skip > 0 && skip < 9 {
                        for _ in 0..skip {
                            let _ = coord_iter.next();
                        };
                    } else if let Some(piece) = Piece::parse(c) {
                        grid[coord_iter.next().unwrap()] = Some(piece);
                    };
                },
                ParsingState::ColorToMove => {
                    // todo move to Color::parse
                    move_color = match c {
                        "b" => Color::Black,
                        _ => Color::White,
                    };
                },
                ParsingState::Castling => {
                    match c {
                        "K" => white_castle.1 = true,
                        "k" => black_castle.1 = true,
                        "Q" => white_castle.0 = true,
                        "q" => black_castle.0 = true,
                        _ => {},
                    };
                },
                ParsingState::StalePlies => {
                    last_buf.push_str(c);
                },
                ParsingState::Useless => {}
            };
        };
        
        stale_plies = last_buf.parse().unwrap();

        Some(Self {
            grid_history: vec![grid],
            white_castle,
            black_castle,
            move_color,
            stale_plies,
            ..Default::default()
        })
    }

    pub fn grid(&self) -> &Grid {
        unsafe { self.grid_history.last().unwrap_unchecked() }
    }

    pub fn grid_mut(&mut self) -> &mut Grid {
        unsafe { self.grid_history.last_mut().unwrap_unchecked() }
    }

    fn unchecked_for_check_possible_moves(&self, for_color: Color) -> Vec<Move> {
        let mut possible_moves = Vec::new();

        for (piece, coord) in self.grid().iter_coord()
            .filter_map(|(piece, coord)| piece.map(|piece| (piece, coord)))
            .filter_map(|(piece, coord)| (piece.color == for_color).then_some((piece.kind, coord))) {
            let _: Option<_> = try {
                match piece {
                    PieceKind::Pawn => {
                        // first move
                        {
                            if coord.rank == for_color.pawn_rank() {
                                let path = coord.checked_add_offset(Offset { vertical: for_color.direction(), horizontal: 0 }).unwrap();
                                let to = coord.checked_add_offset(Offset { vertical: for_color.direction()*2, horizontal: 0 }).unwrap();
                                if self.grid()[to].is_none() && self.grid()[path].is_none() {
                                    possible_moves.push(Move::Simple { from: coord, to });
                                };
                            };
                        }

                        // move forward
                        {
                            let _: Option<_> = try {
                                let to = coord.checked_add_offset(Offset { vertical: for_color.direction(), horizontal: 0 })?;
                                if self.grid()[to].is_none() {
                                    if to.rank != for_color.promotion_rank() {
                                        possible_moves.push(Move::Simple { from: coord, to });
                                    } else {
                                        for piece_kind in [PieceKind::Queen, PieceKind::Rook, PieceKind::Bishop, PieceKind::Knight] {
                                            possible_moves.push(Move::Promotion { from: coord.file, to: to.file, piece: piece_kind });
                                        };
                                    };
                                };
                            };
                        };

                        // move diagonally
                        {
                            for to in [1, -1].map(|of| coord.checked_add_offset(Offset { vertical: for_color.direction(), horizontal: of })) {
                                let _: Option<_> = try {
                                    let to = to?;

                                    if let Some(Piece { color, .. }) = self.grid()[to] && color == for_color.the_other() {
                                        if to.rank != for_color.promotion_rank() {
                                            possible_moves.push(Move::Simple { from: coord, to });
                                        } else {
                                            for piece_kind in [PieceKind::Queen, PieceKind::Rook, PieceKind::Bishop, PieceKind::Knight] {
                                                possible_moves.push(Move::Promotion { from: coord.file, to: to.file, piece: piece_kind });
                                            };
                                        };
                                    };
                                };
                            }
                        };

                        if coord.rank == for_color.en_passant_rank() &&
                            let Some(Move::Simple { from, to }) = self.last_move &&
                            self.grid_history[self.grid_history.len()-2][from].unwrap().kind == PieceKind::Pawn &&
                            to.rank == for_color.en_passant_rank() && (coord.file as i8 - to.file as i8).abs() == 1 {
                            possible_moves.push(Move::EnPassant { from: from.file, to: to.file });
                        };
                    },
                    PieceKind::Knight => {
                        for to in [
                            (2, 1), (2, -1),
                            (-2, 1), (-2, -1),
                            (1, 2), (-1, 2),
                            (1, -2), (-1, -2),
                        ].map(|of| coord.checked_add_offset(of.into())) {
                            let _: Option<_> = try {
                                let to = to?;
                                let square = self.grid()[to];
                                if square.is_none() || matches!(square, Some(Piece { color, .. }) if color == for_color.the_other()) {
                                    possible_moves.push(Move::Simple { from: coord, to });
                                };
                            };
                        };
                    },
                    PieceKind::Bishop => {
                        for of in [
                            (1, 1), (1, -1),
                            (-1, 1), (-1, -1),
                        ] {
                            let _: Option<_> = try {
                                let mut check_coord = coord.checked_add_offset(of.into())?;

                                while self.grid()[check_coord].is_none() {
                                    possible_moves.push(Move::Simple { from: coord, to: check_coord });
                                    check_coord = check_coord.checked_add_offset(of.into())?;
                                };

                                if let Some(Piece { color, .. }) = self.grid()[check_coord] && color == for_color.the_other() {
                                    possible_moves.push(Move::Simple { from: coord, to: check_coord });
                                };
                            };
                        }
                    },
                    PieceKind::Rook => {
                        for of in [
                            (0, 1), (0, -1),
                            (1, 0), (-1, 0),
                        ] {
                            let _: Option<_> = try {
                                let mut check_coord = coord.checked_add_offset(of.into())?;

                                while self.grid()[check_coord].is_none() {
                                    possible_moves.push(Move::Simple { from: coord, to: check_coord });
                                    check_coord = check_coord.checked_add_offset(of.into())?;
                                };

                                if let Some(Piece { color, .. }) = self.grid()[check_coord] && color == for_color.the_other() {
                                    possible_moves.push(Move::Simple { from: coord, to: check_coord });
                                };
                            };
                        }
                    },
                    PieceKind::Queen => {
                        for of in [
                            (0, 1), (0, -1),
                            (1, 0), (-1, 0),
                            (1, 1), (1, -1),
                            (-1, 1), (-1, -1),
                        ] {
                            let _: Option<_> = try {
                                let mut check_coord = coord.checked_add_offset(of.into())?;

                                while self.grid()[check_coord].is_none() {
                                    possible_moves.push(Move::Simple { from: coord, to: check_coord });
                                    check_coord = check_coord.checked_add_offset(of.into())?;
                                };

                                if let Some(Piece { color, .. }) = self.grid()[check_coord] && color == for_color.the_other() {
                                    possible_moves.push(Move::Simple { from: coord, to: check_coord });
                                };
                            };
                        }
                    },
                    PieceKind::King => {
                        for of in [
                            (0, 1), (0, -1),
                            (1, 0), (-1, 0),
                            (1, 1), (1, -1),
                            (-1, 1), (-1, -1),
                        ] {
                            let _: Option<_> = try {
                                let check_coord = coord.checked_add_offset(of.into())?;

                                let piece = self.grid()[check_coord];

                                if piece.is_none() || matches!(piece, Some(Piece { color, .. }) if color == for_color.the_other()) {
                                    possible_moves.push(Move::Simple { from: coord, to: check_coord });
                                };
                            };
                        };

                        let castle_perm = match for_color {
                            Color::White => self.white_castle,
                            Color::Black => self.black_castle,
                        };

                        if castle_perm.0 &&
                            !self.is_under_attack(for_color.the_other(), Coordinate { file: File::F, rank: for_color.home_rank() }, None) &&
                            [File::F, File::G].into_iter().all(|file| self.grid()[Coordinate { file, rank: for_color.home_rank() }].is_none()) {
                            possible_moves.push(Move::Castling { side: Side::King });
                        };

                        if castle_perm.1 &&
                            !self.is_under_attack(for_color.the_other(), Coordinate { file: File::D, rank: for_color.home_rank() }, None) &&
                            [File::D, File::C, File::B].into_iter().all(|file| self.grid()[Coordinate { file, rank: for_color.home_rank() }].is_none()) {
                            possible_moves.push(Move::Castling { side: Side::Queen });
                        };
                    },
                };
            };
        };

        possible_moves
    }

    pub fn is_under_attack(&self, by: Color, mut coord: Coordinate, after: Option<(Color, Move, bool)>) -> bool {
        // todo optional check if attacking piece is pinned
        
        let mut grid = self.grid().clone();
        
        if let Some((color, r#move, adapt)) = after {
            grid.r#move(r#move, color);
            if adapt && r#move.resolve_from(color) == coord {
                coord = r#move.resolve_to(color);
            };
        };

        // check for pawn attacks
        for coord in [-1, 1].map(|file_of| coord.checked_add_offset(Offset { vertical: -by.direction(), horizontal: file_of })) {
            let _: Option<_> = try {
                if let Some(Piece { kind: PieceKind::Pawn, color }) = grid[coord?] && color == by {
                    return true;
                };
            };
        };

        // check for knight attacks
        for coord in [
            (2, 1), (2, -1),
            (-2, 1), (-2, -1),
            (1, 2), (-1, 2),
            (1, -2), (-1, -2),
        ].map(|of| coord.checked_add_offset(of.into())) {
            let _: Option<_> = try {
                if let Some(Piece { kind: PieceKind::Knight, color }) = grid[coord?] && color == by {
                    return true;
                };
            };
        };

        // check for rook/queen attacks
        for of in [
            (0, 1), (0, -1),
            (1, 0), (-1, 0),
        ] {
            let _: Option<_> = try {
                let mut check_coord = coord.checked_add_offset(of.into())?;

                while grid[check_coord].is_none() {
                    check_coord = check_coord.checked_add_offset(of.into())?;
                };

                if let Some(Piece { kind: PieceKind::Rook | PieceKind::Queen, color }) = grid[check_coord] && color == by {
                    return true;
                };
            };
        };

        // check for bishop/queen attacks
        for of in [
            (1, 1), (1, -1),
            (-1, 1), (-1, -1),
        ] {
            let _: Option<_> = try {
                let mut check_coord = coord.checked_add_offset(of.into())?;

                while grid[check_coord].is_none() {
                    check_coord = check_coord.checked_add_offset(of.into())?;
                };

                if let Some(Piece { kind: PieceKind::Bishop | PieceKind::Queen, color }) = grid[check_coord] && color == by {
                    return true;
                };
            };
        };

        // check for king attacks
        for coord in [
            (0, 1), (0, -1),
            (1, 0), (-1, 0),
            (1, 1), (1, -1),
            (-1, 1), (-1, -1),
        ].map(|of| coord.checked_add_offset(of.into())) {
            let _: Option<_> = try {
                if let Some(Piece { kind: PieceKind::King, color }) = grid[coord?] && color == by {
                    return true;
                };
            };
        };

        false
    }

    fn find_piece(&self, piece: Piece) -> Option<Coordinate> {
        self.grid().iter_coord().filter_map(|(piece, coord)| piece.map(|p| (p, coord)))
            .find(|(cpiece, _)| *cpiece == piece).map(|(_, coord)| coord)
    }

    pub fn possible_moves(&self, color: Color) -> Vec<Move> {
        let king_coord = self.find_piece(Piece { kind: PieceKind::King, color }).unwrap_or_else(|| {
            eprintln!("{self}");
            eprintln!("states before:");
            for grid in self.grid_history.iter().rev().skip(1) {
                eprintln!("{grid}");
            };
            panic!("KING HAS GONE WILD");
        });
        self.unchecked_for_check_possible_moves(color)
            .into_iter()
            .filter(|r#move| !self.is_under_attack(color.the_other(), king_coord, Some((color, *r#move, true))))
            .collect()
    }

    fn is_material_sufficient_for_checkmate(&self) -> bool {
        let mut white_bishop_found = false;
        let mut black_bishop_found = false;
        let mut white_knight_found = false;
        let mut black_knight_found = false;
        for piece in self.grid().iter_coord().filter_map(|(p, _)| p).filter(|p| p.kind != PieceKind::King) {
            match piece {
                Piece { kind: PieceKind::Bishop, color: Color::White } => white_bishop_found = true,
                Piece { kind: PieceKind::Bishop, color: Color::Black } => black_bishop_found = true,
                Piece { kind: PieceKind::Knight, color: Color::White } => white_knight_found = true,
                Piece { kind: PieceKind::Knight, color: Color::Black } => black_knight_found = true,
                Piece { color: Color::White, .. } => return true,
                Piece { color: Color::Black, .. } => return true,
            };
        };

        (white_knight_found && white_bishop_found) || (black_knight_found && black_bishop_found)
    }

    fn handle_castling_rights_update(&mut self, color: Color, r#move: Move) {
        let castling_rights = match color {
            Color::White => &mut self.white_castle,
            Color::Black => &mut self.black_castle,
        };

        match r#move {
            Move::Castling { .. } => *castling_rights = (false, false),
            Move::Simple { from: Coordinate { file: File::E, rank  }, .. } if rank == color.home_rank() => *castling_rights = (false, false), 
            Move::Simple { from: Coordinate { file: File::H, rank }, .. } if rank == color.home_rank() => castling_rights.0 = false,
            Move::Simple { from: Coordinate { file: File::A, rank }, .. } if rank == color.home_rank() => castling_rights.1 = false,
            _ => {}
        };
    }
    
    pub fn play_move(&mut self, r#move: PlayerMove) -> Result<Option<GameOutcome>, MoveError> {
        if let Some(game_outcome) = self.game_outcome {
            return Err(MoveError::GameHasOutcome(game_outcome));
        };

        if self.draw_pending.is_some() {
            return Err(MoveError::DrawPending);
        };

        let color_to_move = self.move_color;
        let advancing_move;
        match r#move {
            PlayerMove::Internal(r#move) => {
                if self.possible_moves(self.move_color).into_iter().any(|legal_move| legal_move == r#move) {
                    self.grid_history.push(self.grid().clone());
                    advancing_move = !self.grid_mut().r#move(r#move, color_to_move);
                    self.handle_castling_rights_update(color_to_move, r#move);
                } else {
                    return Err(MoveError::IllegalMove);
                };
            },
            PlayerMove::Long { from, to, promotion } => {
                if let Some(r#move) = self.possible_moves(self.move_color).into_iter().find(|legal_move| legal_move.resolve_from(self.move_color) == from && legal_move.resolve_to(self.move_color) == to && match legal_move { Move::Promotion { piece, .. } => promotion.is_some() && *piece == promotion.unwrap(), _ => true }) {
                    self.grid_history.push(self.grid().clone());
                    advancing_move = self.grid_mut().r#move(r#move, color_to_move);
                    self.handle_castling_rights_update(color_to_move, r#move);
                } else {
                    return Err(MoveError::IllegalMove);
                };
            },
            // PlayerMove::Short { piece, to, from } => {
            //     let possible_moves = self.possible_moves().into_iter().filter(|legal_move| {
            //         let move_from = legal_move.resolve_from(self.move_color);
            //         matches!(self.grid()[legal_move.resolve_from(self.move_color)], Some(Piece { kind, color }) if color == self.move_color && kind == piece) &&
            //             legal_move.resolve_to(self.move_color) == to && from.0.is_none_or(|file| move_from.file == file) && from.1.is_none_or(|rank| move_from.rank == rank)
            //     }).collect::<Vec<_>>();
            //
            //     match possible_moves.len() {
            //         0 => return Err(MoveError::IllegalMove),
            //         1 => {
            //             self.grid_history.push(self.grid().clone());
            //             self.grid_mut().r#move(*possible_moves.first().unwrap(), color_to_move);
            //         },
            //         _ => return Err(MoveError::AmbiguousMove),
            //     };
            // },
            PlayerMove::Short { .. } => todo!(),
        };

        if !advancing_move {
            self.stale_plies += 1;
        } else {
            self.stale_plies = 0;
        };

        if self.possible_moves(self.move_color.the_other()).is_empty() {
            let enemy_king_pos = self.find_piece(Piece { color: self.move_color.the_other(), kind: PieceKind::King }).unwrap();
            if self.is_under_attack(self.move_color, enemy_king_pos, None) {
                self.game_outcome = Some(GameOutcome::Decisive { won: self.move_color, reason: WinReason::Checkmate });
            } else {
                self.game_outcome = Some(GameOutcome::Draw(DrawReason::Stalemate));
            };
        } else if self.stale_plies == 100 {
            self.game_outcome = Some(GameOutcome::Draw(DrawReason::NoAdvancement));
        } else if !self.is_material_sufficient_for_checkmate() {
            self.game_outcome = Some(GameOutcome::Draw(DrawReason::InsufficientMaterial));
        } else {
            match self.grid_history.iter().filter(|position| *position == self.grid()).count() {
                3 => self.draw_pending = Some((true, color_to_move)),
                5 => self.game_outcome = Some(GameOutcome::Draw(DrawReason::FivefoldRepetition)),
                _ => {}
            };
        };

        if self.game_outcome.is_none() {
            self.move_color = color_to_move.the_other();
        };

        Ok(self.game_outcome)
    }
    
    pub fn propose_draw(&mut self, by: Color) {
        if matches!(self.draw_pending, Some((_, color)) if color.the_other() == by) {
            if let Some((true, _)) = self.draw_pending {
                self.game_outcome = Some(GameOutcome::Draw(DrawReason::ThreefoldRepetition));
            } else {
                self.game_outcome = Some(GameOutcome::Draw(DrawReason::Agreement));
            };
        } else {
            self.draw_pending = Some((false, by));
        };
    }
    
    pub fn decline_draw(&mut self) {
        self.draw_pending = None;
    }

    pub fn resign(&mut self, by: Color) {
        self.game_outcome = Some(GameOutcome::Decisive { won: by.the_other(), reason: WinReason::Resignation });
    }
}

#[derive(Debug, Clone)]
pub enum PlayerMove {
    Internal(Move),
    Long {
        from: Coordinate,
        to: Coordinate,
        promotion: Option<PieceKind>,
    },
    Short {
        piece: PieceKind,
        to: (File, Option<Rank>),
        from: (Option<File>, Option<Rank>),
        capture: bool,
        promotion: Option<PieceKind>,
    },
}

impl PlayerMove {
    pub fn parse(raw: &str) -> Option<Self> {
        if raw.len() == 4 &&
            let Some(from) = Coordinate::parse(&raw[0..2]) &&
            let Some(to) = Coordinate::parse(&raw[2..4]) {
            return Some(Self::Long { from, to, promotion: None });
        };

        if raw.len() == 5 &&
            let Some(from) = Coordinate::parse(&raw[0..2]) &&
            let Some(to) = Coordinate::parse(&raw[2..4]) &&
            let Some(piece_kind) = PieceKind::parse(&raw[4..5]) {
            return Some(Self::Long { from, to, promotion: Some(piece_kind) });
        };

        match &raw[0..1] {
            "=" if raw.len() == 4 => {
                if let Some(from) = File::parse(&raw[1..2]) &&
                    let Some(to) = File::parse(&raw[2..3]) &&
                    let Some(piece) = PieceKind::parse(&raw[3..4]) {
                    return Some(Self::Internal(Move::Promotion { from, to, piece }));
                };
            },
            "~" if raw.len() == 3 => {
                if let Some(from) = File::parse(&raw[1..2]) &&
                    let Some(to) = File::parse(&raw[2..3]) {
                    return Some(Self::Internal(Move::EnPassant { from, to }));
                };
            },
            // i mean, this technically also parses "cabc", but who cares
            "c" => match raw.len() {
                4 => return Some(Self::Internal(Move::Castling { side: Side::King })),
                6 => return Some(Self::Internal(Move::Castling { side: Side::Queen })),
                _ => {},
            },
            _ => {},
        };

        None
    }
}

impl Display for PlayerMove {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Internal(r#move) => write!(f, "{move}")?,
            Self::Long { from, to, promotion } => {
                write!(f, "{from}{to}")?;
                if let Some(piece) = promotion {
                    write!(f, "{piece}")?;
                };
            },
            _ => todo!()
        };
        Ok(())
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\nmove #{} (ply #{}), {}'s turn:\n{}", self.stale_plies, self.stale_plies.div_ceil(2), self.move_color, self.grid())
    }
}

impl Display for Grid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "  ")?;
        for rank in 0..8 {
            write!(f, "{} ", File::try_from(rank).unwrap())?;
        };
        writeln!(f)?;

        let mut even_row = false;
        for (i, (piece, Coordinate { rank, .. })) in self.iter_coord().rev().enumerate() {
            if i % 8 == 0 {
                even_row = !even_row;
                if i != 0 {
                    write!(f, " {}\n{rank} ", (rank + 1).unwrap())?;
                } else {
                    write!(f, "{rank} ")?;
                };
            };

            let bg_code = if (i % 2 == 0)^even_row { "100" } else { "47" };

            if let Some(piece) = piece {
                write!(f, "\x1B[{bg_code}m{piece}")?;
            } else {
                write!(f, "\x1B[{bg_code}m  \x1B[0m")?;
            };
        };

        write!(f, " 1\n  ")?;
        for rank in 0..8 {
            write!(f, "{} ", File::try_from(rank).unwrap())?;
        };

        Ok(())
    }
} 
