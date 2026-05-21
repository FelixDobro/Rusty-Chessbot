use chess_bot::chess::Game;
use chess_bot::chess::board::Board;
use chess_bot::chess::board::bitboard::init_lazylocks;
use criterion::BatchSize;
use criterion::{Criterion, criterion_group};
use rayon::prelude::*;

pub fn perft_copy(game: &mut Game, board: &mut Board, depth: u8) -> usize {
    if depth == 0 {
        return 1;
    }

    let nodes = board
        .generate_pseudolegals()
        .as_slice()
        .iter()
        .map(|&m| {
            if let Some(mut new_board) = board.make_pl_move_copy(m) {
                let nodes = perft_copy(game, &mut new_board, depth - 1);
                game.pop_only_state(&new_board);
                return nodes;
            }
            
            0
        })
        .sum();

    return nodes;
}

fn perft(game: &mut Game, depth: u8) -> usize {
    if depth == 0 {
        return 1;
    }

    let total_nodes: usize = game
        .generate_pseudolegals()
        .as_slice()
        .iter()
        .map(|&m| {
            if game.make_pl_move(m) {
                let nodes = perft(game, depth - 1);
                game.unmake_pl_move(m);
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
            || {
                init_lazylocks();
                Game::default()
            },             
            |mut game| perft(&mut game, 5), 
            BatchSize::SmallInput,
        );
    });
}

fn default_perft_copy_depth_5(c: &mut Criterion) {
    c.bench_function("default_perft_copy_depth_5", |b| {
        b.iter_batched(
            || {
                init_lazylocks();
                (Game::default(), Board::default())
            }, 
            |(mut game, mut board)| perft_copy(&mut game, &mut board, 5), 
            BatchSize::SmallInput,
        );
    });
}

fn default_perft_depth_6(c: &mut Criterion) {
    c.bench_function("default_perft_depth_6", |b| {
        b.iter_batched(
            || {
                init_lazylocks();
                Game::default()
            },             
            |mut game| perft(&mut game, 6), 
            BatchSize::SmallInput,
        );
    });
}

fn default_perft_copy_depth_6(c: &mut Criterion) {
    c.bench_function("default_perft_copy_depth_6", |b| {
        b.iter_batched(
            || {
                init_lazylocks();
                (Game::default(), Board::default())
            }, 
            |(mut game, mut board)| perft_copy(&mut game, &mut board, 6), 
            BatchSize::SmallInput,
        );
    });
}


criterion_group!(
    name = perft_bench;
    config = perft_criterion();
    targets = default_perft_depth_5, default_perft_depth_6, default_perft_copy_depth_5, default_perft_copy_depth_6
);
