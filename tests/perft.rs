mod common;
use common::{TEST_DATA};
use rstest_reuse::apply;


use std::time::Instant;
use rstest::rstest;
use rayon::prelude::*;
use chess_bot::chess::{board::Board};

use crate::common::print_test;





fn perft(game: &mut Board, depth: u8) -> usize {

    if depth == 0 {
        return 1; 
    }

    let mut total_nodes: usize = 0;
    
    let total_nodes: usize = game.generate_pseudolegals().as_slice().par_iter()
    .map(|&m| 
    {
        let mut game_cloned = game.clone();
        if game_cloned.make_pl_move(m) {
            let nodes = perft_single(&mut game_cloned, depth - 1);
            return nodes 
        }
        0
    }
    ).sum();


    total_nodes
}


fn perft_single(game: &mut Board, depth: u8) -> usize {

    if depth == 0 {
        return 1; 
    }

    let mut total_nodes: usize = 0;
    
    let moves = game.generate_pseudolegals();

    for &m in moves.as_slice() {
        if game.make_pl_move(m) {
            total_nodes += perft_single(game, depth - 1);
            game.unmake_pl_move(m);
        }
    }

    total_nodes
}



#[test]
#[ignore = "Takes very long"]
fn test_perft() {
    let mut num_fails = 0;
    TEST_DATA.iter().for_each(
        |test| {
            let mut game = Board::from_fen(&test.fen).unwrap();
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
fn test_perft_full() {
    let mut num_fails = 0;
    TEST_DATA.iter().for_each(
        |test| {
            let mut board = Board::from_fen(&test.fen).unwrap();
            if perft(&mut board, test.depth) != test.expected {
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
    let mut game = Board::default();
    assert_eq!(perft(&mut game, 4), 197281);
}


#[test]
fn quick_test_perft_kiwipete() {
    let mut game = Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq").unwrap();
    assert_eq!(perft(&mut game, 3), 97862);
}


#[test]
fn quick_test_perft_1() {
    let mut game = Board::from_fen("8/8/8/8/8/K7/P7/k7 w - - 0 1").unwrap();
    assert_eq!(perft(&mut game, 6), 6249);
}

