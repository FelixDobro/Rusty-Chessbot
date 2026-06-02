

use chess_bot::{chess::{board::Board, chess_move::Move}, search::{SearchAlgorithm, SearchLimits, ids::IDSearch, simple_search::{Negamax, NegamaxTT}}};


#[test]
fn negamax_should_find_best_move() {
    let mut board = Board::from_fen("r1b1k2r/pp3ppp/1qn1p3/3p4/8/5N2/PPP1P1PP/R2Q1RK1 w kq - 0 1").unwrap();
    let mut search = Negamax::new();
    let res = search.search(&mut board,&SearchLimits::depth(2));
    assert!(res.is_some(), "Negamax should be able to find best move");
}


#[test]
fn can_find_move() {
    let mut search = NegamaxTT::new(1000);
    let mut board = Board::from_fen("8/1R3k2/2p5/2K1Q3/8/8/8/7q b - - 54 76").unwrap();
    let search_result = search.search(&mut board, &SearchLimits::depth(6));

    assert!(search_result.is_some(), "Does not find move, even though move possible");
}




#[test]
fn test_hash_table() {

    let negamax = NegamaxTT::new(2u64.pow(18) as usize);
    let mut search = IDSearch::new(negamax);
    let mut board = Board::from_fen("r1b1kbnr/pp3ppp/2p5/1B1p2q1/4P3/2N2P2/PPn2KPP/6NR b kq - 1 11").unwrap();

    search.search(&mut board, &SearchLimits::depth(5));
    board.make_pl_move_from_string::<true>("g5e3");
    search.search(&mut board, &SearchLimits::depth(4));
    board.make_pl_move_from_string::<true>("f2f1");

    let first_search = search.search(&mut board, &SearchLimits::depth(5)).unwrap();
    let first_search_val = first_search.evaluation;
    let first_search_res = search.search(&mut board, &SearchLimits::depth(5)).unwrap().best_move;
    let second_search_res = search.search(&mut board, &SearchLimits::depth(4)).unwrap().best_move;
    let mate_in_1 = Move::from_string("e3e1", &board).unwrap();

    assert_eq!(first_search_res, mate_in_1, "first search oversaw mate in 1");
    assert_eq!(second_search_res, mate_in_1, "second search oversaw mate in 1");
}