use crate::chess::{Board, game::Game};
pub mod static_evaluation;




pub trait BoardEvaluator {
    fn evaluate(board: &Board) -> f32;
}