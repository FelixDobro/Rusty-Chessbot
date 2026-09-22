mod common;

use chess_bot::chess::{
    board::{Board, nnue::Accumulator},
    chess_move::Move,
    constants::{BlackSide, WhiteSide},
};
use common::TEST_DATA;

#[test]
fn test_undo() {
    TEST_DATA.iter().for_each(|test| {
        let mut board = Board::from_fen(&test.fen).unwrap();
        let og = board.clone();
        let m = board.generate_legals().as_mut_slice()[0];
        board.make_pl_move::<true>(m);
        let new_accum_w = Accumulator::new::<WhiteSide>(&board);
        let new_accum_b = Accumulator::new::<BlackSide>(&board);

        assert_eq!(
            [new_accum_w, new_accum_b],
            board.latest_accum().clone(),
            "Accumulators after move are not the same as recalculated, Move: {}, fen: {}",
            m,
            test.fen
        );
        board.unmake_pl_move(m);

        assert_eq!(
            og.latest_accum().clone(),
            board.latest_accum().clone(),
            "Accumulators after undoing move are not the same as og, Move: {}, fen: {}",
            m,
            test.fen
        );
    });
}
