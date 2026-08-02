use std::i16;
use std::time::{Duration, Instant};

use crate::chess::board::Board;
use crate::chess::board::evaluation::{CHECK_MATE_THRESHOLD, NEG_INFINITY, POSITIVE_INFINITY};
use crate::chess::chess_move::{MOVE_GEN_SIZE, Move, MoveList, NULL_MOVE};
use crate::chess::constants::{Color, NUM_PIECES};
use crate::move_sorting::advanced_sorting::{AdvancedSorting, NumericSorting};
use crate::search::Ntype::Exact;
use crate::search::{
    GLOBAL_MAX_SEARCH_DURATION_H, MAX_SEARCH_DEPTH, Ntype, SearchResult, TTable, TTableEntry,
};

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

#[derive(Debug, Clone, Copy)]
pub struct StackItem {
    m: Move,
    moved_piece: usize,
}

impl StackItem {
    pub fn new(m: Move, moved_piece: usize) -> Self {
        debug_assert!(moved_piece < 12, "Moved piece is invalid");
        return Self { m, moved_piece };
    }
}

pub struct SearchStack<const N: usize> {
    count: usize,
    moves: [StackItem; N],
}

impl<const N: usize> SearchStack<N> {
    pub fn new() -> Self {
        SearchStack {
            count: 0,
            moves: [StackItem::new(NULL_MOVE, 0); N],
        }
    }

    #[inline(always)]
    pub fn push(&mut self, item: StackItem) {
        self.moves[self.count] = item;
        self.count += 1;
    }

    #[inline(always)]
    pub fn pop(&mut self) {
        self.count -= 1;
    }

    #[inline(always)]
    pub fn as_slice<const MAX_PLY: usize>(&self) -> &[StackItem] {
        let start = self.count.saturating_sub(MAX_PLY);

        &self.moves[start..self.count]
    }
}

type ContinuationHistory = [[[[i16; 64]; NUM_PIECES * 2]; 64]; NUM_PIECES * 2];
type CaptureHistory = [[[i16; NUM_PIECES]; 64]; NUM_PIECES * 2];
pub struct HistroyT {
    main_history: Box<[[[i16; 64]; 64]; 2]>,
    continuation_history: Box<[ContinuationHistory; Self::NUM_PLY_CONTINUATION]>,
    capture_history: Box<CaptureHistory>,
}

impl HistroyT {
    pub const NUM_PLY_CONTINUATION: usize = 6;
    pub const HISTORY_MAX: i32 = 2i32.pow(13);
    pub const HISTORY_MIN: i32 = -Self::HISTORY_MAX;

    pub fn new() -> Self {
        return Self {
            main_history: Box::new([[[0i16; 64]; 64]; 2]),
            continuation_history: unsafe { Box::new_zeroed().assume_init() },
            capture_history: Box::new([[[0i16; NUM_PIECES]; 64]; NUM_PIECES * 2]),
        };
    }

    #[inline(always)]
    pub fn main_history_update(&mut self, m: Move, bonus: i32, turn_idx: usize) {
        let prev_val = self.main_history[turn_idx][m.from().index()][m.to().index()] as i32;
        self.main_history[turn_idx][m.from().index()][m.to().index()] +=
            (bonus - prev_val * bonus.abs() / Self::HISTORY_MAX) as i16;
    }

    #[inline(always)]
    pub fn continuation_history_update(
        &mut self,
        prev_m: Move,
        prev_moved_piece: usize,
        cur_m: Move,
        cur_moved_piece: usize,
        ply: usize,
        bonus: i32,
    ) {
        let prev_val = self.continuation_history[ply][prev_moved_piece][prev_m.to().index()]
            [cur_moved_piece][cur_m.to().index()] as i32;
        self.continuation_history[ply][prev_moved_piece][prev_m.to().index()][cur_moved_piece]
            [cur_m.to().index()] += (bonus - prev_val * bonus.abs() / Self::HISTORY_MAX) as i16;
    }

    pub fn capture_history_update(
        &mut self,
        moved_piece: usize,
        captured_piece: usize,
        m: Move,
        bonus: i32,
    ) {
        let prev_val = self.capture_history[moved_piece][m.to().index()][captured_piece] as i32;
        self.capture_history[moved_piece][m.to().index()][captured_piece] +=
            (bonus - prev_val * bonus.abs() / Self::HISTORY_MAX) as i16;
    }

