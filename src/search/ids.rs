use std::thread::current;
use std::time::{Duration, Instant};

use crate::search::simple_search::NegamaxTT;
use crate::search::{MAX_SEARCH_DEPTH, SearchAlgorithm, SearchLimits, SearchResult};
use crate::chess::board::{self, Board};



pub struct IDSearch {
    search_algo: NegamaxTT
}

impl IDSearch {
    pub fn new(algo: NegamaxTT) -> Self {
        Self { search_algo: algo }
    }

    pub fn timed_search(&mut self, board: &mut Board, search_time: Duration) -> Option<SearchResult>{
        let total_time = Instant::now();
        let mut last_nodes = 1;
        self.search_algo.reset_killers();
        let mut best_res: Option<SearchResult> = None;

        for current_depth in 1..MAX_SEARCH_DEPTH {
            let depth_start_time = Instant::now();
            if let Some(result) = self.search_algo.search(board, &SearchLimits::depth(current_depth)) {
                let depth_elapsed = depth_start_time.elapsed();
                let total_elapsed = total_time.elapsed();

                let current_searched_nodes = result.nodes_searched;
                let ebf = current_searched_nodes as f32 / 1.max(last_nodes) as f32; 

                let predicted_time = depth_elapsed.mul_f32(ebf);
                last_nodes = current_searched_nodes;
                best_res = Some(result);

                if total_elapsed + predicted_time > search_time {
                    break;
                }
            }
            else {
                break
            }
        }
        best_res
    }
}

impl SearchAlgorithm for IDSearch {

    fn search(&mut self, board: &mut Board, limits: &SearchLimits) -> Option<SearchResult> {
        if let Some(depth) = limits.max_depth {
            return self.search_algo.search(board, limits);
        }
        else if let Some(time) = limits.max_time {
            return self.timed_search(board, time);
        }
        None
    }
}