pub mod simple_search;
pub mod ids;
use crate::chess::board::Board;
use crate::search::Ntype::Exact;
use std::time::Duration;


use crate::chess::chess_move::NULL_MOVE;
use crate::chess::square::Square;
use crate::{chess::chess_move::Move};



#[derive(Default, Clone, Debug)]
pub struct SearchLimits {
    pub max_depth: Option<u8>,
    pub max_time: Option<Duration>,
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

    pub fn time(ms: u64) -> Self {
        Self {
            max_time: Some(Duration::from_millis(ms)),
            ..Default::default()
        }
    }
}


#[derive(Debug, PartialEq)]
pub struct SearchResult {
    pub best_move: Move,
    pub evaluation: i16,
    pub nodes_searched: u64,
    pub depth: u8
}

impl SearchResult {

    pub fn print_info(&self) {
        println!("{}, eval: {}, depth: {}", self.best_move, self.evaluation, self.depth);
    }
}


pub trait SearchAlgorithm {
    fn search(&mut self, board: &mut Board, limits: &SearchLimits) -> Option<SearchResult>;
}


#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Ntype {
    Exact,
    Lower,
    Upper
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
            age: 0 
        }
    }
    
    fn debug_entry(hash: u64, depth: u8, age: u8) -> Self {
        TTableEntry { hash,
            best_move: NULL_MOVE,
            depth: depth,
            score: 0, 
            ntype: Exact,
            age 
        }
    }
}

#[repr(align(64))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Bucket {
    entries: [TTableEntry; 4]
}

impl Bucket {
    pub const fn empty() -> Self {
        Bucket { entries: [TTableEntry::empty(); 4] }
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
                return
            }

            if existing_entry.age != new_entry.age {
                if existing_entry.depth < lowest_outdated_depth {
                    lowest_outdated_depth = existing_entry.depth;
                    should_replace = true;
                    replace_index = i;
                }
            }
            
            else if existing_entry.depth < lowest_depth {
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
    size: usize
}

impl TTable {

    pub fn new(size: usize) -> Self {

        TTable { table: vec![Bucket::empty(); size].into_boxed_slice(), size:size }
    }

    pub fn insert(&mut self, entry: TTableEntry) {
        let index = entry.hash as usize % self.size;
        self.table[index].insert(entry);
    }

    pub fn get(&self, hash: u64) -> Option<&TTableEntry> {
        let index = hash as usize % self.size;
        self.table[index].get(hash)
    }

    pub fn get_bucket(&self, hash: u64) -> Bucket {
        let index = hash as usize % self.size;
        self.table[index]
    }
}

#[cfg(test)]
mod test {

    use crate::{chess::{board::Board, chess_move::Move}, search::{Ntype::Exact, TTable, TTableEntry}};

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
        let bucket_hash= board.get_hash();
        let best_move =Move::from_string("e2e4", &board).unwrap();

        let entry = TTableEntry::debug_entry(bucket_hash, 10, 0);

        table.insert(entry);
        board.make_pl_move_from_string::<true>("e2e4");
        table.insert(
            TTableEntry::debug_entry(board.get_hash(), 8, 0)
        );
        board.make_pl_move_from_string::<true>("e7e5");
        table.insert(
            TTableEntry::debug_entry(board.get_hash(), 2, 7)
        );
        board.make_pl_move_from_string::<true>("g1f3");
        table.insert(
            TTableEntry::debug_entry(board.get_hash(), 9, 0)
        );

        board.make_pl_move_from_string::<true>("g8f6");
        table.insert(
            TTableEntry::debug_entry(board.get_hash(), 21, 7)
        );

        table.get_bucket(bucket_hash).debug_print();

    }
}