    pub fn reward_capture(&mut self, board: &Board, m: Move, depth: u8) {
        let bonus = (depth * depth) as i32;
        let clamped_bonus = Self::HISTORY_MIN.max(bonus).min(Self::HISTORY_MAX);
        let moved_piece = board.get_piece_w_color(m.from());
        let captured_piece = board.get_captured(m);
        self.capture_history_update(moved_piece, captured_piece, m, clamped_bonus);
    }

    pub fn reward_continuation(
        &mut self,
        search_stack: &SearchStack<MAX_SEARCH_DEPTH>,
        depth: u8,
        m: Move,
        piece: usize,
    ) {
        let bonus = (depth * depth) as i32;
        let clamped_bonus = Self::HISTORY_MIN.max(bonus).min(Self::HISTORY_MAX);
        search_stack
            .as_slice::<{ Self::NUM_PLY_CONTINUATION }>()
            .iter()
            .rev()
            .enumerate()
            .for_each(|(ply, item)| {
                self.continuation_history_update(
                    item.m,
                    item.moved_piece,
                    m,
                    piece,
                    ply,
                    clamped_bonus,
                );
            });
    }

    pub fn punish_continuation(
        &mut self,
        quiets: &MoveList<MOVE_GEN_SIZE>,
        search_stack: &SearchStack<MAX_SEARCH_DEPTH>,
        depth: u8,
        board: &Board,
    ) {
        let malus = (depth * depth) as i32 * -1;
        let clamped_bonus = Self::HISTORY_MIN.max(malus).min(Self::HISTORY_MAX);

        search_stack
            .as_slice::<{ Self::NUM_PLY_CONTINUATION }>()
            .iter()
            .rev()
            .enumerate()
            .for_each(|(ply, item)| {
                for m in quiets.as_slice() {
                    self.continuation_history_update(
                        item.m,
                        item.moved_piece,
                        *m,
                        board.get_piece_w_color(m.from()),
                        ply,
                        clamped_bonus,
                    )
                }
            });
    }

    pub fn punish_captures(
        &mut self,
        captures: &MoveList<MOVE_GEN_SIZE>,
        board: &Board,
        depth: u8,
    ) {
        let malus = (depth * depth) as i32 * -1;
        let clamped_bonus = Self::HISTORY_MIN.max(malus).min(Self::HISTORY_MAX);

        captures.as_slice().iter().for_each(|&m| {
            debug_assert!(m.is_capture(), "Move should be capture");
            let moved_piece = board.get_piece_w_color(m.from());
            let captured_piece = board.get_captured(m);
            self.capture_history_update(moved_piece, captured_piece, m, clamped_bonus);
        });
    }

    pub fn punish_main(&mut self, quiets: &MoveList<MOVE_GEN_SIZE>, depth: u8, turn: Color) {
        let malus = (depth * depth) as i32 * -1;
        let clamped_bonus = Self::HISTORY_MIN.max(malus).min(Self::HISTORY_MAX);
        for m in quiets.as_slice() {
            self.main_history_update(*m, clamped_bonus, turn.index());
        }
    }

    #[inline(always)]
    pub fn continuation_val(
        &self,
        search_stack: &SearchStack<MAX_SEARCH_DEPTH>,
        moved_piece: usize,
        m: Move,
    ) -> i32 {
        search_stack
            .as_slice::<{ Self::NUM_PLY_CONTINUATION }>()
            .iter()
            .rev()
            .enumerate()
            .map(|(ply, item)| {
                self.continuation_history[ply][item.moved_piece][item.m.to().index()][moved_piece]
                    [m.to().index()] as i32
            })
            .sum()
    }

    #[inline(always)]
    pub fn main_val(&self, turn: Color, m: Move) -> i32 {
        self.main_history[turn.index()][m.from().index()][m.to().index()] as i32
    }

    #[inline(always)]
    pub fn capture_val(&self, moved_piece: usize, m: Move, captured_piece: usize) -> i32 {
        self.capture_history[moved_piece][m.to().index()][captured_piece] as i32
    }
}

