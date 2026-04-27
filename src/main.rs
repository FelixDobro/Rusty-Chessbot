mod utils;
mod search;
mod evaluation;
mod uci;

use chess::{Board, ChessMove, Square, File::*, Rank::*};
use crate::{uci::UCIManager, utils::*};
use crate::search::MinimaxSearch;
use std::error::Error;
use std::time::Instant;

fn main() -> Result<(), Box<dyn Error>> {
    let search_algorithm = Box::new(MinimaxSearch::new());
    let mut uci_manager = UCIManager::new(search_algorithm);
    uci_manager.start_protocol()?;
    Ok(())
}
