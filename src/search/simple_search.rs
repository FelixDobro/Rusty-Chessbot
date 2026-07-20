use std::i16;
use std::time::{Duration, Instant};

use crate::chess::board::Board;
use crate::chess::board::evaluation::{CHECK_MATE_THRESHOLD, NEG_INFINITY, POSITIVE_INFINITY};
use crate::chess::chess_move::{MOVE_GEN_SIZE, Move, MoveList, NULL_MOVE};
use crate::chess::constants::{Color, NUM_PIECES};
use crate::move_sorting::advanced_sorting::{AdvancedSorting, NumericSorting};
use crate::search::Ntype::Exact;
use crate::search::{GLOBAL_MAX_SEARCH_DURATION_H, Ntype, SearchResult, TTable, TTableEntry};

#[allow(dead_code)]
pub struct Negamax {
    nodes_searched: u64,
}

#[allow(dead_code)]
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
            let res = board.result(board.in_check());
            return res;
        }

        alpha
    }
}

struct HistroyT {
    main_history: [[[i16; 64]; 64]; 2],
    continuation_history: [[[i16; 64]; NUM_PIECES * 2]; Self::NUM_PLY_CONTINUATION],
    capture_history: [[[i16; NUM_PIECES - 1]; 64]; NUM_PIECES * 2],
}

impl HistroyT {
    pub const NUM_PLY_CONTINUATION: usize = 6;
    pub const HISTORY_MAX: i16 = 2i16.pow(13);
    pub const HISTORY_MIN: i16 = -Self::HISTORY_MAX;

    pub fn new() -> Self {
        return Self {
            main_history: [[[0i16; 64]; 64]; 2],
            continuation_history: [[[0i16; 64]; NUM_PIECES * 2]; Self::NUM_PLY_CONTINUATION],
            capture_history: [[[0i16; NUM_PIECES - 1]; 64]; NUM_PIECES * 2],
        };
    }

    #[inline(always)]
    pub fn main_history_update(&mut self, m: Move, bonus: i16, turn_idx: usize) {
        let clamped_bonus = Self::HISTORY_MIN.max(bonus).min(Self::HISTORY_MAX);

        self.main_history[turn_idx][m.from().index()][m.to().index()] += clamped_bonus
            - self.main_history[turn_idx][m.from().index()][m.to().index()] * clamped_bonus.abs()
                / Self::HISTORY_MAX;
    }

    pub fn punish_quiets(&mut self, quiets: &MoveList<MOVE_GEN_SIZE>, depth: u8, turn: Color) {
        let malus = (depth * depth) as i16 * -1;
        for m in quiets.as_slice() {
            self.main_history_update(*m, malus, turn.index());
        }
    }
}

struct SearchInfo {
    killer_table: [[Move; 3]; 64],
    history_tables: HistroyT,
    age: u8,
    nodes_searched: u64,
    timed_nodes: u64,
    time_fail: bool,
    start_time: Instant,
    allowed_duration: Duration,
}

impl SearchInfo {
    pub fn new() -> Self {
        SearchInfo {
            age: 0,
            nodes_searched: 0,
            timed_nodes: 0,
            time_fail: false,
            killer_table: [[NULL_MOVE; 3]; 64],
            history_tables: HistroyT::new(),
            start_time: Instant::now(),
            allowed_duration: Duration::from_hours(GLOBAL_MAX_SEARCH_DURATION_H),
        }
    }

    pub fn reset(&mut self, start_time: Instant, allowed_duration: Duration) {
        self.age = 0;
        self.nodes_searched = 0;
        self.timed_nodes = 0;
        self.time_fail = false;
        self.start_time = start_time;
        self.allowed_duration = allowed_duration
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
    pub fn increment_nodes(&mut self) {
        self.timed_nodes += 1;
        self.nodes_searched += 1
    }

    #[inline(always)]
    pub fn increase_age(&mut self) {
        self.age = self.age.wrapping_add(1);
    }

    #[inline(always)]
    pub fn time_failed(&mut self) {
        self.time_fail = true
    }

    #[inline(always)]
    pub fn reset_timed_nodes(&mut self) {
        self.timed_nodes = 0
    }

    #[inline(always)]
    pub fn punish_quiets(&mut self, quiets: &MoveList<MOVE_GEN_SIZE>, depth: u8, turn: Color) {
        self.history_tables.punish_quiets(quiets, depth, turn);
    }

    #[inline(always)]
    pub fn reward_quiet(&mut self, m: Move, depth: u8, turn: Color) {
        self.history_tables
            .main_history_update(m, (depth * depth) as i16, turn.index());
    }
}

pub struct NegamaxTT {
    ttable: TTable,
    info: SearchInfo,
}

impl NegamaxTT {
    const CHECK_TIME_NODES: u64 = 100_000;
    const RFP_BASE: i16 = 60;
    const RFP_LIN: i16 = 20;
    const RFP_QUAD: i16 = 10;

