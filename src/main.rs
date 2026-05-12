
mod chess;

use std::error::Error;
use crate::chess::{chessMove::{Move, MoveList}, constants::*};
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

    // let boar = 1u64 << sq;
    // let mut right_prefix = boar;
    // right_prefix = (right_prefix << 1) & !FILE_H;
    // moves |= (right_prefix >> 1*8) | (left_prefix << 8*1);  
    
    let m = Move::new(Square::E2 as u16, Square::E3 as u16, 0);
    let m2 = Move::new(Square::C7 as u16, Square::C6 as u16, 0);
    let m3 = Move::new(Square::G1 as u16, Square::F3 as u16, 0);
    let m4 = Move::new(Square::D8 as u16, Square::A5 as u16, 0);
    let m5 = Move::new(Square::F1 as u16, Square::E2 as u16, 0);
    let m6 = Move::new(Square::A5 as u16, Square::D2 as u16, 0);
    let m7 = Move::new(Square::B2 as u16, Square::B4 as u16, 1);
    let mut list = board.generate_pseudolegals();
    board.make_pl_move(m);
    board.make_pl_move(m2);
    board.make_pl_move(m3);
    board.make_pl_move(m4);
    board.make_pl_move(m5);
    board.make_pl_move(m6);
    board.print();
    
    Board::default().perft_copy(8);
    // pseudos.print_list();
    // board.print();
    // let mut some_list = MoveList::new();
    // board.rook_moves::<WhiteSide>(&mut some_list);
    // print_bitboard(board.get_bit_board(Piece::Knight.index()));
    
    // for i in 0..6 {
    //     println!("{}",i);
    //     print_bitboard(board.get_bit_board(i));
    //     print_bitboard(board.get_bit_board(i+ NUM_PIECES ));
    //     println!();
    //     println!();
    // }
    // current issue a2a4
    
    // board.print()?;
    // let search_algorithm = Box::new(MinimaxSearch::new());
    // let mut uci_manager = UCIManager::new(search_algorithm);
    // uci_manager.start_protocol()?;
    Ok(())
}
