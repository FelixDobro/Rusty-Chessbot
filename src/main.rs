
mod chess;
mod search;
mod evaluation;
mod move_sorting;
mod uci;

use core::time;
use std::error::Error;
use std::sync::LazyLock;
use crate::chess::board::Board;
use crate::chess::Game;
use crate::chess::chess_move::{*};
use crate::chess::square::Square;
use crate::chess::constants::{*};
use crate::chess::board::hash::{*};
use crate::move_sorting::{NoSorting, NumericSorting};
use crate::search::SearchAlgorithm;
use crate::uci::UCIManager;
use search::simple_search::NegaMaxCopy;
use evaluation::static_evaluation::MaterialEvaluator;
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


fn perft(game: &mut Game, depth: u8, move_list: &mut MoveList<256>) -> usize {

    if depth == 0 {
        return 1; 
    }

    let mut total_nodes: usize = 0;
    
    let moves = game.generate_pseudolegals();
    
    for &m in moves.as_slice() {
        if game.make_pl_move(m) {
            move_list.push(m);

            total_nodes += perft(game, depth - 1, move_list);
            move_list.pop();
            game.unmake_pl_move(m);
        }
    }

    total_nodes
}




fn main() -> Result<(), Box<dyn Error>> {

    let mut mangager = UCIManager::new(
        NegaMaxCopy,
        MaterialEvaluator, 
        NumericSorting
    );
  
    mangager.start_protocol();
    
    // let mut game = Game::from_fen("4k2r/6r1/8/8/8/8/3R4/R3K3 w Qk - 0 1").unwrap();
    // game.get_board().print();
    // game.make_pl_move(Move::from_string("d2d3", &game).unwrap());

    // game.get_board().print();
    // game.make_pl_move(Move::from_string("h7h6", &game).unwrap());
    
    // game.get_board().print();
    // game.make_pl_move(Move::from_string("c1h6", &game).unwrap());

    // game.get_board().print();
    // let m_critical = Move::from_string("h8h7", &game).unwrap();
    // game.make_pl_move(m_critical);

    // println!("{:?}", game.undo_info);
    // game.unmake_pl_move(m_critical);

    // let m1 = Move::new(Square::E2, Square::E4, 1);
    // let m2 = Move::new(Square::B7, Square::B5,1);
    // let m3 = Move::new(Square::F1, Square::B5, 4);
    // let m4 = Move::new(Square::A7, Square::A6, 0);
    // let m5 = Move::new(Square::H2, Square::G1, 15);
    // let m6 = Move::new(Square::D5, Square::C5, 0);
    // let m7 = Move::new(Square::G2, Square::G4, 1);
    // let m8 = Move::new(Square::B4, Square::A3, 4);
    // let m9 = Move::new(Square::A1, Square::A3, 4);

    // let repetitive_w_1 = Move::new(Square::E1, Square::E2, 0);
    // let repetitive_w_2 = Move::new(Square::E2, Square::E1, 0);
    // let repetitive_b_1 = Move::new(Square::E8, Square::E7, 0);
    // let repetitive_b_2 = Move::new(Square::E7, Square::E8, 0);



    Ok(())
}
