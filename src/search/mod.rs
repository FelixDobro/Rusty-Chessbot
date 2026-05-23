pub mod simple_search;
use crate::chess::board::Board;


use crate::{chess::chess_move::Move};

#[derive(Debug, PartialEq)]
pub struct SearchResult {
    pub best_move: Move,
    pub evaluation: i32,
}

pub trait SearchAlgorithm {
    fn search(&mut self, board: &mut Board, depth: u8) -> Option<SearchResult>;
}