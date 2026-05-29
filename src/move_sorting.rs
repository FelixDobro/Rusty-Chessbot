use rayon::str::SplitWhitespace;

use crate::chess::board::Board;
use crate::chess::board::bitboard::{Bitboard, EMPTY as Empty_BB};
use crate::chess::board::FenError::HalfMove;
use crate::chess::board::move_gen;
use crate::chess::board::evaluation::{*};
use crate::chess::chess_move::MoveList;
use crate::chess::chess_move::Move;
use crate::chess::chess_move::MOVE_GEN_SIZE;
use crate::chess::chess_move::NULL_MOVE;
use crate::chess::constants::Color::{Black, White};
use crate::chess::constants::Piece::{*};
use crate::chess::constants::{BlackSide, WhiteSide};
use crate::chess::square::Square;


pub struct NumericSorting;
impl NumericSorting {
    pub fn move_iter(move_list: &mut MoveList<MOVE_GEN_SIZE>) -> impl Iterator<Item = &Move> {
        move_list.as_mut_slice().sort_unstable_by(|a,b| b.cmp(a));
        move_list.as_slice().iter()
    }
}



pub struct AdvancedSorting {
    tt_move: Option<Move>,
    generate_moves: bool,
    hash_move: bool,
    moves: [(Move, i16) ; MOVE_GEN_SIZE],
    current: usize,
    done: bool
}

impl AdvancedSorting {
    const HASH_M_VAL: i16 = 1000;
    const PIECE_VALS: [i16; 7] = [1,2,3,4,5,6,0];
    const MAX_PIECE_VAL: i16 = 6;

    pub fn new(hash_move: Option<Move>) -> Self {
        Self { tt_move: hash_move, moves: [(NULL_MOVE, 0i16); MOVE_GEN_SIZE], generate_moves: true, hash_move: true, current: 0, done: false}
    }

    #[inline(always)]
    pub fn next(&mut self, board: &Board) -> Option<Move> {

        if self.hash_move {
            if self.tt_move.is_some() {
                self.hash_move = false;
                return self.tt_move;
            }
            self.hash_move = false;
            return self.next(board);
        }
        else if self.generate_moves {

            let moves = board.generate_pseudolegals();

            let mut scored_moves: [(Move, i16); 256] = [(NULL_MOVE, 0); MOVE_GEN_SIZE];
            let len = moves.size();
            let move_slice = moves.as_slice();
            for i in 0..len {
                let m = move_slice[i];
                let score = if self.tt_move.is_some_and(|tt_move| tt_move == m) { - Self::HASH_M_VAL }
                else { Self::eval_move(board, m) };
                scored_moves[i] = (m, score);
            }

            self.generate_moves = false;
            let active_slice = &mut scored_moves[0..len];
            active_slice.sort_unstable_by_key(|&(_, score)| score);
            self.moves[0..len].copy_from_slice(active_slice);
            self.current = len;
            if len == 0 {self.done = true}
        }
        if self.current == 0 {
            self.done = true
        }
        if self.done {
            return None;
        }

        self.current -= 1;
        Some(self.moves[self.current].0)
    }

    #[inline(always)]
    pub fn static_e_e(board: &Board, m: Move) -> i16 {
        let to = m.to();
        let mut attack_mask = board.attack_mask(to);
        let mut turn = board.get_turn();
        let mut occupied = board.get_occupied();
        let mut index = 1;
        let mut gain = [0i16; 32];
        let mut already_attacked = Empty_BB;
        gain[0] = SIMPLE_CP_VALUES[board.get_piece(to).index()];
        while attack_mask != Empty_BB {
            let is_white_turn = turn.is_white();
            let lva = match turn {
                White => board.get_lva::<WhiteSide>(attack_mask),
                Black => board.get_lva::<BlackSide>(attack_mask),
                _ => panic!("No ones turn?")
            };

            if lva == Square::UNDEFINED {
                break
            }
           
            let piece = board.get_piece(lva);

            gain[index] = SIMPLE_CP_VALUES[piece.index()] - gain[index-1];
            let attacker_bb = lva.to_bitboard();
            attack_mask ^= attacker_bb;
            occupied ^= attacker_bb;
            already_attacked ^= attacker_bb;
           
            if is_white_turn {
                attack_mask = board.update_attack_board::<WhiteSide>(to, attack_mask, occupied, already_attacked);
            }
            else {
                attack_mask = board.update_attack_board::<BlackSide>(to, attack_mask, occupied, already_attacked);
            }
            turn = turn.opposite();
            index += 1;
        }

        index -= 1; 
        while index > 0 {

            gain[index - 1] = i16::max(gain[index - 1], -gain[index]);
            index -= 1;
        }

        return gain[0];
    }

    #[inline(always)]
    pub fn eval_move(board: &Board, m: Move) -> i16 {
        let (from, to) = (m.from(), m.to());
        let (from_piece, to_piece) = (board.get_piece(from), board.get_piece(to));
        if to_piece != Empty {
            let static_exchange_e = Self::static_e_e(board, m);
            let victim_score = Self::PIECE_VALS[to_piece.index()] * 10;
            let attacker_bonus = Self::MAX_PIECE_VAL - Self::PIECE_VALS[from_piece.index()];
            return victim_score + attacker_bonus + static_exchange_e;
        }
        0
    }

    pub fn set_hash_m(&mut self, m: Move) {
        if m != NULL_MOVE {
            self.tt_move = Some(m);
        }
    
    }
}



    

