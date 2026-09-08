pub mod ids;
pub mod simple_search;

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use crate::chess::chess_move::Move;
use crate::chess::chess_move::NULL_MOVE;

pub const MAX_SEARCH_DEPTH: usize = 64;
pub const GLOBAL_MAX_SEARCH_DURATION: u64 = 100000000u64;
pub const GLOBAL_MAX_SEARCH_NODES: u64 = 100_000_000_000u64;

pub trait Millis {
    fn now_millis() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    }
}

impl Millis for SystemTime {}

#[cfg(test)]
use crate::search::Ntype::Exact;

#[derive(Default, Clone, Debug)]
#[allow(dead_code)]
pub struct SearchLimits {
    pub max_depth: Option<u8>,
    pub base_inc: Option<(u64, u64)>,
    pub max_nodes: Option<u64>,
    pub infinite: bool,
}

impl SearchLimits {
    pub fn depth(d: u8) -> Self {
        Self {
            max_depth: Some(d),
            ..Default::default()
        }
    }
    pub fn nodes(nodes: u64) -> Self {
        Self {
            max_nodes: Some(nodes),
            ..Default::default()
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SearchResult {
    pub best_move: Move,
    pub evaluation: i16,
    pub nodes_searched: u64,
    pub depth: u8,
}

impl SearchResult {
    pub fn print_info(&self) {
        println!(
            "info depth {} score cp {} nodes {} pv {}",
            self.depth,
            self.evaluation,
            self.nodes_searched,
            self.best_move.to_string(),
        );
    }
}

#[derive(Clone, Default)]
pub struct Restrictions {
    pub max_nodes: u64,
    pub soft_bound: u128,
    pub hard_bound: u128,
    pub depth: u8,
}

impl Restrictions {
    pub const NODES: u8 = 1 << 0;
    pub const TIME: u8 = 1 << 1;
    pub const DEPTH: u8 = 1 << 2;

    pub fn new() -> Self {
        Self {
            max_nodes: GLOBAL_MAX_SEARCH_NODES,
            soft_bound: 0,
            hard_bound: 0,
            depth: 0,
        }
    }

    pub fn depth(d: u8) -> Self {
        Self {
            depth: d,
            ..Restrictions::default()
        }
    }

    pub fn time(soft_bound: u128, hard_bound: u128) -> Self {
        Self {
            soft_bound: soft_bound,
            hard_bound: hard_bound,
            ..Restrictions::default()
        }
    }

    pub fn nodes(nodes: u64) -> Self {
        Self {
            max_nodes: nodes,
            ..Restrictions::default()
        }
    }
}

#[derive(Clone)]
pub struct SearchControl {
    stop_search: Arc<AtomicBool>,
    ponder: Arc<AtomicBool>,
}

impl SearchControl {
    pub fn new() -> Self {
        Self {
            stop_search: Arc::new(AtomicBool::new(false)),
            ponder: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn reset(&mut self) {
        self.stop_pondering();
        self.allow_search();
    }

    pub fn allow_search(&mut self) {
        self.stop_search.store(false, Ordering::Relaxed);
        self.stop_pondering();
    }

    pub fn stop_search(&mut self) {
        self.stop_search.store(true, Ordering::Relaxed);
        self.stop_pondering();
    }

    pub fn stop_pondering(&mut self) {
        self.ponder.store(false, Ordering::Relaxed);
    }

    pub fn start_pondering(&mut self) {
        self.ponder.store(true, Ordering::Relaxed);
    }
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Ntype {
    Exact,
    Lower,
    Upper,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct TTableEntry {
    pub hash: u64,
    pub best_move: Move, //u16
    pub depth: u8,
    pub score: i16,
    pub ntype: Ntype, //u8
    pub age: u8,
}

impl TTableEntry {
    pub const fn empty() -> Self {
        TTableEntry {
            hash: 0,
            best_move: NULL_MOVE,
            depth: 0,
            score: 0,
            ntype: Ntype::Exact,
            age: 0,
        }
    }

    #[cfg(test)]
    fn debug_entry(hash: u64, depth: u8, age: u8) -> Self {
        TTableEntry {
            hash,
            best_move: NULL_MOVE,
            depth: depth,
            score: 0,
            ntype: Exact,
            age,
        }
    }
}

#[repr(align(64))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Bucket {
    entries: [TTableEntry; 4],
}

impl Bucket {
    pub const fn empty() -> Self {
        Bucket {
            entries: [TTableEntry::empty(); 4],
        }
    }

    #[inline(always)]
    pub fn insert(&mut self, new_entry: TTableEntry) {
        let mut replace_index = 0;
        let mut lowest_depth = new_entry.depth;
        let mut lowest_outdated_depth = 255;
        let mut should_replace = false;

        for i in 0..4 {
            let existing_entry = &self.entries[i];

            if existing_entry.hash == new_entry.hash {
                if new_entry.depth >= existing_entry.depth {
                    self.entries[i] = new_entry;
                }
                return;
            }

            if existing_entry.age != new_entry.age {
                if existing_entry.depth < lowest_outdated_depth {
                    lowest_outdated_depth = existing_entry.depth;
                    should_replace = true;
                    replace_index = i;
                }
            } else if existing_entry.depth < lowest_depth {
                should_replace = true;
                lowest_depth = existing_entry.depth;
                replace_index = i;
            }
        }
        if should_replace {
            self.entries[replace_index] = new_entry
        }
    }

    #[inline(always)]
    pub fn get(&self, hash: u64) -> Option<&TTableEntry> {
        for entry in &self.entries {
            if entry.hash == hash {
                return Some(entry);
            }
        }
        None
    }

    #[cfg(test)]
    fn debug_print(&self) {
        for i in 0..4 {
            println!("Entry {}", i);
            let entry = self.entries[i];
            println!("Age: {}", entry.hash);
            println!("Depth: {}", entry.depth);
            println!("Age: {}", entry.age);
            println!();
        }
    }
}

#[repr(align(64))]
pub struct TTable {
    table: Box<[Bucket]>,
    size: usize,
}

impl TTable {
    pub fn prev_power_two(val: usize) -> usize {
        if val == 0 {
            return 0;
        }
        1 << 64 - 1 - val.leading_zeros() as usize
    }

    pub fn num_buckets(mb: usize) -> usize {
        const BUCKET_SIZE: usize = 64;
        let bytes = mb * 1024 * 1024;
        Self::prev_power_two(bytes / BUCKET_SIZE)
    }

    pub fn new(size: usize) -> Self {
        let num_buckets = Self::num_buckets(size);
        TTable {
            table: vec![Bucket::empty(); num_buckets].into_boxed_slice(),
            size: num_buckets,
        }
    }

    pub fn clear(&mut self) {
        self.table = vec![Bucket::empty(); self.size].into_boxed_slice();
    }

    pub fn insert(&mut self, entry: TTableEntry) {
        let index = entry.hash as usize % self.size;
        self.table[index].insert(entry);
    }

    pub fn get(&self, hash: u64) -> Option<&TTableEntry> {
        let index = hash as usize % self.size;
        self.table[index].get(hash)
    }

    #[cfg(test)]
    pub fn get_bucket(&self, hash: u64) -> Bucket {
        let index = hash as usize % self.size;
        self.table[index]
    }
}

#[cfg(test)]
mod test {

    use crate::{
        chess::board::Board,
        search::{TTable, TTableEntry},
    };

    #[test]
    fn insert_get() {
        let mut table = TTable::new(1000);
        let board = Board::default();
        let entry = TTableEntry::debug_entry(board.get_hash(), 2, 2);
        table.insert(entry);
        let &recieved = table.get(board.get_hash()).unwrap();
        assert_eq!(recieved, entry, "Insertet is not the same as recieved");
    }

    #[test]
    fn bucket() {
        let mut table = TTable::new(1);
        let mut board = Board::default();
        let bucket_hash = board.get_hash();

        let entry = TTableEntry::debug_entry(bucket_hash, 10, 0);

        table.insert(entry);
        board.make_pl_move_from_string::<true>("e2e4");
        table.insert(TTableEntry::debug_entry(board.get_hash(), 8, 0));
        board.make_pl_move_from_string::<true>("e7e5");
        table.insert(TTableEntry::debug_entry(board.get_hash(), 2, 7));
        board.make_pl_move_from_string::<true>("g1f3");
        table.insert(TTableEntry::debug_entry(board.get_hash(), 9, 0));

        board.make_pl_move_from_string::<true>("g8f6");
        table.insert(TTableEntry::debug_entry(board.get_hash(), 21, 7));

        table.get_bucket(bucket_hash).debug_print();
    }

    #[test]
    fn prev_power() {
        assert_eq!(TTable::prev_power_two(1), 1);
        assert_eq!(TTable::prev_power_two(2), 2);
        assert_eq!(TTable::prev_power_two(3), 2);
        assert_eq!(TTable::prev_power_two(5), 4);
        assert_eq!(TTable::prev_power_two(6), 4);
        assert_eq!(TTable::prev_power_two(8), 8);
        assert_eq!(TTable::prev_power_two(12), 8);
        assert_eq!(TTable::prev_power_two(16), 16);
        assert_eq!(TTable::prev_power_two(25), 16);
    }
}
