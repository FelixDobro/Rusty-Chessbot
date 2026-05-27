
use chess_bot::chess::board::Board;
use chess_bot::chess::board::bitboard::init_lazylocks;
use chess_bot::search::{SearchAlgorithm, SearchLimits};
use chess_bot::search::simple_search::{Negamax, NegamaxTT};
use criterion::BatchSize;
use criterion::{Criterion, criterion_group};


fn perft_criterion() -> Criterion {
    Criterion::default().sample_size(20)
}

fn search_depth_8_negamax_tt(c: &mut Criterion) {
    c.bench_function("negamax_tt_depth_8", |b| {
        b.iter_batched(
            || {
                init_lazylocks();
                (NegamaxTT::new(2u64.pow(20) as usize), Board::default())
            },             
            |(mut negamax_tt, mut board)| negamax_tt.search(&mut board, &SearchLimits::depth(8)), 
            BatchSize::LargeInput,
        );
    });
}


fn search_depth_8_negamax(c: &mut Criterion) {
    c.bench_function("negamax_depth_8", |b| {
        b.iter_batched(
            || {
                init_lazylocks();
                (Negamax::new(), Board::default())
            },             
            |(mut negamax_tt, mut board)| negamax_tt.search(&mut board, &SearchLimits::depth(8)), 
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(
    name = search_bench;
    config = perft_criterion();
    targets = search_depth_8_negamax_tt, search_depth_8_negamax
);