struct SearchInfo {
    search_stack: SearchStack<MAX_SEARCH_DEPTH>,
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
            search_stack: SearchStack::new(),
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
        self.allowed_duration = allowed_duration;
        self.search_stack = SearchStack::new();
    }

    pub fn push(&mut self, item: StackItem) {
        self.search_stack.push(item);
    }
    pub fn pop(&mut self) {
        self.search_stack.pop();
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
    pub fn punish_main(&mut self, quiets: &MoveList<MOVE_GEN_SIZE>, depth: u8, turn: Color) {
        self.history_tables.punish_main(quiets, depth, turn);
    }

    #[inline(always)]
    pub fn reward_main(&mut self, m: Move, depth: u8, turn: Color) {
        let bonus = (depth * depth) as i32;
        let clamped_bonus = HistroyT::HISTORY_MIN.max(bonus).min(HistroyT::HISTORY_MAX);
        self.history_tables
            .main_history_update(m, clamped_bonus, turn.index());
    }

    #[inline(always)]
    pub fn reward_capture(&mut self, m: Move, board: &Board, depth: u8) {
        self.history_tables.reward_capture(board, m, depth);
    }

    #[inline(always)]
    pub fn punish_captures(
        &mut self,
        captures: &MoveList<MOVE_GEN_SIZE>,
        board: &Board,
        depth: u8,
    ) {
        self.history_tables.punish_captures(captures, board, depth);
    }

    #[inline(always)]
    pub fn punish_continuation(
        &mut self,
        quiets: &MoveList<MOVE_GEN_SIZE>,
        board: &Board,
        depth: u8,
    ) {
        self.history_tables
            .punish_continuation(quiets, &self.search_stack, depth, board);
    }

    pub fn reward_continuation(&mut self, m: Move, depth: u8, piece: usize) {
        self.history_tables
            .reward_continuation(&self.search_stack, depth, m, piece);
    }
}

pub struct NegamaxTT {
    ttable: TTable,
    info: SearchInfo,
    LMR: Box<[[u8; MOVE_GEN_SIZE]; MAX_SEARCH_DEPTH]>,
}

impl NegamaxTT {
    const CHECK_TIME_NODES: u64 = 200_000;

    // Reverse Futility Pruning hyperparams
    const RFP_BASE: i16 = 60;
    const RFP_LIN: i16 = 30;
    const RFP_QUAD: i16 = 10;

    // Prob Cut hyperparams
    const PROB_MIN_DEPTH: u8 = 5;
    const PROB_DEPTH_REDUCTION: u8 = 4;
    const PROB_SLOPE: i16 = 25;
    const PROB_BIAS: i16 = 120;

    // Razoring hyperparams
    const RAZORING_MAX_DEPTH: u8 = 3;
    const RAZORING_SLOPE: i16 = 100;
    const RAZORING_BIAS: i16 = 300;

    const LMR_MIN_DEPTH: u8 = 3;

