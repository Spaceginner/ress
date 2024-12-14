use std::fmt::{Display, Formatter};
use crate::coordinate::Rank;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceKind {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "p" => Some(Self::Pawn),
            "n" => Some(Self::Knight),
            "b" => Some(Self::Bishop),
            "r" => Some(Self::Rook),
            "q" => Some(Self::Queen),
            "k" => Some(Self::King),
            _ => None,
        }
    }
}

impl Display for PieceKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PieceKind::Pawn => write!(f, "🨅"),
            PieceKind::Knight => write!(f, "🨄"),
            PieceKind::Bishop => write!(f, "🨃"),
            PieceKind::Rook => write!(f, "🨂"),
            PieceKind::Queen => write!(f, "🨁"),
            PieceKind::King => write!(f, "🨀"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    Black,
    White,
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::White => write!(f, "white"),
            Color::Black => write!(f, "black"),
        }
    }
}

impl Color {
    pub fn the_other(self) -> Self {
        match self {
            Self::Black => Self::White,
            Self::White => Self::Black,
        }
    }
    
    pub fn direction(self) -> i8 {
        match self {
            Color::White => 1,
            Color::Black => -1,
        }
    }
    
    pub fn home_rank(self) -> Rank {
        match self {
            Self::White => Rank::First,
            Self::Black => Rank::Eighth,
        }
    }
    
    pub fn pawn_rank(self) -> Rank {
        match self {
            Self::White => Rank::Second,
            Self::Black => Rank::Seventh,
        }
    }
    
    pub fn prepromotion_rank(self) -> Rank {
        match self {
            Self::White => Rank::Seventh,
            Self::Black => Rank::Second,
        }
    }
    
    pub fn promotion_rank(self) -> Rank {
        match self {
            Self::White => Rank::Eighth,
            Self::Black => Rank::First,
        }
    }
    
    pub fn en_passant_rank(self) -> Rank {
        match self {
            Self::White => Rank::Fifth,
            Self::Black => Rank::Fourth,
        }
    }
    
    pub fn unpassable_rank(self) -> Rank {
        match self {
            Self::White => Rank::Sixth,
            Self::Black => Rank::Third,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Piece {
    pub kind: PieceKind,
    pub color: Color,
}

#[macro_export]
macro_rules! piece {
    (-) => { None };
    (P) => { Some(Piece { kind: PieceKind::Pawn,   color: Color::White }) };
    (N) => { Some(Piece { kind: PieceKind::Knight, color: Color::White }) };
    (B) => { Some(Piece { kind: PieceKind::Bishop, color: Color::White }) };
    (R) => { Some(Piece { kind: PieceKind::Rook,   color: Color::White }) };
    (Q) => { Some(Piece { kind: PieceKind::Queen,  color: Color::White }) };
    (K) => { Some(Piece { kind: PieceKind::King,   color: Color::White }) };
    (p) => { Some(Piece { kind: PieceKind::Pawn,   color: Color::Black }) };
    (n) => { Some(Piece { kind: PieceKind::Knight, color: Color::Black }) };
    (b) => { Some(Piece { kind: PieceKind::Bishop, color: Color::Black }) };
    (r) => { Some(Piece { kind: PieceKind::Rook,   color: Color::Black }) };
    (q) => { Some(Piece { kind: PieceKind::Queen,  color: Color::Black }) };
    (k) => { Some(Piece { kind: PieceKind::King,   color: Color::Black }) };
}

impl Display for Piece {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (offset, color_code) = match self.color {
            Color::White => (0, "255;255;255"),
            Color::Black => (6, "0;0;0"),
        };

        write!(f, "\x1B[38;2;{color_code}m{} \x1B[0m", char::from_u32(match self.kind {
            PieceKind::Pawn => '♙',
            PieceKind::Knight => '♘',
            PieceKind::Bishop => '♗',
            PieceKind::Rook => '♖',
            PieceKind::Queen => '♕',
            PieceKind::King => '♔',
        } as u32 + offset).unwrap())
    }
}

impl Piece {
    pub fn parse(raw: &str) -> Option<Self> {
        Some(match raw {
            "P" => Self { kind: PieceKind::Pawn, color: Color::White },
            "N" => Self { kind: PieceKind::Knight, color: Color::White },
            "B" => Self { kind: PieceKind::Bishop, color: Color::White },
            "R" => Self { kind: PieceKind::Rook, color: Color::White },
            "Q" => Self { kind: PieceKind::Queen, color: Color::White },
            "K" => Self { kind: PieceKind::King, color: Color::White },
            "p" => Self { kind: PieceKind::Pawn, color: Color::Black },
            "n" => Self { kind: PieceKind::Knight, color: Color::Black },
            "b" => Self { kind: PieceKind::Bishop, color: Color::Black },
            "r" => Self { kind: PieceKind::Rook, color: Color::Black },
            "q" => Self { kind: PieceKind::Queen, color: Color::Black },
            "k" => Self { kind: PieceKind::King, color: Color::Black },
            _ => None?
        })
    }
}
