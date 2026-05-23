#[allow(dead_code)]

mod chess;
mod search;
mod move_sorting;
mod uci;

use core::time;
use std::error::Error;
use std::sync::LazyLock;
use crate::chess::board::Board;
use crate::chess::chess_move::{*};
use crate::chess::square::Square;
use crate::chess::constants::{*};
use crate::chess::board::hash::{*};
use crate::move_sorting::{NoSorting, NumericSorting};
use crate::search::SearchAlgorithm;
use crate::uci::UCIManager;
use crate::chess::board::bitboard;
use crate::search::simple_search::Negamax;
use chess_bot::chess::board::evaluation::MG;
use rayon::prelude::*;


use std::time::Instant;

pub fn init_lazylocks() {
    bitboard::init_lazylocks();
}

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{_pext_u64, _pdep_u64};



fn perft(board: &mut Board, depth: u8, move_list: &mut MoveList<256>) -> usize {

    if depth == 0 {
        return 1; 
    }

    let mut total_nodes: usize = 0;
    
    let moves = board.generate_pseudolegals();
    
    for &m in moves.as_slice() {
        if board.make_pl_move::<true>(m) {
            move_list.push(m);
            move_list.print_list();
            println!();
            total_nodes += perft(board, depth - 1, move_list);
            move_list.pop();
            board.unmake_pl_move(m);
        }
    }

    total_nodes
}




fn main() -> Result<(), Box<dyn Error>> {
    

    // let mut search = Negamax{};
    // let mut mangager = UCIManager::new(Box::new(search));
    // mangager.start_protocol();
    println!("{}", MG[0 + WhiteSide::OFFSET][Square::E2.index()]);
    let mut board = Board::default();
    println!("{}", board.eval());

    board.make_pl_move::<true>(Move::from_string("e2e3", &board).unwrap());
    println!("{}", board.eval());


    // board.get_board().print();
    // board.make_pl_move::<true>(Move::from_string("h7h6", &board).unwrap());
    
    // board.get_board().print();
    // board.make_pl_move::<true>(Move::from_string("c1h6", &board).unwrap());

    // board.get_board().print();
    // let m_critical = Move::from_string("h8h7", &board).unwrap();
    // board.make_pl_move::<true>(m_critical);

    // println!("{:?}", board.undo_info);
    // board.unmake_pl_move(m_critical);

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
