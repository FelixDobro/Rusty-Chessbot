// use crate::chess::board::Board;
// use crate::chess::chess_move::{Move, NULL_MOVE};
// use crate::move_sorting::advanced_sorting::AdvancedSorting;
// use crate::parameters::FIRST_WINDOW;
// use crate::search::simple_search::{HistroyT, Search, SearchStack};
// use crate::search::{GLOBAL_MAX_SEARCH_NODES, MAX_SEARCH_DEPTH, SearchLimits, SearchResult};
// use std::sync::Arc;
// use std::sync::atomic::AtomicU64;
// use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// pub trait Millis {
//     fn now_millis() -> u64 {
//         SystemTime::now()
//             .duration_since(UNIX_EPOCH)
//             .unwrap_or_default()
//             .as_millis() as u64
//     }
// }

// impl Millis for SystemTime {}

// #[derive(Clone)]
// pub struct Restrictions {
//     pub max_nodes: u64,
//     pub end: Arc<AtomicU64>,
// }

// impl Restrictions {
//     pub const NODES: u8 = 1 << 0;
//     pub const TIME: u8 = 1 << 1;
//     pub const DEPTH: u8 = 1 << 2;

//     pub fn new() -> Self {
//         Self {
//             max_nodes: GLOBAL_MAX_SEARCH_NODES,
//             end: Arc::new(AtomicU64::default()),
//         }
//     }
// }

// pub struct IDSearch {
//     search_algo: Search,
//     last_val: i16,
// }

// impl IDSearch {
//     pub fn new(algo: Search) -> Self {
//         Self {
//             search_algo: algo,
//             last_val: 0,
//         }
//     }

//     pub fn get_history(&self) -> &HistroyT {
//         self.search_algo.get_history()
//     }

//     pub fn get_stack(&self) -> &SearchStack<MAX_SEARCH_DEPTH> {
//         self.search_algo.get_stack()
//     }

//     pub fn get_hash_m(&self, board: &Board) -> Move {
//         self.search_algo.get_hash_move(board)
//     }

//     pub fn reset(&mut self) {
//         self.last_val = 0;
//         self.search_algo.clear();
//     }

//     pub fn change_hash(&mut self, mb: usize) {
//         self.search_algo.change_hash_size(mb);
//     }

//     pub fn ids<const LIMIT: u8>(&mut self, board: &mut Board) -> Option<SearchResult> {
//         let mut best_res: Option<SearchResult> = None;
//         let mut nodes_searched = 0;
//         for current_depth in 1..MAX_SEARCH_DEPTH {
//             if let Some(result) = self.search_algo.negamax::<{ LIMIT }>(
//                 board,
//                 current_depth as u8,
//                 FIRST_WINDOW(),
//                 FIRST_WINDOW(),
//                 self.last_val,
//             ) {
//                 nodes_searched += result.nodes_searched;

//                 if result.best_move != NULL_MOVE {
//                     best_res = Some(result);
//                 } else {
//                     break;
//                 }
//             } else {
//                 break;
//             }
//         }
//         self.last_val = best_res.as_mut().map_or(0, |res| res.evaluation);
//         if let Some(best_res) = best_res.as_mut() {
//             best_res.nodes_searched = nodes_searched
//         }
//         best_res
//     }

//     pub fn ponder_prediction(&self, initial: Move, board: &mut Board) -> Option<Move> {
//         board.make_pl_move::<true>(initial);
//         let hash = self.get_hash_m(board);
//         let prediction =
//             AdvancedSorting::new(hash).next(board, self.get_history(), self.get_stack());
//         board.unmake_pl_move(initial);
//         prediction
//     }
// }
