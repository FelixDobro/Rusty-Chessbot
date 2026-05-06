
mod chess;

use std::error::Error;
use crate::chess::constants::{*};
use chess::board::Board;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{_pext_u64, _pdep_u64};

fn main() -> Result<(), Box<dyn Error>> {
    
   
    // println!("{}", Color::Black as usize);
    let board = chess::board::Board::default();
    // println!("{:?}", board);
    let color = Color::White as usize;
    let piece = Piece::Pawn as usize;

    let white_pawns = board.get_bit_board(color * NUM_PIECES + piece);
    let sq = 21 as usize;
    let result = unsafe {_pext_u64(board.get_occupied(), STRAIGHT_LINES[sq])};
    let attack = STRAIGHT_LINES_MAGIC[sq][result as usize];

    print_bitboard(attack);
    // board.print()?;
    // let search_algorithm = Box::new(MinimaxSearch::new());
    // let mut uci_manager = UCIManager::new(search_algorithm);
    // uci_manager.start_protocol()?;
    Ok(())
}
