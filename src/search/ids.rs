use crate::chess::board::Board;
use crate::chess::chess_move::NULL_MOVE;
use crate::parameters::FIRST_WINDOW;
use crate::search::simple_search::NegamaxTT;
use crate::search::{GLOBAL_MAX_SEARCH_NODES, MAX_SEARCH_DEPTH, SearchLimits, SearchResult};
use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
pub struct Restrictions {
    pub max_nodes: u64,
    pub end: Instant,
}

impl Restrictions {
    pub const NODES: u8 = 1 << 0;
    pub const TIME: u8 = 1 << 1;
    pub const DEPTH: u8 = 1 << 2;

    pub fn new() -> Self {
        Self {
            max_nodes: GLOBAL_MAX_SEARCH_NODES,
            end: Instant::now(),
        }
    }
}

pub struct IDSearch {
    search_algo: NegamaxTT,
    last_val: i16,
}

impl IDSearch {
    pub fn new(algo: NegamaxTT) -> Self {
        Self {
            search_algo: algo,
            last_val: 0,
        }
    }

    pub fn reset(&mut self) {
        self.last_val = 0;
        self.search_algo.clear();
    }

    pub fn change_hash(&mut self, mb: usize) {
        self.search_algo.change_hash_size(mb);
    }

    pub fn ids<const LIMIT: u8>(
        &mut self,
        board: &mut Board,
        limits: &SearchLimits,
    ) -> Option<SearchResult> {
        debug_assert!(
            if LIMIT == Restrictions::TIME && limits.base_inc.is_none() {
                false
            } else {
                true
            },
            "Time Search without base and inc"
        );
        debug_assert!(
            if LIMIT == Restrictions::NODES && limits.max_nodes.is_none() {
                false
            } else {
                true
            },
            "Node Search without max_nodes"
        );
        debug_assert!(
            if LIMIT == Restrictions::DEPTH && limits.max_depth.is_none() {
                false
            } else {
                true
            },
            "Depth search without depth"
        );
        let mut restrictions = Restrictions::new();

        let start_time = Instant::now();
        let (base, inc) = if LIMIT == Restrictions::TIME {
            limits.base_inc.unwrap()
        } else {
            (0, 0)
        };
        if LIMIT == Restrictions::NODES {
            restrictions.max_nodes = limits.max_nodes.unwrap();
        }
        let soft_bound = base / 20 + inc / 2;
        let hard_bound = (0.3 * base as f32 - 300.0)
            .min(1.5 * soft_bound as f32)
            .max(50.0);
        let end = start_time + Duration::from_millis(hard_bound as u64);
        restrictions.end = end;
        let soft_bound = Duration::from_millis(soft_bound);
        let mut nodes_searched = 0;

        let max_depth = if LIMIT == Restrictions::DEPTH {
            (limits.max_depth.unwrap() + 1).min(MAX_SEARCH_DEPTH as u8)
        } else {
            MAX_SEARCH_DEPTH as u8
        };

        let mut best_res: Option<SearchResult> = None;

        for current_depth in 1..max_depth {
            if LIMIT == Restrictions::TIME && start_time.elapsed() > soft_bound {
                break;
            }

            if let Some(result) = self.search_algo.negamax::<{ LIMIT }>(
                board,
                current_depth as u8,
                restrictions,
                FIRST_WINDOW(),
                FIRST_WINDOW(),
                self.last_val,
            ) {
                nodes_searched += result.nodes_searched;
                restrictions.max_nodes =
                    restrictions.max_nodes.saturating_sub(result.nodes_searched);
                if result.best_move != NULL_MOVE {
                    best_res = Some(result);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        self.last_val = best_res.as_mut().map_or(0, |res| res.evaluation);
        if let Some(best_res) = best_res.as_mut() {
            best_res.nodes_searched = nodes_searched
        }
        best_res
    }

    pub fn search(&mut self, board: &mut Board, limits: &SearchLimits) -> Option<SearchResult> {
        self.search_algo.new_search_turn();

        if limits.max_depth.is_some() {
            return self.ids::<{ Restrictions::DEPTH }>(board, limits);
        } else if limits.max_nodes.is_some() {
            return self.ids::<{ Restrictions::NODES }>(board, limits);
        } else if limits.base_inc.is_some() {
            return self.ids::<{ Restrictions::TIME }>(board, limits);
        }
        None
    }
}
