
mod chess;

use std::error::Error;
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
    
   


    let m1 = Move::new(Square::A7.u16(), Square::A5.u16(), 1);
    let m2 = Move::new(Square::A7.u16(), Square::A6.u16(), 0);
    let m3 = Move::new(Square::B1.u16(), Square::A3.u16(), 0);
    let m4 = Move::new(Square::A6.u16(), Square::A5.u16(), 0);
    let m5 = Move::new(Square::B1.u16(), Square::B2.u16(), 0);
    let m6 = Move::new(Square::G8.u16(), Square::H6.u16(), 0);
    let m7 = Move::new(Square::G2.u16(), Square::G4.u16(), 1);
    let m8 = Move::new(Square::B4.u16(), Square::A3.u16(), 4);
    let m9 = Move::new(Square::A1.u16(), Square::A3.u16(), 4);

    let mut board = Board::from_fen("rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2").unwrap();

    board.get_enpassant().print();
    println!("Before");
    println!("{}", board.get_hash());
    board.make_pl_move(m1);
    let second_hash_updated = board.get_hash();
    let second_hash_calculated = board.calculate_hash();

    assert_eq!(second_hash_calculated, second_hash_updated);
    Ok(())
}
