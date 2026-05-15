mod common;
use common::{engine_perf_specs, TEST_DATA};
use rstest_reuse::apply;


use std::time::Instant;
use rstest::rstest;
use rayon::prelude::*;
use chess_bot::chess::Board;

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



#[apply(engine_perf_specs)]
fn test_perft(#[case] fen: &str, #[case] depth: u8, #[case] expected_nodes: usize) {
    let board = Board::from_fen(fen).unwrap();
    assert_eq!(perft_copy(&board, depth), expected_nodes);
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
fn test_perft_default() {
    let board = Board::default();
    assert_eq!(perft_copy(&board, 5), 4_865_609);
}
