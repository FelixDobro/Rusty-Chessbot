
mod chess;

use std::error::Error;
use std::sync::LazyLock;
use crate::chess::Board;
use crate::chess::bitboard::EMPTY;
use crate::chess::chessMove::{*};
use crate::chess::square::Square;
use crate::chess::constants::{*};
use crate::chess::hash::{*};

use rayon::prelude::*;


use std::time::Instant;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{_pext_u64, _pdep_u64};

fn perft_copy(board: &Board, depth: u8) -> usize{

    let start = Instant::now();
    let total_nodes: usize = board.generate_pseudolegals().as_slice().iter()
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


    let mut nodes= board.generate_pseudolegals().as_slice().iter()
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
    
   


    let m1 = Move::new(Square::C4.u16(), Square::C5.u16(), 0);
    let m2 = Move::new(Square::E8.u16(), Square::C8.u16(), 3);
    let m3 = Move::new(Square::H6.u16(), Square::F7.u16(), 4);
    let m4 = Move::new(Square::E8.u16(), Square::G8.u16(), 2);
    let m5 = Move::new(Square::H2.u16(), Square::G1.u16(), 15);
    let m6 = Move::new(Square::D5.u16(), Square::C5.u16(), 0);
    let m7 = Move::new(Square::G2.u16(), Square::G4.u16(), 1);
    let m8 = Move::new(Square::B4.u16(), Square::A3.u16(), 4);
    let m9 = Move::new(Square::A1.u16(), Square::A3.u16(), 4);

    let mut board = Board::from_fen("2r1k2r/8/8/8/8/8/8/R3K2R w KQk").unwrap();

    // board.print();
    // perft_copy(&board, 5);

    // board.make_pl_move(m1);
    // board.print();

   
    // board.make_pl_move(m2);
    // board.print();
     

    // board.make_pl_move(m3);
    // board.print();
    // perft_copy(&board, 1);


    // board.make_pl_move(m4);
    // board.print();

    // board.make_pl_move(m5);
    // board.print();

    // board.make_pl_move(m6);
    // board.print();


    Ok(())
}
