use criterion::{criterion_group, criterion_main, Criterion};





criterion_group!(benches, bench_setup);
criterion_main!(benches);