

use std::time::{Duration, Instant};

use chess_bot::{chess::{board::Board, chess_move::Move}, search::{SearchLimits, ids::IDSearch, simple_search::{Negamax, NegamaxTT}}};


#[test]
fn negamax_should_find_best_move() {
    let mut board = Board::from_fen("r1b1k2r/pp3ppp/1qn1p3/3p4/8/5N2/PPP1P1PP/R2Q1RK1 w kq - 0 1").unwrap();
    let mut search = Negamax::new();
    let res = search.negamax(&mut board, 2);
    assert!(res.is_some(), "Negamax should be able to find best move");
}


#[test]
fn can_find_move() {
    let mut search = NegamaxTT::new(1000);
    let mut board = Board::from_fen("8/1R3k2/2p5/2K1Q3/8/8/8/7q b - - 54 76").unwrap();
    let search_result = search.negamax(&mut board, 5, &Instant::now(), &Duration::from_secs(10));

    assert!(search_result.is_some(), "Does not find move, even though move possible, possibly timed out");
}


