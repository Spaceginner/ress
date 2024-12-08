#![feature(iter_array_chunks)]

use std::io::{Read, Write};
use rand::Rng;
use ress::{Board, PlayerMove};
use ress::coordinate::{Coordinate, File, Rank};
use ress::piece::{Color, PieceKind};

#[derive(Clone)]
pub struct Engine {
    // input 69 -> 2×120 -> 60 -> 4×30 -> output 129
    weights: (Box<[f32; 38250]>, Box<[f32; 420]>),
}

impl Engine {
    pub fn save(&self, to: &str) {
        std::fs::File::create(to).unwrap().write_all(&self.weights.0.iter().chain(self.weights.1.iter()).map(|w| w.to_le_bytes()).collect::<Vec<_>>().concat()).unwrap();
    }
    
    pub fn load(from: &str) -> Self {
        let mut file = std::fs::File::open(from).unwrap();
        let mut buf = vec![0; 38250*4+420*4];
        file.read_exact(&mut buf).unwrap();
        let data = buf.into_iter().array_chunks::<4>().map(f32::from_le_bytes).collect::<Vec<_>>();
        Self {
            weights: (
                Box::new(data[0..38250].try_into().unwrap()),
                Box::new(data[38250..38250+420].try_into().unwrap()),
            )
        }
    }
    
    pub fn new_random() -> Self {
        let mut rng = rand::thread_rng();

        let mut coefs = vec![0.0; 38250];
        let mut offsets = vec![0.0; 420];

        coefs.iter_mut().for_each(|c| *c = rng.gen::<f32>()*2.0-1.0);
        offsets.iter_mut().for_each(|o| *o = rng.gen::<f32>()*2.0-1.0);
        
        Self {
            weights: (coefs.into_boxed_slice().try_into().unwrap(), offsets.into_boxed_slice().try_into().unwrap())
        }
    }

    pub fn variate(&mut self, with: &Self) {
        todo!()
    }

    pub fn mutate(&mut self) {
        todo!()
    }

    fn piece_id(piece: PieceKind) -> f32 {
        match piece {
            PieceKind::Pawn => 1.0/12.0,     // 1/12
            PieceKind::Knight => 3.0/12.0,   // 3/12
            PieceKind::Bishop => 3.5/12.0,   // 3.5/12
            PieceKind::Rook => 5.0/12.0,    // 5/12
            PieceKind::Queen => 9.0/12.0,   // 9/12
            PieceKind::King => 1.0, // 12/12
        }
    }

    fn prepare_input(board: &Board) -> [f32; 69] {
        let mut buf = [0.0; 69];

        for (i, (piece, _)) in board.grid().iter_coord().enumerate() {
            buf[i] = match piece {
                None => 0.0,
                Some(piece) => {
                    let id = Self::piece_id(piece.kind);
                    match piece.color {
                        Color::White => id,
                        Color::Black => -id,
                    }
                }
            }
        }

        buf[64] = board.white_castle.0 as u8 as f32;
        buf[65] = board.white_castle.1 as u8 as f32;
        buf[66] = board.black_castle.0 as u8 as f32;
        buf[67] = board.black_castle.1 as u8 as f32;
        buf[68] = board.stale_plies as f32 / 50.0;

        buf
    }

    fn feed(weights: &[f32], offset: usize, state: &mut [f32], source: (usize, usize), layer: (usize, usize)) {
        for i in 0..layer.1 {
            for j in 0..source.1 {
                state[layer.0+i] += state[source.0+j] * weights[offset+i*source.1+j];
            };
            state[layer.0+i] = 2.0 / (1.0 + 9.0f32.powf(-state[layer.0+i])) - 1.0;
        };
    }

    pub fn choose_move(&self, board: &Board, by: Color) -> (PlayerMove, f32) {
        let legal_moves = board.possible_moves(by);

        if legal_moves.is_empty() {
            panic!();
        };

        if legal_moves.len() == 1 {
            return (PlayerMove::Internal(legal_moves[0]), 1.0);
        };

        let input = Self::prepare_input(board);
        let mut state = [0.0; 618];
        state[0..69].copy_from_slice(&input);
        state[69..489].copy_from_slice(&*self.weights.1);

        let mut of = 0;
        let mut source_start = 0;
        for dims in [69, 120, 120, 60, 30, 30, 30, 30, 129].windows(2) {
            let layer_start = source_start + dims[0];
            Self::feed(&*self.weights.0, of, &mut state, (source_start, dims[0]), (layer_start, dims[1]));
            of += dims[0]*dims[1];
            source_start = layer_start;
        };
        
        let mut best_move = (PlayerMove::Internal(legal_moves[0]), 0.0);
        for from_file in 0..8 {
            for from_rank in 0..8 {
                for to_file in 0..8 {
                    for to_rank in 0..8 {
                        let eval = state[from_rank*8+from_file].abs() * state[to_rank*8+to_file+64].abs();
                        if eval > best_move.1 {
                            let mut promote_to = (PieceKind::Queen, 1.0);
                            for piece in [PieceKind::Knight, PieceKind::Bishop, PieceKind::Rook, PieceKind::Queen] {
                                let dist = (Self::piece_id(piece) - state[617]).abs();
                                if dist < promote_to.1 {
                                    promote_to = (piece, dist);
                                };
                            };
                            
                            let chosen_move = (
                                Coordinate { file: File::try_from(from_file as i8).unwrap(), rank: Rank::try_from(from_rank as i8).unwrap() },
                                Coordinate { file: File::try_from(to_file as i8).unwrap(), rank: Rank::try_from(to_rank as i8).unwrap() },
                                Some(promote_to.0)
                            );
                            
                            if legal_moves.iter().any(|m| m.resolve_from(by) == chosen_move.0 && m.resolve_to(by) == chosen_move.1) {
                                best_move = (PlayerMove::Long { from: chosen_move.0, to: chosen_move.1, promotion: chosen_move.2 }, eval);
                            };
                        };
                    };
                };
            };
        };

        best_move
    }
}
