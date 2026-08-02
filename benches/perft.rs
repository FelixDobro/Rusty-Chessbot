use chess_bot::chess::board::Board;
use criterion::BatchSize;
use criterion::{Criterion, criterion_group};

fn perft(board: &mut Board, depth: u8) -> usize {
    if depth == 0 {
        return 1;
    }

    let total_nodes: usize = board
        .generate_pseudolegals()
        .as_slice()
        .iter()
        .map(|&m| {
            if board.make_pl_move::<false>(m) {
                let nodes = perft(board, depth - 1);
                board.unmake_pl_move(m);
                return nodes;
            }
            0
        })
        .sum();
    total_nodes
}

fn perft_criterion() -> Criterion {
    Criterion::default().sample_size(20)
}

fn default_perft_depth_5(c: &mut Criterion) {
    c.bench_function("default_perft_depth_5", |b| {
        b.iter_batched(
            || Board::default(),
            |mut board| perft(&mut board, 5),
            BatchSize::SmallInput,
        );
    });
}

fn default_perft_depth_6(c: &mut Criterion) {
    c.bench_function("default_perft_depth_6", |b| {
        b.iter_batched(
            || Board::default(),
            |mut board| perft(&mut board, 6),
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(
    name = perft_bench;
    config = perft_criterion();
    targets = default_perft_depth_5, default_perft_depth_6
);
