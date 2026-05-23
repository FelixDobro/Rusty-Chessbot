use crate::chess::chess_move::MoveList;
use crate::chess::chess_move::Move;
use crate::chess::chess_move::MOVE_GEN_SIZE;


pub struct NoSorting;

impl NoSorting {
    pub fn move_iter(move_list: &mut MoveList<MOVE_GEN_SIZE>) -> impl Iterator<Item = &Move> {
        return move_list.as_slice().iter()
    }
}

pub struct NumericSorting;
impl NumericSorting {
    pub fn move_iter(move_list: &mut MoveList<MOVE_GEN_SIZE>) -> impl Iterator<Item = &Move> {
        move_list.as_mut_slice().sort_unstable_by(|a,b| b.cmp(a));
        move_list.as_slice().iter()
    }
}