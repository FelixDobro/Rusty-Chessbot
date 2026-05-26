use chess_bot::{chess::board::Board, search::{SearchAlgorithm, simple_search::{Negamax, NegamaxTT}}};


#[test]
fn negamax_should_find_best_move() {
    let mut board = Board::from_fen("r1b1k2r/pp3ppp/1qn1p3/3p4/8/5N2/PPP1P1PP/R2Q1RK1 w kq - 0 1").unwrap();
    let mut search = Negamax{};
    let res = search.search(&mut board, 4);
    assert!(res.is_some(), "Negamax should be able to find best move");
}

#[test]
fn negamax_should_find_same_as_tt() {
    let mut board = Board::default();
    let mut simple_search = Negamax{};
    let table_size = 2u64.pow(22);
    let mut advanced_search = NegamaxTT::new(table_size as usize);

    let mut depth = 2;
    let first_result = simple_search.search(&mut board, depth);
    let second_result = advanced_search.search(&mut board, depth);
    assert_eq!(first_result, second_result, "With TTable does not find the same as without TTable for depth: {}", depth);

    let mut depth = 3;
    let first_result = simple_search.search(&mut board, depth);
    let second_result = advanced_search.search(&mut board, depth);
    assert_eq!(first_result, second_result, "With TTable does not find the same as without TTable for depth: {}", depth);

    let mut depth = 4;
    let first_result = simple_search.search(&mut board, depth);
    let second_result = advanced_search.search(&mut board, depth);
    assert_eq!(first_result, second_result, "With TTable does not find the same as without TTable for depth: {}", depth);

    let mut depth = 5;
    let first_result = simple_search.search(&mut board, depth);
    let second_result = advanced_search.search(&mut board, depth);
    assert_eq!(first_result, second_result, "With TTable does not find the same as without TTable for depth: {}", depth);

    let mut depth = 6;
    let first_result = simple_search.search(&mut board, depth);
    let second_result = advanced_search.search(&mut board, depth);
    assert_eq!(first_result, second_result, "With TTable does not find the same as without TTable for depth: {}", depth);

}