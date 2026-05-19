use chess_bot::{evaluation::static_evaluation::MaterialEvaluator, move_sorting::NoSorting, search::simple_search::NegaMaxCopy, uci::UCIManager};


#[test]
fn test_move_seq() {
    let mut uci = UCIManager::new(NegaMaxCopy, MaterialEvaluator, NoSorting);
    let command = "position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 moves e2e4 h7h5 g1f3 e7e5 f1c4 d8e7 e1g1 c7c5 b1c3 a7a5 d2d3 g7g6 c1g5 f7f6 g5e3 e7d6 d1e2 e8d8 a2a3 g8e7 a1d1 d8e8 b2b3 e8d8 c3b1 e7g8 c2c3 g8e7 b3b4 c5b4 a3b4 a5b4 c3b4 d6b4 c4f7 h8h7 f7c4 b7b5 c4b3 b4b3 b1d2 b3g8 d1b1 d7d5 b1b5 d8e8 e4d5 e7d5 d3d4 d5e3 f2e3 e5d4 f3d4 c8a6 b5b8 a8b8 e2a6 h7c7 f1f6 f8h6 f6g6 h6e3 g1f1 g8f8 g6f6 f8f6 a6f6 c7f7 f6f7 e8f7 d4c6";


    
}