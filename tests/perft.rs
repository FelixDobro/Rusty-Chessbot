use std::time::Instant;
use rstest::rstest;
use rayon::prelude::*;
use chess_bot::chess::board::Board;

pub fn perft_copy_single_threaded(board: &Board, depth: u8) -> usize{

    let total_nodes: usize = board.generate_pseudolegals().as_sclice().iter()
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

    let mut nodes= board.generate_pseudolegals().as_sclice().iter()
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

    let total_nodes: usize = board.generate_pseudolegals().as_sclice().par_iter()
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

    let mut nodes= board.generate_pseudolegals().as_sclice().par_iter()
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



#[rstest]
#[case::start_pos("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 5, 4_865_609)]
#[case::start_pos("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1", 5, 9771632)]
#[case::start_pos("rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2", 5, 11719785)]
#[case::start_pos("8/8/8/4p1K1/2k1P3/8/8/8 b - - 0 1", 8, 4729839)]
#[case::start_pos("4k2r/6r1/8/8/8/8/3R4/R3K3 w Qk - 0 1", 5, 10534193)]
#[case::start_pos("rnbqkbnr/ppp1pppp/8/8/2PpP3/5P2/PP1P2PP/RNBQKBNR b KQkq c3 0 3", 5, 16571869)]
#[case::start_pos("5rk1/p4Qpp/1p6/3B4/2Pb4/1P4Nq/P2r1P1P/4R1K1 b - - 0 26", 6, 150469809)]
#[case::start_pos("8/6n1/8/8/5K2/8/8/1k6 w - - 0 70", 8, 20335969)]
#[case::start_pos("r2rq1k1/1pp2pb1/p1n1bnpp/4p3/PP2P3/B1P1NNP1/2Q1BP1P/3RR1K1 b - - 4 18", 4, 3885495)]
#[case::start_pos("2rq1rk1/ppnnbppp/4p3/3pP3/3P4/1P1Q1N2/P4PPP/R1B1RNK1 b - - 4 14", 5, 48089521)]
#[case::start_pos("3k4/1p3KNq/4r3/3p4/3PnPP1/8/8/8 w - - 9 63", 7, 28101752)]
#[case::start_pos("8/8/5P2/p1p4k/8/1P6/8/4K3 w - - 0 42", 8, 25956602)]
#[case::start_pos("8/5K2/8/4kPRP/7r/8/8/8 w - - 1 57", 6, 16201298)]
#[case::start_pos("3k4/1p3KNq/4r3/3p4/3PnPP1/8/8/8 w - - 9 63", 7, 28101752)]

fn test_perft(#[case] fen: &str, #[case] depth: u8, #[case] expected_nodes: usize) {
    let board = Board::from_fen(fen).unwrap();
    assert_eq!(perft_copy(&board, depth), expected_nodes);
}
