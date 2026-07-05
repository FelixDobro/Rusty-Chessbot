use criterion::criterion_main;

mod perft;
mod make_move;



criterion_main!
(
    make_move::make_move_unmake,
    perft::perft_bench,

);