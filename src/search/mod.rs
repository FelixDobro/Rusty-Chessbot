pub mod simple_search;
pub mod ids;
use crate::chess::board::Board;
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
    pub nodes_searched: u64
}


pub trait SearchAlgorithm {
    fn search(&mut self, board: &mut Board, limits: &SearchLimits) -> Option<SearchResult>;
}


#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Ntype {
    Exact,
    Lower,
    Upper
}

#[derive(Copy, Clone)]
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
}

#[repr(align(64))]
#[derive(Copy, Clone)]
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

    
}