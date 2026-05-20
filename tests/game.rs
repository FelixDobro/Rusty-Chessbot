use chess_bot::chess::square::Square;
use chess_bot::chess::{chess_move::Move,Game};
use chess_bot::chess::board::Board;

#[test]
fn qualify_moves_1() {
    let mut board = Board::default();

    let res1 = board.qualify_move("e2e4");
    assert!(res1.is_ok(), "Move e2e4 failed to qualify");
    let m1 = res1.unwrap();
    assert_eq!(m1, Move::new(Square::E2, Square::E4, Move::DOUBLE_PAWN));
    board.make_pl_move(m1);

    let res2 = board.qualify_move("e7e5");
    assert!(res2.is_ok(), "Move e7e5 failed to qualify");
    let m2 = res2.unwrap();
    assert_eq!(m2, Move::new(Square::E7, Square::E5, Move::DOUBLE_PAWN));
    board.make_pl_move(m2);

    let res3 = board.qualify_move("g1f3");
    assert!(res3.is_ok(), "Move g1f3 failed to qualify");
    let m3 = res3.unwrap();
    assert_eq!(m3, Move::new(Square::G1, Square::F3, Move::QUIET));
    board.make_pl_move(m3);

    let res4 = board.qualify_move("d8g5");
    assert!(res4.is_ok(), "Move d8g5 failed to qualify");
    let m4 = res4.unwrap();
    assert_eq!(m4, Move::new(Square::D8, Square::G5, Move::QUIET));
    board.make_pl_move(m4);

    let res5 = board.qualify_move("f3g5");
    assert!(res5.is_ok(), "Move f3g5 failed to qualify");
    let m5 = res5.unwrap();
    assert_eq!(m5, Move::new(Square::F3, Square::G5, Move::CAPTURE));
    board.make_pl_move(m5);

    let res6 = board.qualify_move("f8c5");
    assert!(res6.is_ok(), "Move f8c5 failed to qualify");
    let m6 = res6.unwrap();
    assert_eq!(m6, Move::new(Square::F8, Square::C5, Move::QUIET));
    board.make_pl_move(m6);

    let res7 = board.qualify_move("f1c4");
    assert!(res7.is_ok(), "Move f1c4 failed to qualify");
    let m7 = res7.unwrap();
    assert_eq!(m7, Move::new(Square::F1, Square::C4, Move::QUIET));
    board.make_pl_move(m7);

    let res8 = board.qualify_move("g8f6");
    assert!(res8.is_ok(), "Move g8f6 failed to qualify");
    let m8 = res8.unwrap();
    assert_eq!(m8, Move::new(Square::G8, Square::F6, Move::QUIET));
    board.make_pl_move(m8);

    let res9 = board.qualify_move("e1g1");
    assert!(res9.is_ok(), "Move e1g1 failed to qualify");
    let m9 = res9.unwrap();
    assert_eq!(m9, Move::new(Square::E1, Square::G1, Move::KING_CASTLE));
    board.make_pl_move(m9);

    let res10 = board.qualify_move("e8g8");
    assert!(res10.is_ok(), "Move e8g8 failed to qualify");
    let m10 = res10.unwrap();
    assert_eq!(m10, Move::new(Square::E8, Square::G8, Move::KING_CASTLE));
    board.make_pl_move(m10);

    let res11 = board.qualify_move("d2d4");
    assert!(res11.is_ok(), "Move d2d4 failed to qualify");
    let m11 = res11.unwrap();
    assert_eq!(m11, Move::new(Square::D2, Square::D4, Move::DOUBLE_PAWN));
    board.make_pl_move(m11);

    let res12 = board.qualify_move("c5b6");
    assert!(res12.is_ok(), "Move c5b6 failed to qualify");
    let m12 = res12.unwrap();
    assert_eq!(m12, Move::new(Square::C5, Square::B6, Move::QUIET));
    board.make_pl_move(m12);

    let res13 = board.qualify_move("d4d5");
    assert!(res13.is_ok(), "Move d4d5 failed to qualify");
    let m13 = res13.unwrap();
    assert_eq!(m13, Move::new(Square::D4, Square::D5, Move::QUIET));
    board.make_pl_move(m13);

    let res14 = board.qualify_move("c7c5");
    assert!(res14.is_ok(), "Move c7c5 failed to qualify");
    let m14 = res14.unwrap();
    assert_eq!(m14, Move::new(Square::C7, Square::C5, Move::DOUBLE_PAWN));
    board.make_pl_move(m14);

    let res15 = board.qualify_move("d5c6");
    assert!(res15.is_ok(), "Move d5c6 failed to qualify");
    let m15 = res15.unwrap();
    assert_eq!(m15, Move::new(Square::D5, Square::C6, Move::EN_PASSANT));
    board.make_pl_move(m15);

    let res16 = board.qualify_move("a7a6");
    assert!(res16.is_ok(), "Move a7a6 failed to qualify");
    let m16 = res16.unwrap();
    assert_eq!(m16, Move::new(Square::A7, Square::A6, Move::QUIET));
    board.make_pl_move(m16);

    let res17 = board.qualify_move("c6c7");
    assert!(res17.is_ok(), "Move c6c7 failed to qualify");
    let m17 = res17.unwrap();
    assert_eq!(m17, Move::new(Square::C6, Square::C7, Move::QUIET));
    board.make_pl_move(m17);

    let res18 = board.qualify_move("a8a7");
    assert!(res18.is_ok(), "Move a8a7 failed to qualify");
    let m18 = res18.unwrap();
    assert_eq!(m18, Move::new(Square::A8, Square::A7, Move::QUIET));
    board.make_pl_move(m18);

    let res19 = board.qualify_move("c7b8n");
    assert!(res19.is_ok(), "Move c7b8n failed to qualify");
    let m19 = res19.unwrap();
    assert_eq!(m19, Move::new(Square::C7, Square::B8, Move::PROMO_CAP_KNIGHT));
    board.make_pl_move(m19);
}



