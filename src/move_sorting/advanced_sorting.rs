use crate::chess::board::Board;
use crate::chess::board::bitboard::EMPTY as Empty_BB;
use crate::chess::chess_move::MOVE_GEN_SIZE;
use crate::chess::chess_move::Move;
use crate::chess::chess_move::MoveList;
use crate::chess::chess_move::NULL_MOVE;
use crate::chess::constants::Color::{Black, White};
use crate::chess::constants::NUM_PIECES;
use crate::chess::constants::{BlackSide, WhiteSide};
use crate::chess::square::Square;
use crate::move_sorting::EvaluatedMoveList;
use crate::search::MAX_SEARCH_DEPTH;
use crate::search::simple_search::HistroyT;
use crate::search::simple_search::SearchStack;

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

pub struct AdvancedSorting {
    tt_move: Move,
    stage: MoveGenStage,
    captures: EvaluatedMoveList<MOVE_GEN_SIZE>,
    quiets: EvaluatedMoveList<MOVE_GEN_SIZE>,
    current_capture: usize,
    num_good_captures: usize,
    num_captures: usize,
}

impl AdvancedSorting {
    pub const HASH_M_VAL: i32 = 2i32.pow(20);
    pub const VICTIM_UNITS: i32 = 2i32.pow(14);
    pub const VICTIM_VALS: [i32; 7] = [
        1 * Self::VICTIM_UNITS,
        2 * Self::VICTIM_UNITS,
        3 * Self::VICTIM_UNITS,
        4 * Self::VICTIM_UNITS,
        5 * Self::VICTIM_UNITS,
        6 * Self::VICTIM_UNITS,
        0,
    ];

    pub const SSE_UNIT: i32 = 2i32.pow(17);
    pub const SSE_VALS: [i32; 7] = [
        1 * Self::SSE_UNIT,
        2 * Self::SSE_UNIT,
        3 * Self::SSE_UNIT,
        4 * Self::SSE_UNIT,
        5 * Self::SSE_UNIT,
        6 * Self::SSE_UNIT,
        0,
    ];
    pub fn new(hash_move: Move) -> Self {
        Self {
            tt_move: hash_move,
            captures: EvaluatedMoveList::new(),
            quiets: EvaluatedMoveList::new(),
            current_capture: 0,
            num_good_captures: 0,
            num_captures: 0,
            stage: MoveGenStage::HashMove,
        }
    }

    #[inline(always)]
    pub fn next(
        &mut self,
        board: &Board,
        history_table: &HistroyT,
        search_stack: &SearchStack<MAX_SEARCH_DEPTH>,
    ) -> Option<Move> {
        loop {
            match self.stage {
                MoveGenStage::HashMove => {
                    self.stage = MoveGenStage::GenerateCaptures;
                    if self.tt_move != NULL_MOVE {
                        return Some(self.tt_move);
                    }
                }

                MoveGenStage::GenerateCaptures => {
                    self.score_captures(board, history_table);
                    self.stage = MoveGenStage::YieldGoodCaptures;
                }

                MoveGenStage::YieldGoodCaptures => {
                    if self.current_capture < self.num_good_captures {
                        let m = self.captures.selection_sort_next().unwrap().0;
                        self.current_capture += 1;

                        if m != self.tt_move {
                            return Some(m);
                        }
                    } else {
                        self.stage = MoveGenStage::GenerateQuiets;
                    }
                }

                MoveGenStage::GenerateQuiets => {
                    self.score_quiets(board, history_table, search_stack);
                    self.stage = MoveGenStage::YieldQuiets;
                }

                MoveGenStage::YieldQuiets => {
                    if let Some((m, _)) = self.quiets.selection_sort_next() {
                        if m != self.tt_move {
                            return Some(m);
                        }
                    } else {
                        self.stage = MoveGenStage::YieldBadCaptures;
                    }
                }

                MoveGenStage::YieldBadCaptures => {
                    if self.current_capture < self.num_captures {
                        let m = self.captures.selection_sort_next().unwrap().0;
                        self.current_capture += 1;
                        if m != self.tt_move {
                            return Some(m);
                        }
                    } else {
                        self.stage = MoveGenStage::Done;
                    }
                }

                MoveGenStage::Done => return None,
            }
        }
    }

    #[inline(always)]
    pub fn score_captures(&mut self, board: &Board, history_table: &HistroyT) {
        let mut moves_evaluated = EvaluatedMoveList::new();
        let mut num_good_moves = 0;

        let moves = board.generate_captures();
        self.num_captures = moves.size();
        for &m in moves.as_slice().iter() {
            let eval = Self::eval_capture(board, m, history_table);
            if eval > 0 {
                num_good_moves += 1;
            }

            moves_evaluated.push(m, eval as i32);
        }

        self.num_good_captures = num_good_moves;
        self.captures = moves_evaluated;
    }

