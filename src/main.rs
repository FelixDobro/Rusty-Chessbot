#[allow(dead_code)]
mod chess;
mod move_sorting;
mod search;
mod uci;

use crate::chess::board::bitboard::init_lazylocks;
use crate::search::ids::IDSearch;
use crate::search::simple_search::NegamaxTT;
use crate::uci::UCIManager;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    init_lazylocks();
    let negamax = NegamaxTT::new(2u64.pow(25) as usize);
    let search = IDSearch::new(negamax);
    let mut mangager = UCIManager::new(search);
    mangager.start_protocol()?;

    Ok(())
}
