use crate::chess::board::Board;
use crate::chess::board::evaluation::{CHECK_MATE, NEG_INFINITY, POSITIVE_INFINITY};
use crate::chess::chess_move::NULL_MOVE;
use crate::move_sorting::{AdvancedSorting, MoveGenStage, NumericSorting};
use crate::search::Ntype::Exact;
use crate::search::{Ntype, SearchAlgorithm, SearchLimits, SearchResult, TTable, TTableEntry};

pub struct Negamax {
    nodes_searched: u64,
}

impl Negamax {
    pub fn new() -> Self {
        return Self { nodes_searched: 0 };
    }

    pub fn negamax(&mut self, board: &mut Board, depth: u8) -> Option<SearchResult> {
        self.nodes_searched = 0;
        let mut best_val = NEG_INFINITY;
        let mut alpha = NEG_INFINITY;
        let mut best_move = None;

        for &m in NumericSorting::move_iter(&mut board.generate_pseudolegals()) {
            if board.make_pl_move::<true>(m) {
                let value = -self.negamax_p(board, depth - 1, NEG_INFINITY, -alpha);
                board.unmake_pl_move(m);
                if value > best_val {
                    best_move = Some(m);
                    best_val = value;
                    alpha = best_val;
                }
            }
        }
        if let Some(m) = best_move {
            return Some(SearchResult {
                best_move: m,
                evaluation: best_val,
                nodes_searched: self.nodes_searched,
                depth: depth,
            });
        }
        None
    }

    fn negamax_p(&mut self, board: &mut Board, depth: u8, mut alpha: i16, beta: i16) -> i16 {
        self.nodes_searched += 1;

        if depth == 0 {
            return board.eval();
        }
        if board.can_claim_draw() {
            return 0;
        }

        let mut num_moves_found = 0;
        for &m in NumericSorting::move_iter(&mut board.generate_pseudolegals()) {
            if board.make_pl_move::<true>(m) {
                num_moves_found += 1;
                let new_eval = -self.negamax_p(board, depth - 1, -beta, -alpha);
                board.unmake_pl_move(m);
                if new_eval >= beta {
                    return beta;
                }
                if new_eval > alpha {
                    alpha = new_eval;
                }
            }
        }
        if num_moves_found == 0 {
            let res = board.result();
            return res;
        }

        alpha
    }
}

impl SearchAlgorithm for Negamax {
    fn search(&mut self, board: &mut Board, limits: &SearchLimits) -> Option<SearchResult> {
        self.negamax(board, limits.max_depth.unwrap())
    }
}

pub struct NegamaxTT {
    ttable: TTable,
    age: u8,
    nodes_searched: u64,
}

impl NegamaxTT {
    pub fn new(ttsize: usize) -> Self {
        Self {
            ttable: TTable::new(ttsize),
            age: 0,
            nodes_searched: 0,
        }
    }

    pub fn quiesence_search(&mut self, board: &mut Board, mut alpha: i16, beta: i16) -> i16 {
        self.nodes_searched += 1;
        if board.can_claim_draw() {
            return 0
        }
        
        let mut tt_move = NULL_MOVE;
        if let Some(entry) = self.ttable.get(board.get_hash()) {
            if entry.depth >= 0 {
                match entry.ntype {
                    Ntype::Exact => {
                        return entry.score;
                    }
                    Ntype::Lower => {
                        if beta <= entry.score {
                            return entry.score;
                        }
                    }
                    Ntype::Upper => {
                        if alpha >= entry.score {
                            return entry.score;
                        }
                    }
                };
            }
            tt_move = entry.best_move;
        }

        let stand_part = board.eval();

        if stand_part >= beta {
            return beta
        }
        if alpha < stand_part {
            alpha = stand_part;
        }

        let mut num_moves_executed = 0;
        for &m in board.generate_captures().as_slice() {
            if board.make_pl_move::<true>(m) {
                num_moves_executed += 1;
                let score = - self.quiesence_search(board, -beta, -alpha);
                board.unmake_pl_move(m);
                if score >= beta { return beta; }
                if score > alpha {alpha = score }
            }
        } 
        
        if num_moves_executed == 0 {
            return board.result();
        }
        
        alpha
    }

