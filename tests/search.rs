use chess_bot::{
    chess::{board::Board, chess_move::Move},
    search::{Restrictions, SearchControl, simple_search::Search},
};

#[test]
fn does_not_draw() {
    let mut search = Search::new(1, SearchControl::new());
    let mut board = Board::from_fen("8/6p1/2p1k1p1/5p2/3P1P1q/4B3/5K2/6R1 w - - 0 56").unwrap();

    board.make_pl_move_from_string::<true>("f2e2");

    search.ids::<{ Restrictions::DEPTH }>(&mut board, Restrictions::depth(5));

    board.make_pl_move_from_string::<true>("e6f7");
    board.make_pl_move_from_string::<true>("g1a1");
    search.ids::<{ Restrictions::DEPTH }>(&mut board, Restrictions::depth(5));

    board.make_pl_move_from_string::<true>("f7e6");
    board.make_pl_move_from_string::<true>("a1g1");

    board.make_pl_move_from_string::<true>("e6f7");
    board.make_pl_move_from_string::<true>("g1a1");

    let possible_repetition = search
        .ids::<{ Restrictions::DEPTH }>(&mut board, Restrictions::depth(5))
        .unwrap();

    assert!(
        possible_repetition.best_move != Move::from_string("f7e6", &board).unwrap(),
        "Engine runs into draw with massive advantage"
    );
}
