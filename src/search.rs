
use chess::{Board, CacheTable, ChessMove, Color, File, MoveGen, Rank, Square};

use crate::evaluation::{MaterialEvaluator, Evaluation};


pub trait SearchAlgorithm {
    fn search(&self, board: &Board, depth: i16) -> Option<(ChessMove, f32)>;
}

#[derive(PartialEq, PartialOrd, Clone, Copy)]
enum Bound {
    Exact, 
    Alpha,
    Beta
}

#[derive(PartialEq, PartialOrd, Clone, Copy)]
struct TableEntry {
    depth: u16,
    evaluation: f32,
    bound: Bound,
    best_move: ChessMove
}


pub struct MinimaxSearch {
    hash_table: CacheTable<TableEntry>
}



impl MinimaxSearch {

    pub fn new() -> MinimaxSearch {
        let m = ChessMove::new(
            Square::make_square(Rank::Eighth, File::A),
            Square::make_square(Rank::Eighth, File::A),
            None
        );
                MinimaxSearch { hash_table: CacheTable::new(2^28, TableEntry { depth:0, evaluation: 0.0, bound: Bound::Exact, best_move: m })}
    }

    fn minimax(&self, board: &Board, maximmizing_player: bool, depth: i16, mut alpha:f32, mut beta:f32) -> f32 {
        let hash = board.get_hash();

        if depth == 0 {
            
            if let Some(entry) = self.hash_table.get(hash) {
                return entry.evaluation;
            }
            return MaterialEvaluator::evaluate(board);
        }

        let move_gen = MoveGen::new_legal(board);

        if move_gen.len() == 0 {
            return if maximmizing_player {-9999.9} else {9999.9};
        }

        

        if maximmizing_player {
            let mut value = f32::MIN;
            
            for m in move_gen {
                let new_board = board.make_move_new(m);
                let eval = self.minimax(&new_board,  false, depth - 1, alpha, beta);
                value = value.max(eval);
                alpha = alpha.max(eval);
                if alpha >= beta {break}
            }
            value
        }
        else {
            let mut value = f32::MAX;
            for m in move_gen {
                let new_board = board.make_move_new(m);
                let eval = self.minimax(&new_board, true, depth-1, alpha, beta);
                
                value = value.min(eval);
                beta = beta.min(eval);

                if alpha >= beta {break}
            }
            value
        }
    }
}

impl SearchAlgorithm for MinimaxSearch {

    

    fn search(&self, board: &Board, depth: i16) -> Option<(ChessMove, f32)> {

        let mut best_move: Option<ChessMove> = None;
        let move_gen = MoveGen::new_legal(board); 
        
        let white = match board.side_to_move() {
            Color::White => true,
            Color::Black => false
        };
        
        let mut best_eval = if white {f32::MIN} else {f32::MAX};
        
        for m in move_gen {
            let new_board = board.make_move_new(m);
            let evaluation = self.minimax(&new_board, !white, depth - 1, f32::MIN, f32::MAX);

            if white && evaluation > best_eval {
                best_move = Some(m);
                best_eval = evaluation;
            }
            if !white && evaluation < best_eval {
                best_move = Some(m);
                best_eval = evaluation;
            }
        }
        

        if let Some(m) = best_move {
            Some((m, best_eval))
        }
        else {None}

    }
}

