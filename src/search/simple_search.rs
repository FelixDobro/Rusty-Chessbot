use crate::chess::game::{Game};
use crate::chess::Board;
use crate::chess::chessMove::Move;
use crate::evaluation::BoardEvaluator;
use crate::evaluation::static_evaluation::MaterialEvaluator;
use crate::move_sorting::MoveSortingAlgorithm;
use crate::search::{SearchAlgorithm, SearchResult};


pub struct NegaMaxCopy;


impl NegaMaxCopy {

    pub fn negamax_copy<Eval: BoardEvaluator, Sort: MoveSortingAlgorithm>(game: &mut Game, depth: u8) -> Option<SearchResult> {
    let mut best_val = f32::MIN;
    let mut alpha = f32::MIN;
    let mut best_move= None ;

    let mut board = game.get_board().clone();

    for &m in Sort::move_iter(&mut board.generate_pseudolegals()) {
        if let Some(mut new_board) = board.make_pl_move_copy(m) {
            game.push_state(&new_board);
            let value = - Self::negamax_copy_p::<Eval, Sort>(game, &mut new_board, depth - 1, f32::MIN, -alpha);
            game.pop_only_state(&new_board);
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

    fn negamax_copy_p<Eval: BoardEvaluator, Sort: MoveSortingAlgorithm>(
        game: &mut Game,
        board: &mut Board, 
        depth: u8,
        mut alpha: f32,
        beta: f32,
    ) -> f32 {
        if depth == 0 {
            return Eval::evaluate(board);
        }  
        if game.can_claim_draw() {
            return 0.0
        }

        let mut num_moves_found = 0;
        for &m in Sort::move_iter(&mut board.generate_pseudolegals()) {
            if let Some(mut new_board) = board.make_pl_move_copy(m) {
                num_moves_found += 1;
                game.push_state(&new_board);

                let new_eval = - Self::negamax_copy_p::<Eval, Sort>(game, &mut new_board, depth - 1, -beta, -alpha);
                game.pop_only_state(&new_board);
                if new_eval >= beta{
                    return beta;
                }
                if new_eval > alpha {
                    alpha = new_eval;
                }
            }
        }
        if num_moves_found == 0 {
            return board.result();
        }
        alpha
    }
}

impl SearchAlgorithm for NegaMaxCopy {
    fn search<E: BoardEvaluator, M: MoveSortingAlgorithm>(&mut self, game: &mut Game, depth: u8) -> Option<SearchResult> {
        Self::negamax_copy::<E, M>(game, depth)
    }
}