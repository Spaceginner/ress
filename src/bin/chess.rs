use std::io::{BufRead, StdinLock, Write};
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
    let mut board = Board::default();
    let mut stdin = std::io::stdin().lock();

    let mut board_changed = true;
    'gameloop: loop {
        for color in [Color::White, Color::Black] {
            if board_changed {
                let plies_count = board.grid_history.len();
                println!("\nmove #{} (ply #{plies_count}), {color}'s turn:\n{board}", plies_count.div_ceil(2));
                board_changed = false;
            };

            println!("\n{color}:");
            loop {
                let command = prompt(&mut stdin);

                if command.len() <= 1 {
                    if command.is_empty() {
                        println!();
                    };
                    println!("to exit enter /exit.");
                    continue;
                };

                match &command.as_str()[..command.len()-1] {
                    "/draw" => { board.propose_draw(color); println!("{color} has proposed a draw."); break; },
                    "/decline" => { board.decline_draw(); println!("the draw has been declined."); break; },
                    "/resign" => { board.resign(color); break; },
                    "/help" => { println!("you can /help, /exit, /draw, /decline, /resign, /moves or enter a move."); },
                    "/exit" => { break 'gameloop; },
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
                            }
                        }
                    }
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

                break 'gameloop;
            };
        };
    };

    println!("goodbye.")
}
