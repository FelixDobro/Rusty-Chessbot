use chess_bot::chess::*;
use chess_bot::chess::chess_move::Move;
use criterion::{Criterion, criterion_group};



fn make_unmake(game: &mut Game, m: Move) {
    game.make_pl_move(m);
    game.unmake_pl_move(m);
}

fn make_unmake_copy(game: &mut Game, m: Move) {
    let new_board = game.make_pl_move_copy(m).unwrap();
    game.pop_only_state(&new_board);
}

fn quiet_move_make_unmake(c: &mut Criterion) {
    c.bench_function("Make unmake quiet",
    |bencher| 
    bencher.iter_batched(
        || {
            let game = Game::default();
            let m = Move::from_string("e2e3", &game).unwrap();
            (game, m)
        },
        |(mut game, m)| make_unmake(&mut game, m),
        criterion::BatchSize::SmallInput)
    );
}

fn double_pawn_make_unmake(c: &mut Criterion) {
    c.bench_function("Make unmake Double pawn",
    |bencher| 
    bencher.iter_batched(
        || {
            let game = Game::default();
            let m = Move::from_string("e2e4", &game).unwrap();
            (game, m)
        },
        |(mut game, m)| make_unmake(&mut game, m),
        criterion::BatchSize::SmallInput)
    );
}

fn capture_make_unmake(c: &mut Criterion) {
    c.bench_function("Make unmake capture",
    |bencher| 
    bencher.iter_batched(
        || {
            let mut game = Game::default();
            let m = Move::from_string("e2e3", &game).unwrap();
            assert!(game.make_pl_move(m));
            let m1 = Move::from_string("b7b5", &game).unwrap();
            assert!(game.make_pl_move(m1));
            let m2 = Move::from_string("f1b5", &game).unwrap();

            (game, m2)
        },
        |(mut game, m)| make_unmake(&mut game, m),
        criterion::BatchSize::SmallInput)
    );
}


fn make_unmake_castle(c: &mut Criterion) {

    c.bench_function("Make unmake castle",
    |bencher| 
    bencher.iter_batched(
        || {
            let game = Game::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap();
            let castle = Move::from_string("e1g1", &game).unwrap();
            (game, castle)
        },
        |(mut game, m)| make_unmake(&mut game, m),
        criterion::BatchSize::SmallInput)
    );
}

fn make_unmake_promo(c: &mut Criterion) {

    c.bench_function("Make unmake promo",
    |bencher| 
    bencher.iter_batched(
        || {
            let game = Game::from_fen("5k2/4P3/5K2/8/8/8/8/8 w - - 0 1").unwrap();
            let promotion = Move::from_string("e7e8q", &game).unwrap();
            (game, promotion)
        },
        |(mut game, m)| make_unmake(&mut game, m),
        criterion::BatchSize::SmallInput)
    );

}



fn make_unmake_copy_quiet(c: &mut Criterion) {

    c.bench_function("Make unmake copy quiet",
    |bencher| 
    bencher.iter_batched(
        || {
            let game = Game::default();
            let m = Move::from_string("e2e3", &game).unwrap();
            (game, m)
        },
        |(mut game, m)| make_unmake_copy(&mut game, m),
        criterion::BatchSize::SmallInput)
    );

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


criterion_group!(
    name = make_move_copy_unmake;
    config = Criterion::default();
    targets = make_unmake_copy_quiet,
);
