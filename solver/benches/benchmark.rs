use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use solver::{generate_random_pg, solve, Algorithm};
use std::env;

fn bench_solvers(c: &mut Criterion) {
    let mut group = c.benchmark_group("Parity Game Solvers");
    
    let sizes: Vec<usize> = env::var("BENCH_SIZES")
        .map(|s| s.split(',').filter_map(|n| n.parse().ok()).collect())
        .unwrap_or_else(|_| vec![50, 100]);

    for size in sizes.iter() {
        let game = generate_random_pg(*size, 4, *size, Some(42));

        group.bench_with_input(BenchmarkId::new("Zielonka", size), &game, |b, g| {
            b.iter(|| {
                let _ = solve(black_box(g), Algorithm::Zielonka).unwrap();
            })
        });

        group.bench_with_input(BenchmarkId::new("Unoptimized Zielonka", size), &game, |b, g| {
            b.iter(|| {
                let _ = solve(black_box(g), Algorithm::UnoptimizedZielonka).unwrap();
            })
        });

        group.bench_with_input(BenchmarkId::new("FPI", size), &game, |b, g| {
            b.iter(|| {
                let _ = solve(black_box(g), Algorithm::Fpi).unwrap();
            })
        });

        group.bench_with_input(BenchmarkId::new("FPJ", size), &game, |b, g| {
            b.iter(|| {
                let _ = solve(black_box(g), Algorithm::Fpj).unwrap();
            })
        });

        group.bench_with_input(BenchmarkId::new("TL", size), &game, |b, g| {
            b.iter(|| {
                let _ = solve(black_box(g), Algorithm::Tl).unwrap();
            })
        });

        group.bench_with_input(BenchmarkId::new("PTL", size), &game, |b, g| {
            b.iter(|| {
                let _ = solve(black_box(g), Algorithm::Ptl).unwrap();
            })
        });

        group.bench_with_input(BenchmarkId::new("SI", size), &game, |b, g| {
            b.iter(|| {
                let _ = solve(black_box(g), Algorithm::Si).unwrap();
            })
        });

        group.bench_with_input(BenchmarkId::new("SPM", size), &game, |b, g| {
            b.iter(|| {
                let _ = solve(black_box(g), Algorithm::Spm).unwrap();
            })
        });
    }

    group.finish();
}

criterion_group!(benches, bench_solvers);

criterion_main!(benches);