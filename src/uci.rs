
use crate::chess::board::Board;
use crate::chess::chess_move::Move;
use crate::search::SearchAlgorithm;

use std::{
    error::Error,
    io::{Stdin,stdin},
};



pub struct UCIManager<>
{   
    search: Box<dyn SearchAlgorithm>,
    board: Board,
    std_in: Stdin,
}

impl UCIManager
{
    pub fn new(search: Box<dyn SearchAlgorithm>) -> UCIManager
    {
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
                        let next_token = tokens.next();
                        match next_token {
                            Some("fen") => {
                                let fen_parts: Vec<&str> = tokens.by_ref().take(6).collect();
                                let fen = fen_parts.join(" ");
                                if let Ok(value) = Board::from_fen(&fen) {
                                    self.board = value;
                                }
                                let move_token = tokens.next();
                                match move_token {
                                    Some("moves") => {
                                        while let Some(token) = tokens.next() {
                                            let m = Move::from_string(token, &self.board)?;
                                          
                                            self.board.make_pl_move::<true>(m);
                                            
                                        }
                                    }
                                    _ => {}
                                };
                            }

                            Some("startpos") => {
                                self.board = Board::default();
                            }
                            _ => {}
                        };
                    }

                    "go" => {
                        let depth_token = tokens.next();
                        let mut search_depth = 6;
                        match depth_token {
                            Some("depth") => {
                                let next_token = tokens.next();
                                search_depth = next_token.as_deref().and_then(|i| i.parse::<u8>().ok()).unwrap();
                            },
                            _ => {},
                        };

                        if let Some(res) = self
                            .search
                            .search(&mut self.board, search_depth)
                        {
                            println!("bestmove {}", res.best_move.to_string());
                        }
                    }
                    _ => {}
                };
            }
        }

        Ok(())
    }
}
