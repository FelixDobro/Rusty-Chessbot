use chess_bot::chess::board::Board;
use chess_bot::chess::board::bitboard::init_lazylocks;
use chess_bot::chess::chess_move::Move;
use criterion::{Criterion, criterion_group};

fn make_unmake(board: &mut Board, m: Move) {
    board.make_pl_move::<true>(m);
    board.unmake_pl_move(m);
}

fn quiet_move_make_unmake(c: &mut Criterion) {
    c.bench_function("Make unmake quiet", |bencher| {
        bencher.iter_batched(
            || {
                init_lazylocks();
                let board = Board::default();
                let m = Move::from_string("e2e3", &board).unwrap();
                (board, m)
            },
            |(mut board, m)| make_unmake(&mut board, m),
            criterion::BatchSize::SmallInput,
        )
    });
}

fn double_pawn_make_unmake(c: &mut Criterion) {
    c.bench_function("Make unmake Double pawn", |bencher| {
        bencher.iter_batched(
            || {
                init_lazylocks();
                let board = Board::default();
                let m = Move::from_string("e2e4", &board).unwrap();
                (board, m)
            },
            |(mut board, m)| make_unmake(&mut board, m),
            criterion::BatchSize::SmallInput,
        )
    });
}

fn capture_make_unmake(c: &mut Criterion) {
    c.bench_function("Make unmake capture", |bencher| {
        bencher.iter_batched(
            || {
                init_lazylocks();
                let mut board = Board::default();
                let m = Move::from_string("e2e3", &board).unwrap();
                assert!(board.make_pl_move::<true>(m));
                let m1 = Move::from_string("b7b5", &board).unwrap();
                assert!(board.make_pl_move::<true>(m1));
                let m2 = Move::from_string("f1b5", &board).unwrap();

                (board, m2)
            },
            |(mut board, m)| make_unmake(&mut board, m),
            criterion::BatchSize::SmallInput,
        )
    });
}

fn make_unmake_castle(c: &mut Criterion) {
    c.bench_function("Make unmake castle", |bencher| {
        bencher.iter_batched(
            || {
                init_lazylocks();
                let board = Board::from_fen(
                    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
                )
                .unwrap();
                let castle = Move::from_string("e1g1", &board).unwrap();
                (board, castle)
            },
            |(mut board, m)| make_unmake(&mut board, m),
            criterion::BatchSize::SmallInput,
        )
    });
}

fn make_unmake_promo(c: &mut Criterion) {
    c.bench_function("Make unmake promo", |bencher| {
        bencher.iter_batched(
            || {
                init_lazylocks();
                let board = Board::from_fen("5k2/4P3/5K2/8/8/8/8/8 w - - 0 1").unwrap();
                let promotion = Move::from_string("e7e8q", &board).unwrap();
                (board, promotion)
            },
            |(mut board, m)| make_unmake(&mut board, m),
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(
    name = make_move_unmake;
    config = Criterion::default();
    targets =
    quiet_move_make_unmake,
    double_pawn_make_unmake,
    capture_make_unmake,
    make_unmake_castle,
    make_unmake_promo
);
