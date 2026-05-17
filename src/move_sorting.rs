use crate::chess::chessMove::MoveList;
use crate::chess::chessMove::Move;
use crate::chess::chessMove::MOVE_GEN_SIZE;

pub trait MoveSortingAlgorithm {
    fn move_iter(move_list: &mut MoveList<MOVE_GEN_SIZE>) -> impl Iterator<Item = &Move>;
}


pub struct NoSorting;

impl MoveSortingAlgorithm for NoSorting {
    fn move_iter(move_list: &mut MoveList<MOVE_GEN_SIZE>) -> impl Iterator<Item = &Move> {
        return move_list.as_slice().iter()
    }
}

pub struct NumericSorting;
impl MoveSortingAlgorithm for NumericSorting {
    fn move_iter(move_list: &mut MoveList<MOVE_GEN_SIZE>) -> impl Iterator<Item = &Move> {
        move_list.as_mut_slice().sort_unstable_by(|a,b| b.cmp(a));
        move_list.as_slice().iter()
    }
}