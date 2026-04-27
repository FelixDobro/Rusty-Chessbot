use chess::{Board, ChessMove, Game};
use std::{
    error::Error,
    io::{Read, Stdin, Stdout, Write, stdin, stdout},
    str::FromStr,
};

use crate::{search::SearchAlgorithm, utils::display_board};

pub struct UCIManager {
    game: Game,
    search_algorithm: Box<dyn SearchAlgorithm>,
    std_in: Stdin,
}

impl UCIManager {
    pub fn new(search_algorithm: Box<dyn SearchAlgorithm>) -> UCIManager {
        UCIManager {
            game: Game::new(),
            search_algorithm: search_algorithm,
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

                    "ucinewgame" => {
                        self.game = Game::new();
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

                                if let Ok(value) = Game::from_str(&fen) {
                                    self.game = value;
                                }
                                let move_token = tokens.next();
                                match move_token {
                                    Some("moves") => {
                                        while let Some(token) = tokens.next() {
                                            if let Ok(m) = ChessMove::from_str(token) {
                                                self.game.make_move(m);
                                            }
                                        }
                                    }
                                    _ => {}
                                };
                            }

                            Some("startpos") => {
                                self.game = Game::new();
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
                                let maybe_int =
                                    next_token.as_deref().and_then(|i| i.parse::<i16>().ok());
                                search_depth = match maybe_int {
                                    Some(n) => n,
                                    None => 6,
                                };
                                println!("{}", search_depth);
                            },
                            _ => {},
                        };

                        if let Some(m) = self
                            .search_algorithm
                            .search(&self.game.current_position(), search_depth)
                        {
                            println!("bestmove {}", m.0.to_string());
                        }
                    }
                    _ => {}
                };
            }
        }

        Ok(())
    }
}
