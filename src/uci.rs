use crate::parameters::{ENGINE_SETTINGS, print_engine_parameters};
use crate::search::SearchLimits;

use crate::chess::board::Board;
use crate::chess::chess_move::Move;
use crate::search::ids::IDSearch;

#[cfg(feature = "tuning")]
use crate::parameters::set_tuning_option;

use std::{
    error::Error,
    io::{Stdin, stdin},
    str::SplitWhitespace,
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

    pub fn handle_uci(&self) {
        print_engine_parameters();
        println!("uciok")
    }

    pub fn handle_set_option(&mut self, tokens: &mut SplitWhitespace) {
        if let Some(n) = tokens.next() {
            if n != "name" {
                return;
            }
            if let Some(name) = tokens.next() {
                if let Some(val_token) = tokens.next() {
                    if val_token == "value" {
                        if let Some(value) = tokens.next() {
                            self.set_option(name, value);
                        }
                    }
                }
            }
        }
    }

    pub fn set_option(&mut self, name: &str, val: &str) {
        if let Some(option) = ENGINE_SETTINGS.get(name) {
            option.validate(val).map_err(|e| println!("{}", e)).ok();
            match name {
                "Hash" => {
                    let mb = val.parse::<usize>().unwrap();
                    self.search.change_hash(mb);
                }
                _ => println!("Option {} not found", name),
            }
        } else {
            #[cfg(feature = "tuning")]
            {
                set_tuning_option(name, val)
                    .map_err(|e| println!("{}", e))
                    .ok();
                return;
            }
            println!("Option {} not found", name)
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

            let mut tokens: SplitWhitespace<'_> = buf_in.split_whitespace();

            if let Some(token) = tokens.next() {
                match token {
                    "quit" => running = false,

                    "uci" => self.handle_uci(),

                    "setoption" => self.handle_set_option(&mut tokens),

                    "ucinewgame" => {
                        self.board = Board::default();
                        self.search.reset();
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
                        let mut nodes = None;

                        while let Some(param) = tokens.next() {
                            match param {
                                "depth" => {
                                    if let Some(val) = tokens.next() {
                                        search_depth = val.parse::<u8>().ok();
                                    }
                                }
                                "nodes" => {
                                    if let Some(val) = tokens.next() {
                                        nodes = val.parse::<u64>().ok();
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
                        } else if let Some(nodes) = nodes {
                            SearchLimits::nodes(nodes)
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
                                    break;
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
