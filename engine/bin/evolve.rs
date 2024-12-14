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

        let plies_count_score = board.grid_history.len() as i32;
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


fn find_best(pool: Vec<Engine>) -> Engine {
    let score_atom = Vec::from_iter((0..pool.len()).map(|_| AtomicI32::new(0)));
    for (i, engine_a) in pool.iter().enumerate() {
        pool.par_iter().enumerate()
            .filter(|(j, _)| i != *j)
            .map(|(j, engine_b)| (j, battle(engine_a, engine_b)))
            .for_each(|(j, (a, b))| {
                score_atom[i].fetch_add(a, Ordering::Relaxed);
                score_atom[j].fetch_add(b, Ordering::Relaxed);
            });
    };
    
    let score = score_atom.into_iter().map(|s| s.into_inner()).collect::<Vec<_>>();
    pool.into_iter().enumerate().max_by_key(|(i, _)| score[*i]).unwrap().1
}


fn create_pool(engine: &Engine, mutation_coef: Option<f32>, size: usize) -> Vec<Engine> {
    (0..size).map(|_| {
        let mut engine = engine.clone();
        engine.mutate(mutation_coef);
        engine
    }).collect()
}


fn create_pools(engine: &Engine, mutation_coef: Option<f32>, size: usize, count: usize) -> Vec<Vec<Engine>> {
    (0..count).into_par_iter().map(|_| create_pool(engine, mutation_coef, size)).collect()
}


const POOL_SIZE: usize = 15;
const POOLS_COUNT: usize = 20;
const HYPER_POOL_SIZE: usize = 10;


fn main() {
    let random;
    let mut engine;
    if let Some(eng) = Engine::load("engine.rew") {
        engine = eng;
        random = false;
    } else {
        engine = Engine::new_random();
        random = true;
    };
    
    let mut epoch_i = 0;
    loop {
        engine.save(&format!("engine_epoch{epoch_i}.rew"));
        epoch_i += 1;
        println!("epoch {epoch_i}");
        
        let hyper_pool = (0..HYPER_POOL_SIZE).into_par_iter().map(|i| {
            println!("generating pools (#{i})...");
            let pools = create_pools(&engine, (epoch_i != 1 && !random).then_some(0.8), POOL_SIZE, POOLS_COUNT);
            
            println!("battling pools (#{i})...");
            let super_pool = pools.into_par_iter().map(find_best).collect::<Vec<_>>();
            
            println!("battling super pool (#{i})...");
            find_best(super_pool)
        }).collect::<Vec<_>>();
        
        println!("battling hyper pool...");
        engine = find_best(hyper_pool);
    };
}
