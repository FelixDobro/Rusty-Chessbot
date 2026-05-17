use crate::chess::Board;
use crate::chess::constants::{*};
use crate::chess::square::Square;

use crate::evaluation::BoardEvaluator;

pub struct MaterialEvaluator;

impl BoardEvaluator for MaterialEvaluator {
    fn evaluate(board: &Board) -> f32 {
        board.count_material()
    }
}