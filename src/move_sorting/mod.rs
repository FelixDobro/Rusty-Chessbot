use crate::chess::chess_move::Move;
use std::mem::MaybeUninit;
pub mod advanced_sorting;

#[derive(Debug, Clone)]
pub struct EvaluatedMoveList<const N: usize> {
    moves: MaybeUninit<[(Move, i32); N]>,
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
    pub fn push(&mut self, m: Move, val: i32) {
        let array_ptr = self.moves.as_mut_ptr() as *mut (Move, i32);
        unsafe {
            let target_ptr = array_ptr.add(self.count);
            target_ptr.write((m, val));
        }
        self.count += 1;
    }

    #[inline(always)]
    fn get_item(&self, index: usize) -> (Move, i32) {
        debug_assert!(index < self.count);
        let array_ptr: *const (Move, i32) = self.moves.as_ptr() as *const (Move, i32);
        unsafe {
            let target_ptr = array_ptr.add(index);
            let entry = target_ptr.read();
            return entry;
        }
    }

    #[inline(always)]
    fn set_item(&mut self, index: usize, item: (Move, i32)) {
        debug_assert!(index < self.count);
        let array_ptr: *mut (Move, i32) = self.moves.as_mut_ptr() as *mut (Move, i32);
        unsafe {
            let target_ptr = array_ptr.add(index);
            target_ptr.write(item);
        }
    }

    #[inline(always)]
    pub fn selection_sort_next(&mut self) -> Option<(Move, i32)> {
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
        Some(best_item)
    }

    #[allow(dead_code)]
    pub fn print_list(&self) {
        for i in 0..self.count {
            println!("{}", self.get_item(i).0);
        }
    }

    #[allow(dead_code)]
    #[inline(always)]
    pub fn size(&self) -> usize {
        self.count
    }

    #[allow(dead_code)]
    pub fn search_index(&self) -> usize {
        self.search_index
    }
}

#[cfg(test)]
mod test {
    use crate::chess::chess_move::MOVE_GEN_SIZE;
    use crate::{
        chess::{chess_move::Move, square::Square},
        move_sorting::EvaluatedMoveList,
    };

    #[test]
    fn insert_empty_moves() {
        let mut move_list: EvaluatedMoveList<MOVE_GEN_SIZE> = EvaluatedMoveList::new();
        let first_move = Move::new(Square::E2, Square::E4, 1);
        move_list.push(first_move, 100);
        let second_move = Move::new(Square::G1, Square::F3, 0);
        move_list.push(second_move, -100);

        assert_eq!(
            move_list.selection_sort_next(),
            Some((first_move, 100)),
            "Selection sort does not find right move"
        );
        assert_eq!(
            move_list.selection_sort_next(),
            Some((second_move, -100)),
            "Selection sort does not find right move"
        );
    }

    #[test]
    fn push_and_get() {
        let mut moves: EvaluatedMoveList<MOVE_GEN_SIZE> = EvaluatedMoveList::new();
        let m = Move::new(Square::E2, Square::E3, 0);
        moves.push(m, 0);
        let recieved = moves.get_item(0);

        assert_eq!(recieved, (m, 0), "Move incorrectly inserted");

        assert_eq!(moves.search_index(), 0, "Search index should start at 0");
        let selection_recieved = moves.selection_sort_next();
        assert_eq!(
            moves.search_index(),
            1,
            "List does not properly conitnue search (search_index 0 -> search_index 1"
        );

        assert!(
            selection_recieved.map_or(false, |m| m == recieved),
            "selection_sort does not find right move"
        );
        assert!(
            moves.selection_sort_next().is_none(),
            "List yields move that does not exist"
        );
    }

