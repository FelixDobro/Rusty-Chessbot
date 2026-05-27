
use crate::chess::board::{Board};
use crate::chess::chess_move::NULL_MOVE;
use crate::move_sorting::{AdvancedSorting, NumericSorting};
use crate::search::{Ntype, SearchAlgorithm, SearchResult, TTable, TTableEntry};
use crate::chess::board::evaluation::{NEG_INFINITY, CHECK_MATE, POSITIVE_INFINITY};


pub struct Negamax;

   

impl Negamax {


    pub fn negamax(&mut self, board: &mut Board, depth: u8) -> Option<SearchResult> {
    let mut best_val = NEG_INFINITY;
    let mut alpha = NEG_INFINITY;
    let mut best_move= None ;


    for &m in NumericSorting::move_iter(&mut board.generate_pseudolegals()) {
        if board.make_pl_move::<true>(m) {
            let value = - self.negamax_p(board, depth - 1, NEG_INFINITY, -alpha);
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
        mut alpha: i16,
        beta: i16,
    ) -> i16 {
        if depth == 0 {
            return board.eval();
        }  
        if board.can_claim_draw() {
            return 0
        }

        let mut num_moves_found = 0;
        for &m in NumericSorting::move_iter(&mut board.generate_pseudolegals()) {
            if board.make_pl_move::<true>(m) {
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


pub struct NegamaxTT {
    ttable: TTable,
    age: u8,

}

impl NegamaxTT {

    pub fn new(ttsize: usize) -> Self {
        Self { ttable: TTable::new(ttsize), age: 0}

    }

    pub fn negamax(&mut self, board: &mut Board, depth: u8) -> Option<SearchResult> {
    let mut best_val = NEG_INFINITY;
    let mut alpha = NEG_INFINITY;



    let mut best_move = NULL_MOVE;
    let mut num_moves = 0;
    let mut sorter = AdvancedSorting::new(None);
    while let Some(m) = sorter.next(board) {
        if board.make_pl_move::<true>(m) {
            num_moves += 1;
            let value = - self.negamax_p(board, depth - 1, NEG_INFINITY, -alpha);
            println!("{}", value);
            board.unmake_pl_move(m);
            if value > best_val {
                best_move = m;
                best_val = value;
                alpha = best_val;
            }
        }
    }
    self.age += self.age.wrapping_add(1);

    if best_move != NULL_MOVE {
        return Some(
            SearchResult {
                best_move: best_move,
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
        mut alpha: i16,
        beta: i16,
    ) -> i16 {
        let mut move_iter = AdvancedSorting::new(None);
        if let Some(entry) = self.ttable.get(board.get_hash()) {
            if entry.depth >= depth {
                match entry.ntype {
                    Ntype::Exact => {return entry.score;},
                    Ntype::Lower => {
                        if beta <= entry.score {
                            return entry.score;
                        }
                    },
                    Ntype::Upper => {
                        if alpha >= entry.score {
                            return entry.score;
                        }
                    },
                };
            }
            move_iter.set_hash_m(entry.best_move);
        }

        if depth == 0 {
            return board.eval();
        }  
        if board.can_claim_draw() {
            return 0
        }

        let mut ntype = Ntype::Upper;
        let mut best_move = NULL_MOVE;
        let mut num_moves = 0;
        while let Some(m) = move_iter.next(board) {
            if board.make_pl_move::<true>(m) {
                num_moves += 1;
                let new_eval = - self.negamax_p(board, depth - 1, -beta, -alpha);
                board.unmake_pl_move(m);
                if new_eval >= beta{
                    self.ttable.insert(TTableEntry {
                        hash: board.get_hash(),
                        best_move: m,     
                        score: beta,       
                        depth: depth,
                        ntype: Ntype::Lower, 
                        age: self.age,
                    });
                    return beta;
                }
                if new_eval > alpha {
                    alpha = new_eval;
                    best_move = m;
                    ntype = Ntype::Exact
                }
            }
        }

        if num_moves == 0 {
            let res=  board.result();
            return res
        }

    
        self.ttable.insert(
            TTableEntry {
            hash: board.get_hash(),
            best_move: best_move,
            depth: depth,
            score: alpha,
            ntype:ntype,
            age: self.age
        }
        );
        alpha
    }
}

impl SearchAlgorithm for NegamaxTT {
    fn search(&mut self, board: &mut Board, depth: u8) -> Option<SearchResult> {
        self.negamax(board, depth)
    }
}