
mod chess;

use std::error::Error;
use crate::chess::{chessMove::Move, constants::*};
use chess::board::Board;

use std::time::Instant;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{_pext_u64, _pdep_u64};


fn main() -> Result<(), Box<dyn Error>> {
    
   
    // println!("{}", Color::Black as usize);
    let mut board = chess::board::Board::default();
    // println!("{:?}", board);
    let color = Color::White;
    let square = Square::A2;

    let attack = board.sq_attacked_by(square, color);
    // let boar = 1u64 << sq;
    // let mut right_prefix = boar;
    // right_prefix = (right_prefix << 1) & !FILE_H;
    // moves |= (right_prefix >> 1*8) | (left_prefix << 8*1);   
    let m = Move::new(Square::E2 as u16, Square::E4 as u16, 1);
    let m2 = Move::new(Square::E7 as u16, Square::E5 as u16, 1);
    let m3 = Move::new(Square::G1 as u16, Square::F3 as u16, 0);
    let m4 = Move::new(Square::B8 as u16, Square::C6 as u16, 0);
    let m5 = Move::new(Square::F3 as u16, Square::E5 as u16, 4);
    let m6 = Move::new(Square::C6 as u16, Square::E5 as u16, 4);
    board.make_pseudolegal_move(m);
    let now = Instant::now();
    
    board.make_pseudolegal_move(m2);
    board.make_pseudolegal_move(m3);
    board.make_pseudolegal_move(m4);
    board.make_pseudolegal_move(m5);
    
    board.make_pseudolegal_move(m6);
    let time_elapsed = now.elapsed();
    println!("Time elapsed: {:?}", time_elapsed);


  

    print_bitboard(board.get_bit_board(Piece::Pawn.index() + Color::Black.offset()));
    print!("{:?}", board.get_piece(Square::D5));
    println!("{:?} attacked by {:?}: {}", square, color, attack);
    board.print();
    // board.print()?;
    // let search_algorithm = Box::new(MinimaxSearch::new());
    // let mut uci_manager = UCIManager::new(search_algorithm);
    // uci_manager.start_protocol()?;
    Ok(())
}
