use crate::chess::board::Board;
use crate::chess::board::evaluation::{NEG_INFINITY, POSITIVE_INFINITY};
use crate::search::simple_search::NegamaxTT;
use crate::search::{GLOBAL_MAX_SEARCH_DURATION_H, MAX_SEARCH_DEPTH, SearchLimits, SearchResult};
use std::time::{Duration, Instant};

pub struct IDSearch {
    search_algo: NegamaxTT,
}

impl IDSearch {
    pub fn new(algo: NegamaxTT) -> Self {
        Self { search_algo: algo }
    }

    pub fn timed_search(&mut self, board: &mut Board, base: u64, inc: u64) -> Option<SearchResult> {
        let start_time = Instant::now();

        let soft_bound = base / 20 + inc / 2;
        let hard_bound = (0.3 * base as f32 - 300.0)
            .max(50.0)
            .min(1.5 * soft_bound as f32);

        let hard_duration = Duration::from_millis(hard_bound as u64);
        let soft_bound = Duration::from_millis(soft_bound);

        let mut best_res: Option<SearchResult> = None;

        for current_depth in 1..MAX_SEARCH_DEPTH {
            if current_depth > 1 && start_time.elapsed() > soft_bound {
                break;
            }

            let last_val = best_res.as_ref().map_or(0, |res| res.evaluation);

            if let Some(result) = self.search_algo.negamax(
                board,
                current_depth as u8,
                start_time,
                hard_duration,
                25,
                25,
                last_val,
            ) {
                best_res = Some(result)
            } else {
                break;
            }
        }
        best_res
    }

    pub fn depth_search(&mut self, board: &mut Board, depth: u8) -> Option<SearchResult> {
        self.search_algo.negamax(
            board,
            depth,
            Instant::now(),
            Duration::from_hours(GLOBAL_MAX_SEARCH_DURATION_H),
            POSITIVE_INFINITY,
            NEG_INFINITY,
            0,
        )
    }

    pub fn search(&mut self, board: &mut Board, limits: &SearchLimits) -> Option<SearchResult> {
        if let Some(depth) = limits.max_depth {
            return self.depth_search(board, depth);
        } else if let Some((base, inc)) = limits.base_inc {
            return self.timed_search(board, base, inc);
        }
        None
    }
}
