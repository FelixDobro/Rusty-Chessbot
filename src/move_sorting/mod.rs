use std::{collections::btree_map::Entry, mem::MaybeUninit};

use crate::chess::chess_move::{MOVE_GEN_SIZE, Move, MoveList};

pub mod advanced_sorting;
pub struct NumericSorting;
impl NumericSorting {
    pub fn move_iter(move_list: &mut MoveList<MOVE_GEN_SIZE>) -> impl Iterator<Item = &Move> {
        move_list.as_mut_slice().sort_unstable_by(|a, b| b.cmp(a));
        move_list.as_slice().iter()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum MoveGenStage {
    HashMove,
    GenerateCaptures,
    YieldGoodCaptures,
    GenerateQuiets,
    YieldQuiets,
    YieldBadCaptures,
    Done,
}

#[derive(Debug, Clone)]
pub struct EvaluatedMoveList<const N: usize> {
    moves: MaybeUninit<[(Move, i16); N]>,
    count: usize,
    search_index: usize,
}

impl<const N: usize> EvaluatedMoveList<N> {
    pub fn new() -> Self {
        Self {
            moves: MaybeUninit::uninit(),
            count: 0,
            search_index: 0,
        }
    }

    #[inline(always)]
    pub fn push(&mut self, m: Move, val: i16) {
        let array_ptr = self.moves.as_mut_ptr() as *mut (Move, i16);
        unsafe {
            let target_ptr = array_ptr.add(self.count);
            target_ptr.write((m, val));
        }
        self.count += 1;
    }

    #[inline(always)]
    fn get_item(&self, index: usize) -> (Move, i16) {
        debug_assert!(index < self.count);
        let array_ptr: *const (Move, i16) = self.moves.as_ptr() as *const (Move, i16);
        unsafe {
            let target_ptr = array_ptr.add(index);
            let entry = target_ptr.read();
            return entry;
        }
    }


    #[inline(always)]
    fn set_item(&mut self, index: usize, item: (Move, i16))  {
        debug_assert!(index < self.count);
        let array_ptr: *mut (Move, i16) = self.moves.as_mut_ptr() as *mut (Move, i16);
        unsafe {
            let target_ptr = array_ptr.add(self.count);
            target_ptr.write(item);
        }
    }

    #[inline(always)]
    pub fn selection_sort_next(&mut self) -> Option<Move> {
        if self.count == self.search_index {
            return None;
        }

        let size = self.count;
        let mut best_index = self.search_index;
        let mut best_val = self.get_item(best_index).1;
        let mut compare_index = self.search_index + 1;

        while compare_index < size {
            let compare_val = self.get_item(compare_index).1;
            if compare_val > best_val {
                best_index = compare_index;
                best_val = compare_val;
            }
            compare_index += 1;
        }
        let best_item = self.get_item(best_index);
        if best_index != self.search_index {
            let current_first = self.get_item(self.search_index);
            self.set_item(best_index, current_first);
            self.set_item(self.search_index, best_item);
        }
        self.search_index += 1;
        Some(best_item.0)
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &[Move] {
        unsafe {
            let array_ptr = self.moves.as_ptr() as *const Move;
            std::slice::from_raw_parts(array_ptr, self.count)
        }
    }

    pub fn print_list(&self) {
        for m in self.as_slice() {
            println!("{}", m)
        }
    }

    #[inline(always)]
    pub fn size(&self) -> usize {
        self.count
    }
}



#[cfg(test)]
mod test {
    use crate::{chess::{board::Board, chess_move::{Move, NULL_MOVE}, square::Square}, move_sorting::{EvaluatedMoveList, advanced_sorting::AdvancedSorting}};
    use crate::chess::chess_move::MOVE_GEN_SIZE;

    #[test]
    fn insert_empty_moves() {

        let mut move_list: EvaluatedMoveList<MOVE_GEN_SIZE> = EvaluatedMoveList::new();        
        let first_move =Move::new(Square::E2, Square::E4, 1); 
        move_list.push(first_move, 100);
        let second_move = Move::new(Square::G1, Square::F3, 0);
        move_list.push(second_move, -100);

        assert_eq!(move_list.selection_sort_next(), Some(first_move),"Selection sort does not find right move");
        assert_eq!(move_list.selection_sort_next(), Some(second_move), "Selection sort does not find right move");
    }
}