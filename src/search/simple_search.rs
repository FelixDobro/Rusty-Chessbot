use crate::chess::board::{Board};
use crate::move_sorting::{NumericSorting};
use crate::search::{SearchAlgorithm, SearchResult};


pub struct Negamax;


impl Negamax {

    const INFINITY: i32 = 2_000_000;
    const NEG_INFINITY: i32 = -2_000_000;


    pub fn negamax(&mut self, board: &mut Board, depth: u8) -> Option<SearchResult> {
    let mut best_val = Self::NEG_INFINITY;
    let mut alpha = Self::NEG_INFINITY;
    let mut best_move= None ;


    for &m in NumericSorting::move_iter(&mut board.generate_pseudolegals()) {
        if board.make_pl_move(m) {
            let value = - self.negamax_p(board, depth - 1, Self::NEG_INFINITY, -alpha);
            board.unmake_pl_move(m);
            if value > best_val {
                best_move = Some(m);
                best_val = value;
                alpha = best_val;
            }
        }
    }
    if let Some(m) = best_move {
        return Some(
            SearchResult {
                best_move: m,
                evaluation: best_val
            }
        )
    }
    None
}

    fn negamax_p(
        &mut self,
        board: &mut Board,
        depth: u8,
        mut alpha: i32,
        beta: i32,
    ) -> i32 {
        if depth == 0 {
            return board.get_eval();
        }  
        if board.can_claim_draw() {
            return 0
        }

        let mut num_moves_found = 0;
        for &m in NumericSorting::move_iter(&mut board.generate_pseudolegals()) {
            if board.make_pl_move(m) {
                num_moves_found += 1;
                let new_eval = - self.negamax_p(board, depth - 1, -beta, -alpha);
                board.unmake_pl_move(m);
                if new_eval >= beta{
                    return beta;
                }
                if new_eval > alpha {
                    alpha = new_eval;
                }
            }
        }
        if num_moves_found == 0 {
            let res=  board.result();
            return res
        }
        alpha
    }

    

}

impl SearchAlgorithm for Negamax {
    fn search(&mut self, board: &mut Board, depth: u8) -> Option<SearchResult> {
        self.negamax(board, depth)
    }
}