#[test]
fn qualify_moves_2() {
    let mut board = Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap();

    let res1 = board.qualify_move("d5d6");
    assert!(res1.is_ok(), "Move d5d6 failed to qualify");
    let m1 = res1.unwrap();
    assert_eq!(m1, Move::new(Square::D5, Square::D6, Move::QUIET));
    board.make_pl_move(m1);

    let res2 = board.qualify_move("b4c3");
    assert!(res2.is_ok(), "Move b4c3 failed to qualify");
    let m2 = res2.unwrap();

    assert_eq!(m2, Move::new(Square::B4, Square::C3, Move::CAPTURE));
    board.make_pl_move(m2);

    let res3 = board.qualify_move("d6c7");
    assert!(res3.is_ok(), "Move d6c7 failed to qualify");
    let m3 = res3.unwrap();
    assert_eq!(m3, Move::new(Square::D6, Square::C7, Move::CAPTURE));
    board.make_pl_move(m3);

    let res4 = board.qualify_move("c3b2");
    assert!(res4.is_ok(), "Move c3b2 failed to qualify");
    let m4 = res4.unwrap();
    assert_eq!(m4, Move::new(Square::C3, Square::B2, Move::CAPTURE));
    board.make_pl_move(m4);

    let res5 = board.qualify_move("c7c8n");
    assert!(res5.is_ok(), "Move c7c8n failed to qualify");
    let m5 = res5.unwrap();
    assert_eq!(m5, Move::new(Square::C7, Square::C8, Move::PROMO_KNIGHT));
    board.make_pl_move(m5);

    let res6 = board.qualify_move("b2b1b");
    assert!(res6.is_ok(), "Move b2b1b failed to qualify");
    let m6 = res6.unwrap();
    assert_eq!(m6, Move::new(Square::B2, Square::B1, Move::PROMO_BISHOP));
    board.make_pl_move(m6);


}
