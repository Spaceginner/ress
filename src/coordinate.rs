use std::fmt::{Display, Formatter};
use std::ops::{Add, Sub};
use crate::piece::{Color, Piece, PieceKind};

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Rank {
    First = 0,
    Second = 1,
    Third,
    Fourth,
    Fifth,
    Sixth,
    Seventh,
    Eighth,
}

impl Rank {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "1" => Some(Rank::First),
            "2" => Some(Rank::Second),
            "3" => Some(Rank::Third),
            "4" => Some(Rank::Fourth),
            "5" => Some(Rank::Fifth),
            "6" => Some(Rank::Sixth),
            "7" => Some(Rank::Seventh),
            "8" => Some(Rank::Eighth),
            _ => None,
        }
    }
}

impl TryFrom<i8> for Rank {
    type Error = ();

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::First),
            1 => Ok(Self::Second),
            2 => Ok(Self::Third),
            3 => Ok(Self::Fourth),
            4 => Ok(Self::Fifth),
            5 => Ok(Self::Sixth),
            6 => Ok(Self::Seventh),
            7 => Ok(Self::Eighth),
            _ => Err(())
        }
    }
}

impl Add<i8> for Rank {
    type Output = Option<Rank>;

    fn add(self, rhs: i8) -> Self::Output {
        Self::try_from(self as i8 + rhs).ok()
    }
}

impl Sub<i8> for Rank {
    type Output = Option<Rank>;

    fn sub(self, rhs: i8) -> Self::Output {
        self + (-rhs)
    }
}

impl Display for Rank {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self as i8 + 1)
    }
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum File {
    A = 0,
    B = 1,
    C,
    D,
    E,
    F,
    G,
    H,
}

impl File {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "a" => Some(File::A),
            "b" => Some(File::B),
            "c" => Some(File::C),
            "d" => Some(File::D),
            "e" => Some(File::E),
            "f" => Some(File::F),
            "g" => Some(File::G),
            "h" => Some(File::H),
            _ => None,
        }
    }
}

impl TryFrom<i8> for File {
    type Error = ();
    
    fn try_from(value: i8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::A),
            1 => Ok(Self::B),
            2 => Ok(Self::C),
            3 => Ok(Self::D),
            4 => Ok(Self::E),
            5 => Ok(Self::F),
            6 => Ok(Self::G),
            7 => Ok(Self::H),
            _ => Err(())
        }
    }
}

impl Add<i8> for File {
    type Output = Option<File>;

    fn add(self, rhs: i8) -> Self::Output {
        Self::try_from(self as i8 + rhs).ok()
    }
}

impl Sub<i8> for File {
    type Output = Option<File>;

    fn sub(self, rhs: i8) -> Self::Output {
        self + (-rhs)
    }
}

impl Display for File {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match *self {
            File::A => "a",
            File::B => "b",
            File::C => "c",
            File::D => "d",
            File::E => "e",
            File::F => "f",
            File::G => "g",
            File::H => "h",
        })
    }
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub struct Coordinate {
    pub file: File,
    pub rank: Rank,
}

impl Display for Coordinate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.file, self.rank)
    }
}

#[derive(Clone, Debug, Copy)]
pub struct Offset {
    pub vertical: i8,
    pub horizontal: i8,
}

impl From<(i8, i8)> for Offset {
    fn from(of: (i8, i8)) -> Self {
        Self {
            horizontal: of.0,
            vertical: of.1,
        }
    }
}

impl Coordinate {
    pub fn next(self) -> Option<Self> {
        if let Some(file) = self.file + 1 {
            Some(Self { file, ..self })
        } else {
            (self.rank + 1).map(|rank| Self { file: File::A, rank })
        }
    }
    
    pub fn back_next(self) -> Option<Self> {
        if let Some(file) = self.file + 1 {
            Some(Self { file, ..self })
        } else {
            (self.rank - 1).map(|rank| Self { file: File::A, rank })
        }
    }
    
    pub fn iter() -> Iter {
        Iter::new()
    }
    
    pub fn checked_add_offset(self, offset: Offset) -> Option<Self> {
        Some(Self {
            file: (self.file + offset.horizontal)?,
            rank: (self.rank + offset.vertical)?,
        })
    }
    
    pub fn parse(raw: &str) -> Option<Self> {
        Some(Self {
            file: File::parse(&raw[0..1])?,
            rank: Rank::parse(&raw[1..2])?,
        })
    }
}

impl Sub<Self> for Coordinate {
    type Output = Offset;

    fn sub(self, rhs: Self) -> Self::Output {
        Offset {
            horizontal: self.file as i8 - rhs.file as i8,
            vertical: self.rank as i8 - rhs.rank as i8,
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Side {
    King,
    Queen,
}

impl Side {
    pub fn king_safespot_file(self) -> File {
        match self {
            Self::King => File::G,
            Self::Queen => File::C,
        }
    }
    
    pub fn rook_home_file(self) -> File {
        match self {
            Self::King => File::H,
            Self::Queen => File::A, 
        }
    }
    
    pub fn rook_castled_file(self) -> File {
        match self {
            Self::King => File::F,
            Self::Queen => File::D,
        }
    }
}

impl Display for Side {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::King => write!(f, "o-o"),
            Self::Queen => write!(f, "o-o-o"),
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Move {
    Simple {
        from: Coordinate,
        to: Coordinate,
    },
    Promotion {
        from: File,
        to: File,
        piece: PieceKind,
    },
    EnPassant {
        from: File,
        to: File,
    },
    Castling {
        side: Side,
    }
}

impl Move {
    #[inline]
    pub fn resolve_from(self, color: Color) -> Coordinate {
        match self {
            Self::Simple { from, .. } => from,
            Self::Promotion { from, .. } => Coordinate { file: from, rank: color.prepromotion_rank() },
            Self::EnPassant { from, .. } => Coordinate { file: from, rank: color.en_passant_rank() },
            Self::Castling { .. } => Coordinate { file: File::E, rank: color.home_rank() },
        }
    }

    #[inline]
    pub fn resolve_to(self, color: Color) -> Coordinate {
        match self {
            Self::Simple { to, .. } => to,
            Self::Promotion { to, .. } => Coordinate { file: to, rank: color.promotion_rank() },
            Self::EnPassant { to, .. } => Coordinate { file: to, rank: color.unpassable_rank() },
            Self::Castling { side } => Coordinate { file: side.king_safespot_file(), rank: color.home_rank() },
        }
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Move::Simple { from, to } => write!(f, "{from}{to}"),
            Move::Promotion { from, to, piece } => write!(f, "={from}{to}{piece}"),
            Move::EnPassant { from, to } => write!(f, "~{from}{to}"),
            Move::Castling { side } => write!(f, "c{side}"),
        }
    }
}

pub struct Iter {
    next: Option<Coordinate>,
    back_next: Option<Coordinate>,
}

impl Iter {
    pub fn new() -> Self {
        Self {
            next: Some(Coordinate { file: File::A, rank: Rank::First }),
            back_next: Some(Coordinate { file: File::A, rank: Rank::Eighth }),
        }
    }
}

impl Iterator for Iter {
    type Item = Coordinate;

    fn next(&mut self) -> Option<Self::Item> {
        let curr_cord = self.next?;

        self.next = curr_cord.next();

        Some(curr_cord)
    }
}

impl DoubleEndedIterator for Iter {
    fn next_back(&mut self) -> Option<Self::Item> {
        let curr_cord = self.back_next?;

        self.back_next = curr_cord.back_next();

        Some(curr_cord)
    }
}