    #[inline(always)]
    pub fn score_quiets(
        &mut self,
        board: &Board,
        history_table: &HistroyT,
        search_stack: &SearchStack<MAX_SEARCH_DEPTH>,
    ) {
        let mut moves_evaluated = EvaluatedMoveList::new();
        let turn = board.get_turn();
        for m in board.generate_quiets().as_slice().iter() {
            let mut value = 0;
            let moved_piece = board.get_piece_w_color(m.from());
            value += history_table.continuation_val(search_stack, moved_piece, *m);
            value += history_table.main_val(turn, *m);
            moves_evaluated.push(*m, value);
        }
        self.quiets = moves_evaluated;
    }

    #[inline(always)]
    pub fn static_e_e(board: &Board, m: Move) -> i32 {
        let to = m.to();
        let mut attack_mask = board.attack_mask(to);
        let mut turn = board.get_turn();
        let mut occupied = board.get_occupied();
        let mut index = 1;
        let mut gain = [0i32; 32];
        let mut already_attacked = Empty_BB;

        gain[0] = Self::SSE_VALS[board.get_piece(to).index()];
        while attack_mask != Empty_BB {
            let is_white_turn = turn.is_white();
            let lva = match turn {
                White => board.get_lva::<WhiteSide>(attack_mask),
                Black => board.get_lva::<BlackSide>(attack_mask),
                _ => panic!("No ones turn?"),
            };

            if lva == Square::UNDEFINED {
                break;
            }

            let piece = board.get_piece(lva);

            gain[index] = Self::SSE_VALS[piece.index()] - gain[index - 1];
            let attacker_bb = lva.to_bitboard();
            attack_mask ^= attacker_bb;
            occupied ^= attacker_bb;
            already_attacked ^= attacker_bb;

            if is_white_turn {
                attack_mask = board.update_attack_board::<WhiteSide>(
                    to,
                    attack_mask,
                    occupied,
                    already_attacked,
                );
            } else {
                attack_mask = board.update_attack_board::<BlackSide>(
                    to,
                    attack_mask,
                    occupied,
                    already_attacked,
                );
            }
            turn = turn.opposite();
            index += 1;
        }

        index -= 1;
        while index > 0 {
            gain[index - 1] = i32::max(gain[index - 1], -gain[index]);
            index -= 1;
        }

        return gain[0];
    }

    #[inline(always)]
    pub fn eval_capture(board: &Board, m: Move, history_table: &HistroyT) -> i32 {
        debug_assert!(m.is_capture(), "Move is non-capture");
        let captured_piece = board.get_captured(m);
        let moved_piece = board.get_piece_w_color(m.from());

        debug_assert!(
            captured_piece.index() < NUM_PIECES,
            "Captured piece is invalid"
        );
        let static_exchange_e = Self::static_e_e(board, m);
        let victim_score = Self::VICTIM_VALS[captured_piece.index()];
        let attacker_bonus = history_table.capture_val(moved_piece, m, captured_piece.index());
        return victim_score + attacker_bonus + static_exchange_e;
    }

    pub fn sort_only_captures(
        board: &Board,
        hash_move: Move,
        history_table: &HistroyT,
    ) -> EvaluatedMoveList<MOVE_GEN_SIZE> {
        let mut moves_evaluated: EvaluatedMoveList<{ MOVE_GEN_SIZE }> = EvaluatedMoveList::new();
        let moves = board.generate_captures();

        for &m in moves.as_slice().iter() {
            let eval = if m == hash_move {
                Self::HASH_M_VAL
            } else {
                Self::eval_capture(board, m, history_table)
            };
            moves_evaluated.push(m, eval);
        }
        moves_evaluated
    }
}

// #[cfg(test)]
// mod test {
//     use crate::{
//         chess::{board::Board, chess_move::NULL_MOVE},
//         move_sorting::advanced_sorting::AdvancedSorting,
//     };

//     #[test]
//     fn insert_empty_moves() {
//         let mut sorter = AdvancedSorting::new(NULL_MOVE);
//         let board = Board::from_fen("6k1/8/6K1/8/8/8/8/1R6 w - - 0 1").unwrap();
//         let m = sorter.next(&board, &[[[0i16; 64]; 64]; 2]);
//         assert!(m.is_some(), "No moves found");
//     }
// }
