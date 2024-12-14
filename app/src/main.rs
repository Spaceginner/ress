use std::io::{BufRead, StdinLock, Write};
use engine::Engine;
use ress::{Board, GameOutcome, MoveError, PlayerMove};
use ress::piece::Color;

fn prompt(stdin: &mut StdinLock) -> String {
    print!(">>> ");
    std::io::stdout().flush().unwrap();
    let mut buf = String::new();
    stdin.read_line(&mut buf).unwrap();
    buf
}

fn main() {
    let mut stdin = std::io::stdin().lock();

    let mut engine = None;
    let mut engine_white = false;
    let mut engine_black = false;
    println!("to start a new game enter /start or enter /help for more commands.");
    'menu: loop {
        println!("menu:");
        let command = prompt(&mut stdin);

        if command.len() <= 1 {
            if command.is_empty() {
                println!();
            };
            println!("to exit enter /exit.");
            continue;
        };
        
        match &command.as_str()[..command.len()-1] {
            "/enginew" => {
                if engine.is_none() {
                    engine = Some(Engine::load("engine.rew").unwrap());
                };
                
                engine_white ^= true;
                println!("switching engine playing white (now {engine_white}).");
            },
            "/engineb" => {
                if engine.is_none() {
                    engine = Some(Engine::load("engine.rew").unwrap());
                };
                
                engine_black ^= true;
                println!("switching engine playing black (now {engine_black})");
            },
            "/help" => { println!("you can /start, /exit, /enginew or /engineb.") },
            "/start" => {
                println!("starting game...");
                // let mut board = Board::from_fen("rnb2bnr/ppp1pppp/5k2/3K4/6Q1/2N5/PPPPPPPP/R1B2BNR b HAha - 0 1").unwrap();
                let mut board = Board::default();
                let mut board_changed = true;
                'game: loop {
                    for color in [board.move_color, board.move_color.the_other()] {
                        if board_changed {
                            println!("{board}");
                            board_changed = false;
                        };

                        println!("\n{color}:");

                        if (engine_white && color == Color::White) || (engine_black && color == Color::Black) {
                            if board.draw_pending.is_some() {
                                println!("e>> /decline");
                                board.decline_draw();
                            } else {
                                let r#move = engine.as_ref().unwrap().choose_move(&board, color);
                                println!("e>> {} (c{:.0}%)", r#move.0, r#move.1*100.0);
                                board.play_move(r#move.0).unwrap();
                                board_changed = true;
                            };
                        } else {
                            loop {
                                let command = prompt(&mut stdin);

                                if command.len() <= 1 {
                                    if command.is_empty() {
                                        println!();
                                    };
                                    println!("to abort enter /abort or to exit enter /exit.");
                                    continue;
                                };

                                match &command.as_str()[..command.len()-1] {
                                    "/draw" => { board.propose_draw(color); println!("{color} has proposed a draw."); break; },
                                    "/decline" => { board.decline_draw(); println!("the draw has been declined."); break; },
                                    "/resign" => { board.resign(color); break; },
                                    "/help" => { println!("you can /help, /abort, /exit, /draw, /decline, /resign, /moves or enter a move."); },
                                    "/exit" => { break 'menu; },
                                    "/abort" => { break 'game; },
                                    "/moves" => {
                                        println!("possible moves are:");
                                        for (i, r#move) in board.possible_moves(board.move_color).iter().enumerate() {
                                            print!("{move} ");
                                            if (i+1) % 6 == 0 {
                                                println!();
                                            };
                                        };
                                        println!();
                                    },
                                    _ if &command[0..1] == "/" => { println!("unknown command. enter /help for help.") }
                                    raw_move => {
                                        if board.draw_pending.is_some() {
                                            println!("there is a draw pending. accept or decline it.");
                                            continue;
                                        };

                                        let r#move = PlayerMove::parse(raw_move);

                                        match r#move {
                                            None => println!("move is invalid, you can enter either long algebraic or internal notation."),
                                            Some(r#move) => {
                                                if let Err(move_err) = board.play_move(r#move) {
                                                    match move_err {
                                                        MoveError::IllegalMove => { println!("the move you have entered is illegal."); },
                                                        MoveError::AmbiguousMove => { println!("the move you have entered is ambiguous."); },
                                                        _ => unreachable!(),
                                                    };

                                                    continue;
                                                };

                                                board_changed = true;
                                                break;
                                            },
                                        };
                                    },
                                };
                            };
                        };

                        if let Some(outcome) = board.game_outcome {
                            if board_changed {
                                let plies_count = board.grid_history.len();
                                println!("\nmove #{} (ply #{plies_count}), {color}'s turn:\n{board}", plies_count.div_ceil(2));
                            };

                            println!();
                            match outcome {
                                GameOutcome::Decisive { won, reason } => {
                                    println!("the game is over. {won} has won, because of a {reason}.")
                                },
                                GameOutcome::Draw(reason) => {
                                    println!("the game is over. it is a draw, because of a(n) {reason}.")
                                },
                            };

                            break 'game;
                        };
                    };
                };
            },
            "/exit" => { break; },
            _ if &command[0..1] == "/" => { println!("unknown command. enter /help for help.") }
            _ => { println!("the game hasn't been started yet! type /start to start.") }
        }
    }

    println!("goodbye.")
}