    pub fn new(ttsize: usize) -> Self {
        let reductions: Box<[[u8; MOVE_GEN_SIZE]; MAX_SEARCH_DEPTH]> = Box::new({
            let mut table = [[0u8; MOVE_GEN_SIZE]; MAX_SEARCH_DEPTH];
            let mut d = 0;
            while d < MAX_SEARCH_DEPTH {
                let mut m = 0;
                while m < MOVE_GEN_SIZE {
                    let d_f = d as f64;
                    let m_f = m as f64;

                    table[d][m] = (1.0 + (d_f.ln() * m_f.ln()) / 2.25) as u8;
                    m += 1;
                }
                d += 1;
            }
            table
        });

        Self {
            ttable: TTable::new(ttsize),
            info: SearchInfo::new(),
            LMR: reductions,
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
    pub fn get_LMR_reduction(&self, depth: u8, move_count: usize) -> u8 {
        self.LMR[depth as usize][move_count]
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

        if tt_move != NULL_MOVE && tt_move.is_capture() {
            if board.make_pl_move::<true>(tt_move) {
                let score = -self.quiesence_search(board, -beta, -alpha);
                board.unmake_pl_move(tt_move);
                if score >= beta {
                    return beta;
                }
                if score > alpha {
                    alpha = score;
                }
            }
        }

        let captures =
            AdvancedSorting::sort_only_captures(board, tt_move, &self.info.history_tables);
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
        let mut quiets_played: MoveList<MOVE_GEN_SIZE> = MoveList::new();
        let mut captures_played: MoveList<MOVE_GEN_SIZE> = MoveList::new();
        let current_turn = board.get_turn();

        while let Some(m) = sorter.next(board, &self.info.history_tables, &self.info.search_stack) {
            let moved_piece = board.get_piece_w_color(m.from());

            let stack_item = StackItem::new(m, moved_piece);
            if board.make_pl_move::<true>(m) {
                self.info.push(stack_item);

                if m.is_quiet() {
                    quiets_played.push(m);
                } else if m.is_capture() {
                    captures_played.push(m);
                }

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
                self.info.pop();

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
                    if m.is_quiet() {
                        quiets_played.pop();
                        self.info.punish_main(&quiets_played, depth, current_turn);
                        self.info.reward_main(m, depth, current_turn);
                        self.info.punish_continuation(&quiets_played, board, depth);
                        self.info.reward_continuation(m, depth, moved_piece);
                    } else if m.is_capture() {
                        captures_played.pop();
                        self.info.reward_capture(m, board, depth);
                        self.info.punish_captures(&captures_played, board, depth);
                    }
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
        let static_eval = board.eval();
        if has_heavy_material && !in_check {
            if depth < 7 {
                let margin = Self::rfp_margin(depth);

                if beta < CHECK_MATE_THRESHOLD && static_eval >= beta + margin {
                    return static_eval;
                }
            }
            if !is_pv && !made_nullmove && Self::nullmove_depth_possible(depth) {
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

        // Prob Cut && Razoring
        if !in_check && !is_pv {
            if depth >= Self::PROB_MIN_DEPTH {
                let margin =
                    Self::PROB_BIAS.saturating_add(Self::PROB_SLOPE.saturating_mul(depth as i16));
                let prob_cut_beta = beta.saturating_add(margin);
                let prob_cut_alpha = prob_cut_beta - 1;

                let shallow_eval = self.negamax_p(
                    board,
                    depth - Self::PROB_DEPTH_REDUCTION,
                    prob_cut_alpha,
                    prob_cut_beta,
                    ply,
                    made_nullmove,
                );

                if shallow_eval >= prob_cut_beta {
                    return beta;
                }
            }

            if depth <= Self::RAZORING_MAX_DEPTH {
                let razor_margin = Self::RAZORING_BIAS + Self::RAZORING_SLOPE * (depth as i16);

                if static_eval.saturating_add(razor_margin) < alpha {
                    let q_eval = self.quiesence_search(board, alpha - 1, alpha);

                    if q_eval < alpha {
                        return q_eval;
                    }
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
        let mut captures_played: MoveList<MOVE_GEN_SIZE> = MoveList::new();
        let depth_LMR = depth >= Self::LMR_MIN_DEPTH;

        while let Some(m) = sorter.next(board, &self.info.history_tables, &self.info.search_stack) {
            debug_assert_ne!(m, NULL_MOVE);

            let moved_piece = board.get_piece_w_color(m.from());
            let stack_item = StackItem::new(m, moved_piece);

            if board.make_pl_move::<true>(m) {
                let is_quiet = m.is_quiet();

                self.info.push(stack_item);
                if is_quiet {
                    quiets_played.push(m);
                }
                if m.is_capture() {
                    captures_played.push(m);
                }

                num_moves_played += 1;

                let mut needs_full_search = true;
                let mut value = 0;

                // Late Move Reduction
                if !is_pv
                    && num_moves_played > 5
                    && is_quiet
                    && depth_LMR
                    && !in_check
                    && !board.in_check()
                {
                    let mut reduction = self.get_LMR_reduction(depth, num_moves_played);
                    let history_score = self.info.history_tables.main_val(current_turn, m);

                    if history_score > HistroyT::HISTORY_MAX / 2 {
                        reduction = reduction.saturating_sub(1);
                    } else if history_score < HistroyT::HISTORY_MIN / 2 {
                        reduction += 1;
                    }

                    let reduced_depth = depth.saturating_sub(1 + reduction);

                    value =
                        -self.negamax_p(board, reduced_depth, -alpha - 1, -alpha, ply + 1, false);

                    if value <= alpha {
                        needs_full_search = false
                    }
                }
                if needs_full_search {
                    if first_move {
                        value = -self.negamax_p(board, depth - 1, -beta, -alpha, ply + 1, false);
                        first_move = false;
                    } else {
                        value =
                            -self.negamax_p(board, depth - 1, -alpha - 1, -alpha, ply + 1, false);

                        if value > alpha && value < beta {
                            value =
                                -self.negamax_p(board, depth - 1, -beta, -value, ply + 1, false);
                        }
                    }
                }

                board.unmake_pl_move(m);
                self.info.pop();

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
                        self.info.punish_main(&quiets_played, depth, current_turn);
                        self.info.reward_main(m, depth, current_turn);
                        self.info.punish_continuation(&quiets_played, board, depth);
                        self.info.reward_continuation(m, depth, moved_piece);
                    }
                    if m.is_capture() {
                        captures_played.pop();
                        self.info.punish_captures(&captures_played, board, depth);
                        self.info.reward_capture(m, board, depth);
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
