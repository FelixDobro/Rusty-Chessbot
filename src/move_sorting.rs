use core::num;

use rayon::str::SplitWhitespace;

use crate::chess::board::Board;
use crate::chess::board::bitboard::{Bitboard, EMPTY as Empty_BB};
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
    captures: [(Move, i16); MOVE_GEN_SIZE],
    quiets: [(Move, i16); MOVE_GEN_SIZE],
    current_capture: usize,
    current_quiet: usize,
    num_good_captures: usize,
    num_quiets: usize,
    num_captures: usize,
}

impl AdvancedSorting {
    const HASH_M_VAL: i16 = 1000;
    const PIECE_VALS: [i16; 7] = [1,2,3,4,5,6,0];
    const MAX_PIECE_VAL: i16 = 6;

    pub fn new(hash_move: Move) -> Self {
        Self { 
            tt_move: hash_move, 
            captures: [(NULL_MOVE, 0i16); MOVE_GEN_SIZE], 
            quiets: [(NULL_MOVE, 0i16); MOVE_GEN_SIZE], 
            current_capture: 0,
            current_quiet: 0,
            num_good_captures:0,
            num_quiets:0,
            num_captures:0,
            stage:MoveGenStage::HashMove
        }
    }


   #[inline(always)]
    pub fn next(&mut self, board: &Board) -> Option<Move> {
        loop {
            match self.stage {
                MoveGenStage::HashMove => {
                    self.stage = MoveGenStage::GenerateCaptures;
                    if self.tt_move != NULL_MOVE{
                        return Some(self.tt_move);
                    }
                }
                
                MoveGenStage::GenerateCaptures => {
                    let captures = board.generate_captures();
                    self.score_captures(board);
                    
                    self.stage = MoveGenStage::YieldGoodCaptures;
                }
                
                MoveGenStage::YieldGoodCaptures => {
                    if self.current_capture < self.num_good_captures {
                        let m = self.captures[self.current_capture].0;
                        self.current_capture += 1;

                        if m != self.tt_move {
                            return Some(m);
                        }
                    } else {
                        self.stage = MoveGenStage::GenerateQuiets;
                    }
                }
                
                MoveGenStage::GenerateQuiets => {

                    self.score_quiets(board);
                    self.stage = MoveGenStage::YieldQuiets;
                }
                
                MoveGenStage::YieldQuiets => {
                    if self.current_quiet < self.num_quiets {
                        let m = self.quiets[self.current_quiet].0;
                        self.current_quiet += 1;
                        if m != self.tt_move {
                            return Some(m);
                        }
                    } else {
                        self.stage = MoveGenStage::YieldBadCaptures;
                    }
                }
                
                MoveGenStage::YieldBadCaptures => {
                    if self.current_capture < self.num_captures {
                        let m = self.captures[self.current_capture].0;
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
    pub fn score_captures(&mut self, board: &Board) {
        let mut moves_evaluated = [(NULL_MOVE, 0i16); MOVE_GEN_SIZE];
        let mut num_good_moves = 0;
        
        let moves = board.generate_captures();
        self.num_captures = moves.size();
        for (i, &m) in moves.as_slice().iter().enumerate() {
            let eval = Self::eval_capture(board, m);
            if eval > 0 {
                num_good_moves += 1;
            }
        
            moves_evaluated[i] = (m, eval);
        }
        moves_evaluated.sort_by_key(|entry| - entry.1);
        self.num_good_captures = num_good_moves;

        self.captures = moves_evaluated;
    }

    #[inline(always)]
    pub fn score_quiets(&mut self, board: &Board) {
        let mut moves_evaluated = [(NULL_MOVE, 0i16); MOVE_GEN_SIZE];

        let mut num_quiets = 0;
        for (i, &m) in board.generate_quiets().as_slice().iter().enumerate() {
            num_quiets += 1;
            moves_evaluated[i] = (m, 0);
        }
        moves_evaluated.sort_by_key(|entry| entry.1);
        self.num_quiets = num_quiets;
        self.quiets = moves_evaluated;
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
    pub fn eval_capture(board: &Board, m: Move) -> i16 {
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

    pub fn sort_only_captures(board: &Board, hash_move: Move) -> MoveList<64>{
        let mut moves_evaluated = [(NULL_MOVE, 0i16); 64];
        let moves = board.generate_captures();
        let num_captures = moves.size();
        for (i, &m) in moves.as_slice().iter().enumerate() {
            let eval = if m == hash_move {Self::HASH_M_VAL} else {Self::eval_capture(board, m)};
            moves_evaluated[i] = (m, eval);
        }
        moves_evaluated
        .sort_by_key(|entry| - entry.1);

        let mut result = [NULL_MOVE; 64];
        for i in 0..num_captures {
            result[i] = moves_evaluated[i].0;
        }
        MoveList::from_slice(result, num_captures)
    }

    pub fn set_hash_m(&mut self, m: Move) {
        if m != NULL_MOVE {
            self.tt_move = m;
        }
    
    }
}



    

