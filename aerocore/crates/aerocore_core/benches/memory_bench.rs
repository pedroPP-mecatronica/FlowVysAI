//! Memory subsystem benchmarks — validates KR 1.1 allocation performance.

use criterion::{criterion_group, criterion_main, Criterion, black_box};
use aerocore_core::memory::arena::SimArena;
use aerocore_core::memory::pool::ObjectPool;

fn bench_arena_alloc(c: &mut Criterion) {
    let arena = SimArena::new(1024 * 1024 * 100); // 100 MB

    c.bench_function("arena_alloc_1000_f64", |b| {
        b.iter(|| {
            let slice = arena.alloc_slice(1000, black_box(0.0_f64));
            black_box(slice);
        });
    });
}

fn bench_pool_acquire_release(c: &mut Criterion) {
    c.bench_function("pool_acquire_release_cycle", |b| {
        let mut pool: ObjectPool<f64> = ObjectPool::with_capacity(10_000);
        b.iter(|| {
            let h = pool.acquire(black_box(42.0));
            let val = pool.release(h);
            black_box(val);
        });
    });
}

criterion_group!(benches, bench_arena_alloc, bench_pool_acquire_release);
criterion_main!(benches);
