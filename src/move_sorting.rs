use crate::chess::board::Board;
use crate::chess::board::FenError::HalfMove;
use crate::chess::board::move_gen;
use crate::chess::chess_move::MoveList;
use crate::chess::chess_move::Move;
use crate::chess::chess_move::MOVE_GEN_SIZE;
use crate::chess::chess_move::NULL_MOVE;
use crate::chess::constants::Piece::{*};


pub struct NumericSorting;
impl NumericSorting {
    pub fn move_iter(move_list: &mut MoveList<MOVE_GEN_SIZE>) -> impl Iterator<Item = &Move> {
        move_list.as_mut_slice().sort_unstable_by(|a,b| b.cmp(a));
        move_list.as_slice().iter()
    }
}



pub struct AdvancedSorting {
    tt_move: Option<Move>,
    moves: MoveList<MOVE_GEN_SIZE>,
    generate_moves: bool,
    hash_move: bool
}

impl AdvancedSorting {
    const HASH_M_VAL: i16 = 1000;
    const PIECE_VALS: [i16; 7] = [1,2,3,4,5,6,0];
    const MAX_PIECE_VAL: i16 = 6;

    pub fn new(hash_move: Option<Move>) -> Self {
        Self { tt_move: hash_move, moves: MoveList::new(), generate_moves: true, hash_move: true}
    }

    #[inline(always)]
    pub fn next(&mut self, board: &Board) -> Option<Move> {

        if self.hash_move {
            if let Some(m) = self.tt_move {
                self.hash_move = false;
                return self.tt_move;
            }
            self.hash_move = false;
            return self.next(board);
        }
        if self.generate_moves {
            let mut moves = board.generate_pseudolegals();
            moves.as_mut_slice()
            .sort_unstable_by_key(|&movelist_m| {
                if self.tt_move.is_some_and(|m| m == movelist_m)  {-1000}
                else {
                    Self::eval_move(board, movelist_m)
                }
            });
            self.moves = moves;
            self.generate_moves = false;
        }

        self.moves.pop_get()
    }


    #[inline(always)]
    pub fn eval_move(board: &Board, m: Move) -> i16 {
        let (from, to) = (m.from(), m.to());
        let (from_piece, to_piece) = (board.get_piece(from), board.get_piece(to));
        if to_piece != Empty {
            let victim_score = Self::PIECE_VALS[to_piece.index()] * 10;
            let attacker_bonus = Self::MAX_PIECE_VAL - Self::PIECE_VALS[from_piece.index()];
            return victim_score + attacker_bonus;
        }
        0
    }

    pub fn set_hash_m(&mut self, m: Move) {
        if m != NULL_MOVE {
            self.tt_move = Some(m);
        }
    
    }
}



    

