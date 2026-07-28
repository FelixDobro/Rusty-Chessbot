pub mod ids;
pub mod simple_search;

use crate::chess::chess_move::Move;
use crate::chess::chess_move::NULL_MOVE;

pub const MAX_SEARCH_DEPTH: usize = 64;
pub const GLOBAL_MAX_SEARCH_DURATION_H: u64 = 100000000u64;

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
            "{}, eval: {}, depth: {}, nodes: {} Mio",
            self.best_move,
            self.evaluation,
            self.depth,
            (self.nodes_searched as f32) / 1000000f32
        );
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
    pub fn new(size: usize) -> Self {
        let two_power_size = size.next_power_of_two();
        TTable {
            table: vec![Bucket::empty(); two_power_size].into_boxed_slice(),
            size: two_power_size,
        }
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
}