    pub fn new(ttsize: usize) -> Self {
        Self {
            ttable: TTable::new(ttsize),
            info: SearchInfo::new(),
        }
    }

    #[inline(always)]
    pub fn get_new_window(window: i16) -> i16 {
        match window {
            25 => 100,
            _ => POSITIVE_INFINITY,
        }
    }

    #[inline(always)]
    pub fn reset_killers(&mut self) {
        self.info.reset_killers();
    }

    #[inline(always)]
    pub fn get_hash_move(&self, board: &Board) -> Move {
        self.ttable
            .get(board.get_hash())
            .map_or(NULL_MOVE, |entry| entry.best_move)
    }

    #[inline(always)]
    fn table_value(
        entry: &TTableEntry,
        alpha: i16,
        beta: i16,
        depth: u8,
        is_pv: bool,
    ) -> Option<i16> {
        if !is_pv && entry.depth >= depth {
            return match entry.ntype {
                Ntype::Exact => Some(entry.score),
                Ntype::Lower if beta <= entry.score => Some(entry.score),
                Ntype::Upper if alpha >= entry.score => Some(entry.score),
                _ => None,
            };
        }
        None
    }

    #[inline(always)]
    fn nullmove_depth_possible(depth: u8) -> bool {
        if depth <= depth / 3 + 3 {
            return false;
        }
        true
    }

    #[inline(always)]
    fn rfp_margin(depth: u8) -> i16 {
        let depth_i16 = depth as i16;
        Self::RFP_BASE + depth_i16 * Self::RFP_LIN + depth_i16 * depth_i16 * Self::RFP_QUAD
    }

    pub fn quiesence_search(&mut self, board: &mut Board, mut alpha: i16, beta: i16) -> i16 {
        self.info.increment_nodes();
        if board.position_will_draw() {
            return 0;
        }

        let mut tt_move = NULL_MOVE;
        if let Some(entry) = self.ttable.get(board.get_hash()) {
            if let Some(val) = Self::table_value(entry, alpha, beta, 0, false) {
                return val;
            }

            tt_move = entry.best_move;
        }

        let stand_part = board.eval();

        if stand_part >= beta {
            return beta;
        }
        if alpha < stand_part {
            alpha = stand_part;
        }

        let captures = AdvancedSorting::sort_only_captures(board, tt_move);
        for &m in captures.as_slice() {
            if board.make_pl_move::<true>(m) {
                let score = -self.quiesence_search(board, -beta, -alpha);
                board.unmake_pl_move(m);
                if score >= beta {
                    return beta;
                }
                if score > alpha {
                    alpha = score
                }
            }
        }

        alpha
    }

    pub fn negamax(
        &mut self,
        board: &mut Board,
        depth: u8,
        start_time: Instant,
        allowed_duration: Duration,
        alpha_dec: i16,
        beta_inc: i16,
        last_val: i16,
    ) -> Option<SearchResult> {
        self.info.reset(start_time, allowed_duration);
        let ply = 0;

        let original_alpha = last_val.saturating_sub(alpha_dec).max(NEG_INFINITY);
        let original_beta = last_val.saturating_add(beta_inc).min(POSITIVE_INFINITY);
        let mut alpha = original_alpha;
        let beta = original_beta;

        let tt_move = self.get_hash_move(board);

        let mut sorter = AdvancedSorting::new(tt_move);
        let mut ntype = Ntype::Upper;
        let mut best_move = NULL_MOVE;
        let mut any_move = NULL_MOVE;
        let mut first_move = true;

        while let Some(m) = sorter.next(
            board,
            &self.info.killer_table[ply],
            &self.info.history_tables.main_history,
        ) {
            if board.make_pl_move::<true>(m) {
                let mut value;
                if first_move {
                    value = -self.negamax_p(board, depth - 1, -beta, -alpha, ply + 1, false);
                    first_move = false;
                } else {
                    value = -self.negamax_p(board, depth - 1, -alpha - 1, -alpha, ply + 1, false);

                    if value > alpha && value < beta {
                        value = -self.negamax_p(board, depth - 1, -beta, -value, ply + 1, false);
                    }
                }

                board.unmake_pl_move(m);

                if self.info.time_fail {
                    return None;
                }

                if value > alpha {
                    best_move = m;
                    alpha = value;
                    ntype = Ntype::Exact;
                }
                any_move = m;
                if alpha >= beta {
                    break;
                }
            }
        }

        /*
        Check whether the predicted bounds were accurate bounds for the search
        */

        if alpha <= original_alpha && alpha_dec < POSITIVE_INFINITY {
            return self.negamax(
                board,
                depth,
                start_time,
                allowed_duration,
                Self::get_new_window(alpha_dec),
                beta_inc,
                last_val,
            );
        }
        if alpha >= original_beta && beta_inc < POSITIVE_INFINITY {
            return self.negamax(
                board,
                depth,
                start_time,
                allowed_duration,
                alpha_dec,
                Self::get_new_window(beta_inc),
                last_val,
            );
        }

        if self.info.nodes_searched == 0 {
            let res = board.result(board.in_check());
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
                age: self.info.age,
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
            age: self.info.age,
        });

