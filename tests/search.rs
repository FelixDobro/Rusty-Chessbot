

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
fn does_not_draw() {
    let mut ngeamax = NegamaxTT::new(100000);
    let mut search = IDSearch::new(ngeamax);
    let mut board = Board::from_fen("8/6p1/2p1k1p1/5p2/3P1P1q/4B3/5K2/6R1 w - - 0 56").unwrap();
    board.print();
    board.make_pl_move_from_string::<true>("f2e2");
    board.print();
    search.depth_search(&mut board, 5);

    board.make_pl_move_from_string::<true>("e6f7");
    board.make_pl_move_from_string::<true>("g1a1");
    search.depth_search(&mut board, 5);
    
    board.make_pl_move_from_string::<true>("f7e6");
    board.make_pl_move_from_string::<true>("a1g1");
   
    board.make_pl_move_from_string::<true>("e6f7");
    board.make_pl_move_from_string::<true>("g1a1");
    board.print();
    println!();
    println!("{:?}", board.get_turn());
    let possible_repetition = search.depth_search(&mut board, 5).unwrap();
    println!("Hello");
    possible_repetition.print_info();
    assert!(possible_repetition.best_move != Move::from_string("f7e6", &board).unwrap(), "Engine runs into draw with massive advantage");



}

