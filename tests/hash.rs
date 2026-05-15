mod common;


use common::TEST_DATA;
use chess_bot::chess::{Board, chessMove::Move};

fn test_1_move_deep(board: &Board, fen: &str, parent_move: Move) {
    board.generate_pseudolegals().as_slice()
    .iter()
    .for_each(|&m| {
        let hash_before = board.get_hash();
        if let Some(new_board) = board.make_pl_move_copy(m) {
            assert_eq!(
                new_board.get_hash(),
                new_board.calculate_hash(),
                "Hash inconsistency after moves {} {} in FEN {}",parent_move, m, fen
        )
        }
    });
}



#[test]
fn test_hash_vs_calculated() {
    println!("test hash_vs_calculated: Running {} tests", TEST_DATA.len());
    TEST_DATA.iter().for_each(|test| {
        let fen = &test.fen;
        let board = Board::from_fen(fen).unwrap();
        board.generate_pseudolegals().as_slice()
        .iter()
        .enumerate().for_each(|(i, &m)| {
            if let Some(new_board) = board.make_pl_move_copy(m) {
                assert_eq!(
                    new_board.get_hash(),
                    new_board.calculate_hash(),
                    "Hash inconsistency after moves {} in FEN {}", m, fen
                );
                println!("test_hash_vs_calculated_{} ... ok", test.name);
                test_1_move_deep(&new_board, fen, m);
            }
        });

    });
    
}
