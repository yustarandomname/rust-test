use criterion::{black_box, criterion_group, criterion_main, Criterion};
use graph_walker::Universe2D;

fn tick_1_benchmark(c: &mut Criterion) {
    let mut universe = black_box(Universe2D::new(100, 100000));

    c.bench_function("tick algorithm 1 iter", |b| b.iter(|| universe.tick()));
}

fn tick_300_benchmark(c: &mut Criterion) {
    let mut universe = black_box(Universe2D::new(100, 100000));

    c.bench_function("tick algorithm 300 iter", |b| {
        b.iter(|| {
            for _ in 0..300 {
                universe.tick()
            }
        })
    });
}

criterion_group!(benches, tick_1_benchmark, tick_300_benchmark);
criterion_main!(benches);