    #[test]
    fn insert_multiple_moves() {
        let mut moves: EvaluatedMoveList<MOVE_GEN_SIZE> = EvaluatedMoveList::new();
        let m1 = Move::new(Square::E2, Square::E3, 0);
        let m2 = Move::new(Square::E2, Square::E4, 1);
        let m3 = Move::new(Square::E3, Square::E4, 2);
        let m4 = Move::new(Square::E4, Square::E5, 3);

        moves.push(m1, 1);
        moves.push(m2, 10);
        moves.push(m3, 25);
        moves.push(m4, 0);

        let recieved = moves.get_item(2);

        assert_eq!(recieved.0, m3, "Move incorrectly inserted");

        let selection_recieved = moves.selection_sort_next();
        assert!(
            selection_recieved.map_or(false, |(m, _)| m == m3),
            "selection_sort does not find next hightest_value move"
        );

        let selection_recieved = moves.selection_sort_next();
        assert!(
            selection_recieved.map_or(false, |(m, _)| m == m2),
            "selection_sort does not find next hightest_value move"
        );

        let selection_recieved = moves.selection_sort_next();
        assert!(
            selection_recieved.map_or(false, |(m, _)| m == m1),
            "selection_sort does not find next hightest_value move"
        );

        let selection_recieved = moves.selection_sort_next();
        assert!(
            selection_recieved.map_or(false, |(m, _)| m == m4),
            "selection_sort does not find next hightest_value move"
        );

        assert!(
            moves.selection_sort_next().is_none(),
            "List yields move that does not exist"
        );

        // test if the entries are rightfully swapped
        let pos0 = moves.get_item(0).0;
        let pos1 = moves.get_item(1).0;
        let pos2 = moves.get_item(2).0;
        let pos3 = moves.get_item(3).0;

        assert_eq!(pos0, m3, "m3 is the best move and should be on index 0");
        assert_eq!(
            pos1, m2,
            "m2 is the second best move and should be on index 1"
        );
        assert_eq!(
            pos2, m1,
            "m1 is the third best move and should be on index 2"
        );
        assert_eq!(pos3, m4, "m4 did not change position");
    }

    // #[test]
    // fn do_history_update() {
    //     let mut table = [[[0i16; 64]; 64]; 2];
    //     let mut moves: EvaluatedMoveList<MOVE_GEN_SIZE> = EvaluatedMoveList::new();

    //     let m1 = Move::new(Square::E2, Square::E3, 0);
    //     let m2 = Move::new(Square::E2, Square::E4, 1);
    //     let m3 = Move::new(Square::E3, Square::E4, 2);
    //     let m4 = Move::new(Square::E4, Square::E5, 3);
    //     let m1_val = 1;
    //     let m2_val = 2;
    //     let m3_val = 3;
    //     let m4_val = 4;
    //     moves.push(m1, m1_val);
    //     moves.push(m2, m2_val);
    //     moves.push(m3, m3_val);
    //     moves.push(m4, m4_val);

    //     // m4 and m3 and m2 are being made
    //     moves.selection_sort_next();
    //     moves.selection_sort_next();
    //     moves.selection_sort_next();

    //     moves.history_update(3, Color::White, &mut table, -100, 100);

    //     let m1_val_a = table[0][m1.from().usize()][m1.to().usize()];
    //     let m2_val_a = table[0][m2.from().usize()][m2.to().usize()];
    //     let m3_val_a = table[0][m3.from().usize()][m3.to().usize()];
    //     let m4_val_a = table[0][m4.from().usize()][m4.to().usize()];

    //     assert!(
    //         m1_val_a == 0,
    //         "Move m1 was never used in search but got punished; Eval: {}",
    //         m1_val_a
    //     );
    //     assert!(
    //         m2_val_a > 0,
    //         "Move m2 was the one move ending the search and should get rewarded Eval; {}",
    //         m2_val_a
    //     );
    //     assert!(
    //         m3_val_a < 0,
    //         "Move m3 did not exit the search, should be punished; Eval: {}",
    //         m3_val_a
    //     );
    //     assert!(
    //         m4_val_a < 0,
    //         "Move m4 did not exit the search, should be punished; Eval: {}",
    //         m4_val_a
    //     )
    // }
}
