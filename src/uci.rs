use crate::parameters::{ENGINE_SETTINGS, print_engine_parameters};
use crate::search::{Restrictions, SearchControl, SearchLimits, SearchResult};

use crate::chess::board::Board;
use crate::chess::chess_move::{Move, NULL_MOVE};

#[cfg(feature = "tuning")]
use crate::parameters::set_tuning_option;
use crate::search::simple_search::Search;
use crate::uci::UciCommand::{Go, IsReady, Position, SetOption, UcinewGame};

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::SystemTime;
use std::{
    error::Error,
    io::{Stdin, stdin},
    str::SplitWhitespace,
    sync::mpsc,
    thread,
};

pub enum UciCommand {
    Go(String),
    UcinewGame,
    SetOption(String),
    Position(String),
    IsReady,
}

pub struct UCIManager {
    std_in: Stdin,
}

impl UCIManager {
    pub fn new() -> UCIManager {
        UCIManager { std_in: stdin() }
    }

    pub fn handle_uci(&self) {
        print_engine_parameters();
        println!("uciok")
    }

    pub fn handle_set_option(input: String, search: &mut Search) {
        let mut tokens = input.split_whitespace();
        if tokens.next() != Some("setoption") {
            return;
        }
        if let Some(n) = tokens.next() {
            if n != "name" {
                return;
            }
            if let Some(name) = tokens.next() {
                if let Some(val_token) = tokens.next() {
                    if val_token == "value" {
                        if let Some(value) = tokens.next() {
                            Self::set_option(name, value, search);
                        }
                    }
                }
            }
        }
    }

    pub fn handle_pos(input: String, board: &mut Board) {
        let mut tokens = input.split_whitespace();
        if tokens.next() != Some("position") {
            return;
        }
        while let Some(next_token) = tokens.next() {
            match next_token {
                "fen" => {
                    let fen_parts: Vec<&str> = tokens.by_ref().take(6).collect();
                    let fen = fen_parts.join(" ");
                    if let Ok(value) = Board::from_fen(&fen) {
                        *board = value;
                    }
                }

                "moves" => {
                    while let Some(token) = tokens.next() {
                        let m = Move::from_string(token, board).unwrap();
                        board.make_pl_move::<true>(m);
                    }
                }

                "startpos" => {
                    *board = Board::default();
                }
                _ => {}
            };
        }
    }

    pub fn parse_limits(input: String, board: &Board) -> SearchLimits {
        let mut tokens = input.split_whitespace();
        tokens.next();

        let mut depth = None;
        let mut wtime = None;
        let mut btime = None;
        let mut winc = None;
        let mut binc = None;
        let mut nodes = None;
        let mut ponder = false;

        while let Some(param) = tokens.next() {
            match param {
                "depth" => {
                    if let Some(val) = tokens.next() {
                        depth = val.parse::<u8>().ok();
                    }
                }
                "ponder" => ponder = true,
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
        let mut base_inc = None;
        if board.get_turn().is_white() {
            if wtime.is_some() {
                base_inc = Some((wtime.unwrap(), winc.unwrap_or(0)));
            }
        } else {
            if btime.is_some() {
                base_inc = Some((btime.unwrap(), binc.unwrap_or(0)));
            }
        }
        SearchLimits {
            max_depth: depth,
            base_inc: base_inc,
            max_nodes: nodes,
            infinite: ponder,
        }
    }

    pub fn search(input: String, board: &mut Board, search: &mut Search) -> Option<SearchResult> {
        let limits = Self::parse_limits(input, board);
        if limits.infinite {
            search.start_ponder()
        }
        if let Some(d) = limits.max_depth {
            return search.ids::<{ Restrictions::DEPTH }>(board, Restrictions::depth(d));
        } else if let Some(nodes) = limits.max_nodes {
            return search.ids::<{ Restrictions::NODES }>(board, Restrictions::nodes(nodes));
        } else {
            let (b, inc) = limits.base_inc.unwrap_or((100, 0));
            let (soft, hard) = Self::calc_time(b, inc);
            return search.ids::<{ Restrictions::TIME }>(board, Restrictions::time(soft, hard));
        };
    }

    pub fn calc_time(base: u64, inc: u64) -> (u128, u128) {
        let soft_bound = base / 20 + inc / 2;
        let hard_bound = (0.3 * base as f32 - 300.0)
            .min(1.5 * soft_bound as f32)
            .max(50.0);
        (soft_bound as u128, hard_bound as u128)
    }

    pub fn handle_go(
        board: &mut Board,
        search: &mut Search,
        input: String,
    ) -> Option<SearchResult> {
        search.allow_search();

        if let Some(res) = Self::search(input, board, search) {
            res.print_info();
            print!("bestmove {}", res.best_move.to_string());
            if let Some(p) = search.ponder_prediction(res.best_move, board) {
                print!(" ponder {}", p.to_string());
            }
            println!();

            return Some(res);
        } else {
            println!("Warning: Search did not find any move, returning any legal move!");
            let pseudos = board.generate_pseudolegals();
            for m in pseudos.as_slice() {
                if board.make_pl_move::<true>(*m) {
                    println!("bestmove {}", m.to_string());
                    board.unmake_pl_move(*m);
                    break;
                }
            }
            None
        }
    }

    pub fn set_option(name: &str, val: &str, search: &mut Search) {
        if let Some(option) = ENGINE_SETTINGS.get(name) {
            option.validate(val).map_err(|e| println!("{}", e)).ok();
            match name {
                "Hash" => {
                    let mb = val.parse::<usize>().unwrap();
                    search.change_hash_size(mb);
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
        let (tx, rx) = mpsc::channel::<UciCommand>();

        let mut main_control = SearchControl::new();
        let mut search = Search::new(100, main_control.clone());
        let mut board = Board::default();

        thread::spawn(move || {
            loop {
                match rx.recv().unwrap() {
                    Go(s) => {
                        Self::handle_go(&mut board, &mut search, s);
                    }
                    UcinewGame => {
                        board = Board::default();
                        search.reset();
                    }
                    SetOption(s) => Self::handle_set_option(s, &mut search),
                    Position(s) => Self::handle_pos(s, &mut board),
                    IsReady => println!("readyok"),
                }
            }
        });

        let mut running = true;

        while running {
            let mut buf_in = String::new();
            let bytes_read = self.std_in.read_line(&mut buf_in)?;
            if bytes_read == 0 {
                break;
            }

            let mut tokens: SplitWhitespace<'_> = buf_in.split_whitespace();

            if let Some(token) = tokens.next() {
                match token {
                    "quit" => running = false,

                    "uci" => self.handle_uci(),

                    "setoption" => {
                        main_control.stop_search();
                        tx.send(SetOption(buf_in))?
                    }

                    "ucinewgame" => {
                        main_control.stop_search();
                        tx.send(UcinewGame)?;
                    }

                    "isready" => {
                        main_control.stop_search();
                        tx.send(IsReady)?;
                    }

                    "position" => {
                        main_control.stop_search();
                        tx.send(Position(buf_in))?;
                    }
                    "go" => {
                        main_control.stop_search();
                        tx.send(Go(buf_in)).unwrap()
                    }
                    "ponderhit" => {
                        main_control.stop_pondering();
                    }
                    "stop" => main_control.stop_search(),
                    _ => {}
                };
            }
        }

        Ok(())
    }
}
