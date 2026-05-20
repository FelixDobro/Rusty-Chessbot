
use crate::chess::Game;
use crate::chess::chess_move::Move;
use crate::search::SearchAlgorithm;
use crate::evaluation::BoardEvaluator;
use crate::move_sorting::MoveSortingAlgorithm;


use std::{
    error::Error,
    io::{Read, Stdin, Stdout, Write, stdin, stdout},
    str::FromStr,
};



pub struct UCIManager<Search, Eval, Sort>
where 
Search: SearchAlgorithm,
Eval: BoardEvaluator,
Sort: MoveSortingAlgorithm,
{   
    search: Search,
    eval: Eval,
    sort: Sort,
    game: Game,
    std_in: Stdin,
}

impl<Search, Eval, Sort> UCIManager<Search, Eval, Sort>
where 
Search: SearchAlgorithm,
Eval: BoardEvaluator,
Sort: MoveSortingAlgorithm,
{
    pub fn new(search: Search, eval: Eval, sort: Sort) -> UCIManager<Search, Eval, Sort>
    where 
    Search: SearchAlgorithm,
    Eval: BoardEvaluator,
    Sort: MoveSortingAlgorithm, 
    {
        UCIManager {
            search: search,
            eval: eval,
            sort: sort,
            game: Game::default(),
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
                        self.game = Game::default();
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
                                if let Ok(value) = Game::from_fen(&fen) {
                                    self.game = value;
                                }
                                let move_token = tokens.next();
                                match move_token {
                                    Some("moves") => {
                                        while let Some(token) = tokens.next() {
                                            let m = Move::from_string(token, &self.game)?;
                                          
                                            self.game.make_pl_move_copy(m);
                                            
                                        }
                                    }
                                    _ => {}
                                };
                            }

                            Some("startpos") => {
                                self.game = Game::default();
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
                                    search_depth = next_token.as_deref().and_then(|i| i.parse::<u8>().ok()).unwrap();
                            },
                            _ => {},
                        };

                        if let Some(res) = self
                            .search
                            .search::<Eval,Sort>(&mut self.game, search_depth)
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
