mod common;
use common::{TEST_DATA};
use rstest_reuse::apply;


use std::time::Instant;
use rstest::rstest;
use rayon::prelude::*;
use chess_bot::chess::{Game, board::Board};

use crate::common::print_test;

pub fn perft_copy_single_threaded(board: &Board, depth: u8) -> usize{

    let total_nodes: usize = board.generate_pseudolegals().as_slice().iter()
    .map(|m| 
    {
        if let Some(mut new_board) = board.make_pl_move_copy(*m) {
            let nodes = private_perft_copy_single_threaded(&new_board,depth - 1);
            return nodes 
        }
        0
    }
    ).sum();
    total_nodes
}

pub fn private_perft_copy_single_threaded(board: &Board, depth: u8) -> usize {
    if depth == 0 {
        return 1; 
    }

    let mut nodes= board.generate_pseudolegals().as_slice().iter()
    .map(|m| 
    {
        if let Some(mut new_board) = board.make_pl_move_copy(*m) {
            let nodes = private_perft_copy_single_threaded(&new_board, depth - 1);
            return nodes 
        }
        0
    }
    ).sum();

    return nodes
}


fn perft_copy(board: &Board, depth: u8) -> usize{

    let total_nodes: usize = board.generate_pseudolegals().as_slice().par_iter()
    .map(|m| 
    {
        if let Some(mut new_board) = board.make_pl_move_copy(*m) {
            let nodes = private_perft_copy(&new_board, depth - 1);
            return nodes 
        }
        0
    }
    ).sum();

    total_nodes
    
}

fn private_perft_copy(board: &Board, depth: u8) -> usize {
    
    if depth == 0 {
        return 1; 
    }

    let mut nodes= board.generate_pseudolegals().as_slice().par_iter()
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



fn perft(game: &mut Game, depth: u8) -> usize {

    if depth == 0 {
        return 1; 
    }

    let mut total_nodes: usize = 0;
    
    let moves = game.generate_pseudolegals();

    for &m in moves.as_slice() {
        if game.make_pl_move(m) {
            total_nodes += perft(game, depth - 1);
            game.unmake_pl_move(m);
        }
    }

    total_nodes
}



#[test]
#[ignore = "Takes very long"]
fn test_perft_no_copy() {
    let mut num_fails = 0;
    TEST_DATA.iter().for_each(
        |test| {
            let mut game = Game::from_fen(&test.fen).unwrap();
            if perft(&mut game, test.depth) != test.expected {
                print_test(&test.name, false);
                num_fails += 1;
            }
            else {
                print_test(&test.name, true);
            }
        }
    );
    if num_fails != 0 {panic!()}
}


#[test]
#[ignore = "Takes very long"]
fn test_perft() {
    let mut num_fails = 0;
    TEST_DATA.iter().for_each(
        |test| {
            let board = Board::from_fen(&test.fen).unwrap();
            let test_name = &("Perft no copy::".to_string() + &test.fen);
            if perft_copy_single_threaded(&board, test.depth) != test.expected {
                print_test(test_name, false);
                num_fails += 1;
            }
            else {
                print_test(test_name, true);
            }
        }
    );
    if num_fails != 0 {panic!()}
}


// #[test]
// fn test_perft() {
//     TEST_DATA.iter()
//     .for_each(|test| {
//         let board = Board::from_fen(&test.fen).unwrap();
//         assert_eq!(perft_copy(&board, test.depth), test.expected);
//     }
        
//     );
// }

#[test]
fn quick_test_perft_default() {
    let board = Board::default();
    assert_eq!(perft_copy_single_threaded(&board, 4), 197281);
}


#[test]
fn quick_test_perft_kiwipete() {
    let board = Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq").unwrap();
    assert_eq!(perft_copy_single_threaded(&board, 3), 97862);
}


#[test]
fn quick_test_perft_1() {
    let board = Board::from_fen("8/8/8/8/8/K7/P7/k7 w - - 0 1").unwrap();
    assert_eq!(perft_copy_single_threaded(&board, 6), 6249);
}

