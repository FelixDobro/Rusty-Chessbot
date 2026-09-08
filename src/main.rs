#[allow(dead_code)]
mod chess;
mod move_sorting;
mod parameters;
mod search;
mod uci;

use crate::uci::UCIManager;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut mangager = UCIManager::new();
    mangager.start_protocol()?;

    Ok(())
}
