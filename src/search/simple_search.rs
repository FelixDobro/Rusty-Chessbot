use crate::chess::board::Board;
use crate::chess::board::evaluation::{NEG_INFINITY, POSITIVE_INFINITY};
use crate::chess::chess_move::{Move, MoveList, NULL_MOVE, MOVE_GEN_SIZE};
use crate::chess::constants::Color;
use crate::move_sorting::advanced_sorting::{AdvancedSorting, NumericSorting};
use crate::search::Ntype::Exact;
use crate::search::{Ntype, SearchAlgorithm, SearchLimits, SearchResult, TTable, TTableEntry};


#[allow(dead_code)]
pub struct Negamax {
    nodes_searched: u64,
}

#[allow(dead_code)]
impl Negamax {

    pub fn new() -> Self {
        return Self { nodes_searched: 0};
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
    killer_table: [[Move; 3]; 64],
    history_table: [[[i16; 64]; 64]; 2],
    age: u8,
    nodes_searched: u64,
}

impl NegamaxTT {

    pub fn new(ttsize: usize) -> Self {
        Self {
            ttable: TTable::new(ttsize),
            age: 0,
            nodes_searched: 0,
            killer_table: [[NULL_MOVE; 3]; 64],
            history_table: [[[0i16; 64]; 64]; 2],
        }
    }

    pub fn reset_killers(&mut self) {
        self.killer_table = [[NULL_MOVE; 3]; 64];
    }

    #[inline(always)]
    pub fn append_killer_move(&mut self, ply: usize, m: Move) {
        self.killer_table[ply][2] = self.killer_table[ply][1];
        self.killer_table[ply][1] = self.killer_table[ply][0];
        self.killer_table[ply][0] = m;
    }

    #[inline(always)]
    pub fn score_history_move(&mut self, m: Move, bonus: i16, turn: Color) {
        let clamped_bonus = AdvancedSorting::HISTORY_MIN.max(bonus).min(AdvancedSorting::HISTORY_MAX);
        self.history_table[turn.index()][m.from().index()][m.to().index()] += 
        clamped_bonus - self.history_table[turn.index()][m.from().index()][m.to().index()] * clamped_bonus.abs() / AdvancedSorting::HISTORY_MAX;
    }

    #[inline(always)]
    pub fn punish_quiets(&mut self, moves: &MoveList<MOVE_GEN_SIZE>, depth: u8, turn: Color) {
        let bonus = -(300 * (depth as i16) - 250);
        for &m in moves.as_slice() {
            if m.is_quiet() {
                self.score_history_move(m, bonus, turn);
            }
        }
    }


    pub fn quiesence_search(&mut self, board: &mut Board, mut alpha: i16, beta: i16) -> i16 {
        self.nodes_searched += 1;
        if board.can_claim_draw() {
            return 0
        }
        
        let mut tt_move = NULL_MOVE;
        if let Some(entry) = self.ttable.get(board.get_hash()) {
             
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
            
            tt_move = entry.best_move;
        }

        let stand_part = board.eval();

        if stand_part >= beta {
            return beta
        }
        if alpha < stand_part {
            alpha = stand_part;
        }

        
        let captures = AdvancedSorting::sort_only_captures(board, tt_move);
        for &m in captures.as_slice() {
            if board.make_pl_move::<true>(m) {
                let score = - self.quiesence_search(board, -beta, -alpha);
                board.unmake_pl_move(m);
                if score >= beta { return beta; }
                if score > alpha {alpha = score }
            }
        } 
        
        
        alpha
    }

    pub fn negamax(&mut self, board: &mut Board, depth: u8) -> Option<SearchResult> {
        self.nodes_searched = 0;
        let ply = 0;
        let mut alpha = NEG_INFINITY;
        let beta = POSITIVE_INFINITY;


        let mut tt_move = NULL_MOVE;
        if let Some(entry) = self.ttable.get(board.get_hash()) {
            if entry.depth >= depth {

                if entry.ntype == Ntype::Exact {
                    return Some(SearchResult {
                        best_move: entry.best_move,
                        evaluation: entry.score,
                        nodes_searched: self.nodes_searched,
                        depth: depth,
                    });
                }

            }
            tt_move = entry.best_move;
        }

        let mut sorter = AdvancedSorting::new(tt_move);
        let mut ntype = Ntype::Upper;
        let mut best_move = NULL_MOVE;

        while let Some(m) = sorter.next(board, &self.killer_table[ply], &self.history_table) {
            if board.make_pl_move::<true>(m) {
                let value = -self.negamax_p(board, depth - 1, -beta, -alpha, ply+1);

                board.unmake_pl_move(m);
                if value > alpha {
                    best_move = m;
                    alpha = value;
                    ntype = Ntype::Exact;
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
                evaluation: alpha,
                nodes_searched: self.nodes_searched,
                depth: depth,
            });
        }
        None
    }

    fn negamax_p(&mut self, board: &mut Board, depth: u8, mut alpha: i16, beta: i16, ply: usize) -> i16 {
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
        let mut sorter = AdvancedSorting::new(tt_move);
        let mut moves_played: MoveList<MOVE_GEN_SIZE> = MoveList::new();
        let current_turn = board.get_turn();
        while let Some(m) = sorter.next(board, &self.killer_table[ply], &self.history_table) {
            if board.make_pl_move::<true>(m) {
                moves_played.push(m);
                let new_eval = -self.negamax_p(board, depth - 1, -beta, -alpha, ply+1);
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
                    self.punish_quiets(&moves_played, depth, current_turn);
                    if m.is_quiet() {
                        self.append_killer_move(ply, m);
                        self.score_history_move(m, (depth*depth) as i16, current_turn);
                    }
                    return beta;
                }
                if new_eval > alpha {
                    alpha = new_eval;
                    best_move = m;
                    ntype = Ntype::Exact
                }
            }
        }

        if moves_played.size() == 0 {
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
