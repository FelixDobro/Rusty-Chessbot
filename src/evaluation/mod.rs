use crate::chess::board::Board;
pub mod static_evaluation;




pub trait BoardEvaluator {
    fn evaluate(board: &Board) -> i32;
}