        self.info.increase_age();

        if best_move != NULL_MOVE {
            return Some(SearchResult {
                best_move: best_move,
                evaluation: alpha,
                nodes_searched: self.info.nodes_searched,
                depth: depth,
            });
        } else if any_move != NULL_MOVE {
            return Some(SearchResult {
                best_move: any_move,
                evaluation: alpha,
                nodes_searched: self.info.nodes_searched,
                depth: depth,
            });
        }
        None
    }

    fn negamax_p(
        &mut self,
        board: &mut Board,
        depth: u8,
        mut alpha: i16,
        beta: i16,
        ply: usize,
        made_nullmove: bool,
    ) -> i16 {
        if self.info.timed_nodes > Self::CHECK_TIME_NODES {
            if self.info.start_time.elapsed() > self.info.allowed_duration {
                self.info.time_failed();
                return 0;
            }
            self.info.reset_timed_nodes();
        }
        if board.position_will_draw() {
            return 0;
        }

        let is_pv = beta.saturating_sub(alpha) > 1;

        self.info.increment_nodes();
        let mut tt_move = NULL_MOVE;
        if let Some(entry) = self.ttable.get(board.get_hash()) {
            if let Some(val) = Self::table_value(entry, alpha, beta, depth, is_pv) {
                return val;
            }
            tt_move = entry.best_move;
        }

        if depth == 0 {
            self.info.nodes_searched -= 1;
            return self.quiesence_search(board, alpha, beta);
        }

        let in_check = board.in_check();
        let has_heavy_material = !board.only_king_and_pawns();

        if has_heavy_material && !in_check {
            if depth < 7 {
                let s_eval = board.eval();
                let margin = Self::rfp_margin(depth);

                if beta < CHECK_MATE_THRESHOLD && s_eval >= beta + margin {
                    return s_eval;
                }
            }
            if !made_nullmove && Self::nullmove_depth_possible(depth) {
                let reduction = 3 + depth / 3;
                board.make_nullmove();
                let val =
                    -self.negamax_p(board, depth - reduction, -beta, -beta + 1, ply + 1, true);
                board.unmake_nullmove();
                if val >= beta {
                    return val;
                }
            }
        }

        let mut ntype = Ntype::Upper;
        let mut best_move = NULL_MOVE;
        let mut sorter = AdvancedSorting::new(tt_move);
        let current_turn = board.get_turn();
        let mut num_moves_played = 0;
        let mut first_move = true;
        let mut quiets_played: MoveList<MOVE_GEN_SIZE> = MoveList::new();
        while let Some(m) = sorter.next(
            board,
            &self.info.killer_table[ply],
            &self.info.history_tables.main_history,
        ) {
            debug_assert_ne!(m, NULL_MOVE);
            if board.make_pl_move::<true>(m) {
                if m.is_quiet() {
                    quiets_played.push(m);
                }
                num_moves_played += 1;
                let mut value;
                if first_move {
                    value = -self.negamax_p(board, depth - 1, -beta, -alpha, ply + 1, false);
                    first_move = false;
                } else {
                    value = -self.negamax_p(board, depth - 1, -alpha - 1, -alpha, ply + 1, false);

                    if value > alpha && value < beta {
                        value = -self.negamax_p(board, depth - 1, -beta, -value, ply + 1, false);
                    }
                }

                board.unmake_pl_move(m);

                if self.info.time_fail {
                    return 0;
                }

                if value >= beta {
                    self.ttable.insert(TTableEntry {
                        hash: board.get_hash(),
                        best_move: m,
                        score: beta,
                        depth: depth,
                        ntype: Ntype::Lower,
                        age: self.info.age,
                    });

                    if m.is_quiet() {
                        quiets_played.pop();
                        self.info.append_killer_move(ply, m);
                        self.info.punish_quiets(&quiets_played, depth, current_turn);
                        self.info.reward_quiet(m, depth, current_turn);
                    }
                    return beta;
                }
                if value > alpha {
                    alpha = value;
                    best_move = m;
                    ntype = Ntype::Exact
                }
            }
        }

        if num_moves_played == 0 {
            let res = board.result(in_check);
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
                age: self.info.age,
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
            age: self.info.age,
        });
        alpha
    }
}
