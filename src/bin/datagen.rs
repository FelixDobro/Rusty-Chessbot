use chess_bot::{
    chess::{
        self,
        board::{Board, GameResult},
    },
    search::{self, Restrictions, SearchControl, SearchResult, simple_search::Search},
    uci::{UCIManager, UciCommand::UcinewGame},
};
use rayon::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fs::{self, File},
    io::{BufWriter, Write},
    sync::{Arc, Mutex, mpsc},
    thread,
};

pub struct Position {
    pub fen: String,
    eval: i16,
    res: GameResult,
    depth: usize,
    nodes: u64,
}

impl Position {
    pub fn from_result(res: &SearchResult, fen: String) -> Self {
        Self {
            fen: fen,
            eval: res.evaluation,
            res: GameResult::Draw,
            depth: res.depth as usize,
            nodes: res.nodes_searched,
        }
    }

    pub fn to_string(&self) -> String {
        format!(
            "{},{},{},{},{}",
            self.fen,
            self.eval,
            self.res.to_string(),
            self.depth,
            self.nodes
        )
    }
}

pub fn play_game(fen: String) -> Vec<Position> {
    let mut board = Board::from_fen(&fen).unwrap();
    let mut s = Search::new(100, SearchControl::new());
    let mut positions = Vec::new();
    while let None = board.game_result() {
        if let Some(res) = s.ids::<{ Restrictions::NODES }>(&mut board, Restrictions::nodes(50000))
        {
            board.make_pl_move::<true>(res.best_move);
            positions.push(Position::from_result(&res, board.to_fen()));
        } else {
            panic!("Search did not find any move but game continued");
        }
    }
    if let Some(res) = board.game_result() {
        positions.iter_mut().for_each(|pos| pos.res = res);
    } else {
        panic!("No Game Result at the end of game");
    }
    positions
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let (tx, rx) = mpsc::channel::<Vec<Position>>();

    let writer = thread::spawn(move || {
        let mut positions: HashSet<String> = HashSet::new();

        let file =
            File::create("/home/felix/projects/chess/Rusty-Chessbot/data/first.csv").unwrap();

        let mut f_writer = BufWriter::new(file);
        writeln!(f_writer, "fen,eval,res,depth,nodes").unwrap();

        while let Ok(results) = rx.recv() {
            results.iter().for_each(|pos| {
                if positions.insert(pos.fen.clone()) {
                    writeln!(f_writer, "{}", pos.to_string()).unwrap();
                }
            });
        }
    });

    let p = "/home/felix/projects/chess/Rusty-Chessbot/data/UHO_2024_8mvs_big_+080_+099.epd";
    println!("Loading Data");
    let openings = fs::read_to_string(p)?;
    println!("Data loaded");
    openings
        .par_lines()
        .for_each_with(tx, |tx, fen| tx.send(play_game(fen.to_string())).unwrap());
    writer.join().unwrap();
    Ok(())
}
