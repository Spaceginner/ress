use std::sync::atomic::{AtomicI32, Ordering};
use rayon::prelude::*;
use engine::Engine;
use ress::{Board, DrawReason, GameOutcome};
use ress::piece::Color;


fn battle(white: &Engine, black: &Engine) -> (i32, i32) {
    [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",  // starting
        "rnbq1bnr/ppppkppp/8/4p3/4P3/8/PPPPKPPP/RNBQ1BNR w - - 2 3", // double bongcloud
        "rnbqk2r/pppp1ppp/5n2/2b1p3/2B1P3/2N5/PPPP1PPP/R1BQK1NR w KQkq - 4 4", // vienna
        "rnbqkb1r/ppp2ppp/3p4/8/3Pn3/5N2/PPP2PPP/RNBQKB1R b KQkq - 0 5",  // petrov's
        "rnbqkb1r/pp3p1p/3p1np1/2pP4/4PP2/2N5/PP4PP/R1BQKBNR b KQkq f3 0 7", // "The Flick-Knife Attack"
        "r1bqkb1r/pppp1ppp/2n2n2/4p3/4P3/2N2N2/PPPP1PPP/R1BQKB1R w KQkq - 4 4",  // four knights
        "rnb1kbnr/ppp1pppp/8/q7/8/2N5/PPPP1PPP/R1BQKBNR w KQkq - 2 4",  // scandi
        "rn1qkbnr/pp2pppp/2p5/3pPb2/3P4/8/PPP2PPP/RNBQKBNR w KQkq - 1 4",  // caro-kann advanced
    ].into_par_iter().map(|pos| {
        let mut score = (0, 0);
        let mut board = Board::from_fen(pos).unwrap();

        while board.game_outcome.is_none() {
            if board.draw_pending.is_some() {
                board.decline_draw();
            };

            let engine = match board.move_color {
                Color::White => white,
                Color::Black => black,
            };

            let r#move = engine.choose_move(&board, board.move_color).0;
            let _ = board.play_move(r#move);
        };

        let plies_count_score = (board.grid_history.len()/4) as i32;
        score.0 += plies_count_score;
        score.1 += plies_count_score;

        match board.game_outcome.unwrap() {
            GameOutcome::Decisive { won, .. } => {
                match won {
                    Color::White => score.0 += 500,
                    Color::Black => score.1 += 500,
                };
            },
            GameOutcome::Draw(DrawReason::InsufficientMaterial | DrawReason::Stalemate) => {
                score.0 += 350;
                score.1 += 350;
            },
            _ => {}
        };
        score
    }).reduce(|| (0, 0), |r#final, battle| (r#final.0 + battle.0, r#final.1 + battle.1))
}


const POOL_SIZE: usize = 100;


fn main() {
    let mut engine = Engine::load("engine.rew").unwrap_or_else(Engine::new_random);
    let mut epoch_i = 0;
    loop {
        engine.save(&format!("engine_epoch{epoch_i}.rew"));
        println!("epoch {epoch_i}");
        let mut pool = Vec::new();
        println!("generating pool");
        for _ in 1..POOL_SIZE {
            let mut engine = engine.clone();
            engine.mutate();
            pool.push((engine, 0));
        };
        pool.push((engine, 0));

        let score_change = [const { AtomicI32::new(0) }; POOL_SIZE];
        for (i, (engine_a, _)) in pool.iter().enumerate() {
            println!("battling engine {i}");
            pool.par_iter().enumerate()
                .filter(|(j, _)| i != *j)
                .map(|(j, (engine_b, _))| (j, battle(engine_a, engine_b)))
                .for_each(|(j, (a, b))| { score_change[i].fetch_add(a, Ordering::Relaxed); score_change[j].fetch_add(b, Ordering::Relaxed); });
        };
        dbg!(&score_change);
        
        for (i, change) in score_change.into_iter().enumerate() {
            pool[i].1 += change.into_inner();
        };
        
        engine = pool.into_iter().max_by_key(|e| e.1).unwrap().0;
        epoch_i += 1;
    };
}
