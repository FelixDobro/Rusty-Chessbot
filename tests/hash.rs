mod common;

use chess_bot::chess::{board::Board, chess_move::Move};
use common::TEST_DATA;

fn test_1_move_deep(board: &mut Board, fen: &str, parent_move: Move) {
    board
        .generate_pseudolegals()
        .as_slice()
        .iter()
        .for_each(|&m| {
            if board.make_pl_move::<false>(m) {
                assert_eq!(
                    board.get_hash(),
                    board.calculate_hash(),
                    "Hash inconsistency after moves {} {} in FEN {}",
                    parent_move,
                    m,
                    fen
                );
                board.unmake_pl_move(m);
            }
        });
}

#[test]
fn test_hash_vs_calculated() {
    println!("test hash_vs_calculated: Running {} tests", TEST_DATA.len());
    TEST_DATA.iter().for_each(|test| {
        let fen = &test.fen;
        let mut board = Board::from_fen(fen).unwrap();
        board
            .generate_pseudolegals()
            .as_slice()
            .iter()
            .for_each(|&m| {
                if board.make_pl_move::<false>(m) {
                    assert_eq!(
                        board.get_hash(),
                        board.calculate_hash(),
                        "Hash inconsistency after moves {} in FEN {}",
                        m,
                        fen
                    );
                    println!("test_hash_vs_calculated_{} ... ok", test.name);
                    test_1_move_deep(&mut board, fen, m);
                    board.unmake_pl_move(m);
                }
            });
    });
}
