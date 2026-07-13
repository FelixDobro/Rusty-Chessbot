use crate::search::SearchLimits;

use crate::chess::board::Board;
use crate::chess::chess_move::Move;
use crate::search::ids::IDSearch;

use std::{
    error::Error,
    io::{Stdin, stdin},
};

pub struct UCIManager {
    search: IDSearch,
    board: Board,
    std_in: Stdin,
}

impl UCIManager {
    pub fn new(search: IDSearch) -> UCIManager {
        UCIManager {
            search: search,
            board: Board::default(),
            std_in: stdin(),
        }
    }

    pub fn start_protocol(&mut self) -> Result<(), Box<dyn Error>> {
        let mut buf_in = String::new();

        let mut running = true;

        while running {
            buf_in.clear();

            let bytes_read = self.std_in.read_line(&mut buf_in)?;

            if bytes_read == 0 {
                break;
            }

            let mut tokens = buf_in.split_whitespace();

            if let Some(token) = tokens.next() {
                match token {
                    "quit" => running = false,

                    "uci" => {
                        println!("uciok");
                    }

                    "ucinewboard" => {
                        self.board = Board::default();
                    }

                    "isready" => {
                        println!("readyok");
                    }

                    "position" => {
                        while let Some(next_token) = tokens.next() {
                            match next_token {
                                "fen" => {
                                    let fen_parts: Vec<&str> = tokens.by_ref().take(6).collect();
                                    let fen = fen_parts.join(" ");
                                    if let Ok(value) = Board::from_fen(&fen) {
                                        self.board = value;
                                    }
                                }

                                "moves" => {
                                    while let Some(token) = tokens.next() {
                                        let m = Move::from_string(token, &self.board)?;

                                        self.board.make_pl_move::<true>(m);
                                    }
                                }

                                "startpos" => {
                                    self.board = Board::default();
                                }
                                _ => {}
                            };
                        }
                    }
                    "go" => {
                        let mut search_depth = None;
                        let mut wtime = None;
                        let mut btime = None;
                        let mut winc = None;
                        let mut binc = None;

                        while let Some(param) = tokens.next() {
                            match param {
                                "depth" => {
                                    if let Some(val) = tokens.next() {
                                        search_depth = val.parse::<u8>().ok();
                                    }
                                }
                                "wtime" => {
                                    if let Some(val) = tokens.next() {
                                        wtime = val.parse::<u64>().ok();
                                    }
                                }
                                "btime" => {
                                    if let Some(val) = tokens.next() {
                                        btime = val.parse::<u64>().ok();
                                    }
                                }
                                "winc" => {
                                    if let Some(val) = tokens.next() {
                                        winc = val.parse::<u64>().ok();
                                    }
                                }
                                "binc" => {
                                    if let Some(val) = tokens.next() {
                                        binc = val.parse::<u64>().ok();
                                    }
                                }
                                _ => {}
                            }
                        }
                        let limit = if let Some(d) = search_depth {
                            SearchLimits::depth(d)
                        } else {
                            let remaining_time_ms = if self.board.get_turn().is_white() {
                                wtime.unwrap_or(5000)
                            } else {
                                btime.unwrap_or(5000)
                            };

                            let increment = if self.board.get_turn().is_white() {
                                winc.unwrap_or(5000)
                            } else {
                                binc.unwrap_or(5000)
                            };

                            SearchLimits {
                                max_depth: None,
                                base_inc: Some((remaining_time_ms, increment)),
                                max_nodes: None,
                                infinite: false,
                            }
                        };

                        if let Some(res) = self.search.search(&mut self.board, &limit) {
                            res.print_info();
                            println!("bestmove {}", res.best_move.to_string());
                        } else {
                            println!(
                                "Warning: Search did not find any move, returning any legal move!"
                            );
                            let pseudos = self.board.generate_pseudolegals();
                            for m in pseudos.as_slice() {
                                if self.board.make_pl_move::<true>(*m) {
                                    println!("bestmove {}", m.to_string());
                                    self.board.unmake_pl_move(*m);
                                }
                            }
                            println!("best")
                        }
                    }
                    _ => {}
                };
            }
        }

        Ok(())
    }
}
