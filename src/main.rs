
mod chess;

use std::error::Error;
use crate::chess::{chessMove::{Move, MoveList}, constants::*};
use chess::board::Board;
use rayon::prelude::*;
use std::time::Instant;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{_pext_u64, _pdep_u64};

fn perft_copy(board: &Board, depth: u8) -> usize{

    let start = Instant::now();
    let total_nodes: usize = board.generate_pseudolegals().as_sclice().iter()
    .map(|m| 
    {   

        if let Some(mut new_board) = board.make_pl_move_copy(*m) {
            let nodes = private_perft_copy(&new_board, depth - 1);
            println!("Move: {}, found nodes: {}", m, nodes);
            return nodes 
        }
        0
    }
    ).sum();
    let time = start.elapsed();
    println!("Found nodes: {}, time: {:?}", total_nodes, time);
    total_nodes
    
}

fn private_perft_copy(board: &Board, depth: u8) -> usize {
    
    if depth == 0 {
        return 1; 
    }
    let moves = board.generate_pseudolegals();


    let mut nodes= board.generate_pseudolegals().as_sclice().iter()
    .map(|m| 
    {   
        if let Some(mut new_board) = board.make_pl_move_copy(*m) {
            let nodes = private_perft_copy(&new_board, depth - 1);
            return nodes 
        }
        0
    }
    ).sum();

    return nodes
}



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
    
    let m1 = Move::new(Square::D4 as u16, Square::D3 as u16, 0);
    let m2 = Move::new(Square::G1 as u16, Square::E2 as u16, 0);
    let m3 = Move::new(Square::D3 as u16, Square::E2 as u16, 4);
    let m4 = Move::new(Square::A2 as u16, Square::A3 as u16, 0);
    let m5 = Move::new(Square::E2 as u16, Square::F1 as u16, 15);
    let m6 = Move::new(Square::G8 as u16, Square::H6 as u16, 0);
    let m7 = Move::new(Square::G2 as u16, Square::G4 as u16, 1);
    let m8 = Move::new(Square::B4 as u16, Square::A3 as u16, 4);
    let m9 = Move::new(Square::A1 as u16, Square::A3 as u16, 4);

    let mut board = Board::from_fen("r2rq1k1/1pp2pb1/p1n1bnpp/4p3/PP2P3/B1P1NNP1/2Q1BP1P/3RR1K1 b - - 4 18").unwrap();
    board.print();

    // board.make_pl_move(m1);

    // board.print();


    // board.make_pl_move(m2);
    // board.print();
    

  
    // board.make_pl_move(m3);
    // board.print();

    
    // board.make_pl_move(m4);
    // board.print();
    // perft_copy(&board, 1);

    // board.make_pl_move(m5);

    // board.print();

    // pseudos.print_list();
    // board.print()
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