    pub fn negamax(&mut self, board: &mut Board, depth: u8) -> Option<SearchResult> {
        self.nodes_searched = 0;
        let mut best_val = NEG_INFINITY;
        let mut alpha = NEG_INFINITY;

        let mut tt_move = NULL_MOVE;
        if let Some(entry) = self.ttable.get(board.get_hash()) {
            tt_move = entry.best_move;
        }

        let mut sorter = AdvancedSorting::new(tt_move);
        let mut ntype = Ntype::Upper;
        let mut best_move = NULL_MOVE;
        while let Some(m) = sorter.next(board) {
            if board.make_pl_move::<true>(m) {
                let value = -self.negamax_p(board, depth - 1, NEG_INFINITY, -alpha);

                board.unmake_pl_move(m);
                if value > best_val {
                    best_move = m;
                    best_val = value;
                    alpha = best_val;
                }
            }
        }

        if self.nodes_searched == 0 {
            let res = board.result();
            self.ttable.insert(TTableEntry {
                hash: board.get_hash(),
                best_move: if best_move != NULL_MOVE {
                    best_move
                } else {
                    tt_move
                },
                depth: depth,
                score: res,
                ntype: Exact,
                age: self.age,
            });
            return None;
        }

        self.ttable.insert(TTableEntry {
            hash: board.get_hash(),
            best_move: if best_move != NULL_MOVE {
                best_move
            } else {
                tt_move
            },
            depth: depth,
            score: alpha,
            ntype: ntype,
            age: self.age,
        });

        self.age = self.age.wrapping_add(1);

        if best_move != NULL_MOVE {
            return Some(SearchResult {
                best_move: best_move,
                evaluation: best_val,
                nodes_searched: self.nodes_searched,
                depth: depth,
            });
        }
        None
    }

    fn negamax_p(&mut self, board: &mut Board, depth: u8, mut alpha: i16, beta: i16) -> i16 {
        self.nodes_searched += 1;
        if board.can_claim_draw() {
            return 0;
        }
        let mut tt_move = NULL_MOVE;
        if let Some(entry) = self.ttable.get(board.get_hash()) {
            if entry.depth >= depth {
                match entry.ntype {
                    Ntype::Exact => {
                        return entry.score;
                    }
                    Ntype::Lower => {
                        if beta <= entry.score {
                            return entry.score;
                        }
                    }
                    Ntype::Upper => {
                        if alpha >= entry.score {
                            return entry.score;
                        }
                    }
                };
            }
            tt_move = entry.best_move;
        }
        
        if depth == 0 {
            self.nodes_searched -= 1;
            return self.quiesence_search(board, alpha, beta);
        }
    

        let mut ntype = Ntype::Upper;
        let mut best_move = NULL_MOVE;
        let mut num_moves = 0;
        let mut move_iter = AdvancedSorting::new(tt_move);
        while let Some(m) = move_iter.next(board) {
            if board.make_pl_move::<true>(m) {
                num_moves += 1;
                let new_eval = -self.negamax_p(board, depth - 1, -beta, -alpha);
                board.unmake_pl_move(m);
                if new_eval >= beta {
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
            let res = board.result();
            self.ttable.insert(TTableEntry {
                hash: board.get_hash(),
                best_move: if best_move != NULL_MOVE {
                best_move
            } else {
                tt_move
            },
                depth: depth,
                score: res,
                ntype: Exact,
                age: self.age,
            });
            return res;
        }

        self.ttable.insert(TTableEntry {
            hash: board.get_hash(),
            best_move: if best_move != NULL_MOVE {
                best_move
            } else {
                tt_move
            },
            depth: depth,
            score: alpha,
            ntype: ntype,
            age: self.age,
        });
        alpha
    }
}

impl SearchAlgorithm for NegamaxTT {
    fn search(&mut self, board: &mut Board, limits: &SearchLimits) -> Option<SearchResult> {
        self.negamax(board, limits.max_depth.unwrap())
    }
}
