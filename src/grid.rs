use std::ops::{Index, IndexMut};
use crate::coordinate::{Coordinate, File, Move, Rank, Side};
use crate::piece::{Color, Piece, PieceKind};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Grid(pub [[Option<Piece>; 8]; 8]);

impl Index<Coordinate> for Grid {
    type Output = Option<Piece>;
    
    fn index(&self, index: Coordinate) -> &Self::Output {
        &self.0[index.rank as usize][index.file as usize]
    }
}

impl IndexMut<Coordinate> for Grid {
    fn index_mut(&mut self, index: Coordinate) -> &mut Self::Output {
        &mut self.0[index.rank as usize][index.file as usize]
    }
}

impl Grid {
    pub fn r#move(&mut self, r#move: Move, color: Color) -> bool {
        match r#move {
            Move::Simple { from, to } => {
                let advancing_move = matches!(self[from], Some(Piece { kind: PieceKind::Pawn, .. })) | self[to].is_some();
                self[to] = self[from].take();
                advancing_move
            },
            r#move @ Move::Promotion { piece, .. } => {
                self[r#move.resolve_to(color)] = Some(Piece { color, kind: piece });
                self[r#move.resolve_from(color)] = None;
                true
            },
            r#move @ Move::EnPassant { to, .. } => {
                self[r#move.resolve_to(color)] = self[r#move.resolve_from(color)].take();
                self[Coordinate { file: to, rank: color.en_passant_rank() }] = None;
                true
            },
            r#move @ Move::Castling { side } => {
                self[r#move.resolve_to(color)] = self[r#move.resolve_from(color)].take();
                {
                    let rank = color.home_rank();
                    self[Coordinate { file: side.rook_castled_file(), rank }] = self[Coordinate { file: side.rook_home_file(), rank }].take();
                };
                false
            },
        }
    }
    
    pub fn iter_coord(&self) -> Iter {
        Iter {
            grid: self,
            next: Some(Coordinate { file: File::A, rank: Rank::First }),
            back_next: Some(Coordinate { file: File::A, rank: Rank::Eighth }),
        }
    }
}

pub struct Iter<'a> {
    grid: &'a Grid,
    next: Option<Coordinate>,
    back_next: Option<Coordinate>,
}

impl Iterator for Iter<'_> {
    type Item = (Option<Piece>, Coordinate);
    
    fn next(&mut self) -> Option<Self::Item> {
        let curr_cord = self.next?;

        let piece = self.grid[curr_cord];
        
        self.next = curr_cord.next();
        
        Some((piece, curr_cord))
    }
}

impl DoubleEndedIterator for Iter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let curr_cord = self.back_next?;

        let piece = self.grid[curr_cord];

        self.back_next = curr_cord.back_next();

        Some((piece, curr_cord))
    }
}
