pub mod simple_search;
use crate::chess::game::Game;

use crate::move_sorting::MoveSortingAlgorithm;
use crate::{chess::chessMove::Move, evaluation::BoardEvaluator};

#[derive(Debug, PartialEq)]
pub struct SearchResult {
    pub best_move: Move,
    pub evaluation: i32,
}

pub trait SearchAlgorithm {
    fn search<Eval: BoardEvaluator, Sort: MoveSortingAlgorithm>(&mut self, game: &mut Game, depth: u8) -> Option<SearchResult>;
}