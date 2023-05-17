use criterion::{black_box, criterion_group, criterion_main, Criterion};
use graph_walker::universe::{universe_3d::Universe3D, Universe, Universe2D};

fn tick_1_benchmark_2d(c: &mut Criterion) {
    let mut universe = black_box(Universe2D::new(100, 100000));

    c.bench_function("tick algorithm 1 iter 2d", |b| b.iter(|| universe.tick()));
}

fn tick_1_benchmark_3d(c: &mut Criterion) {
    let mut universe = black_box(Universe3D::new(100, 100000));

    c.bench_function("tick algorithm 1 iter 3d", |b| b.iter(|| universe.tick()));
}

fn tick_300_benchmark_2d(c: &mut Criterion) {
    let mut universe = black_box(Universe2D::new(100, 100000));

    c.bench_function("tick algorithm 300 iter 2d", |b| {
        b.iter(|| {
            for _ in 0..300 {
                universe.tick()
            }
        })
    });
}

fn tick_300_benchmark_3d(c: &mut Criterion) {
    let mut group = c.benchmark_group("tick algorithm 300 iter");

    let mut universe2D = black_box(Universe2D::new(100, 100000));
    let mut universe3D = black_box(Universe3D::new(100, 100000));

    group.sample_size(10);
    group.bench_function("2d", |b| b.iter(|| universe2D.iterate(300)));
    group.bench_function("3d", |b| b.iter(|| universe3D.iterate(300)));
    group.finish();
}

criterion_group!(
    benches,
    tick_1_benchmark_2d,
    tick_1_benchmark_3d,
    tick_300_benchmark_2d,
    tick_300_benchmark_3d
);
criterion_main!(benches);
