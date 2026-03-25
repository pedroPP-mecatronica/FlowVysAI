//! Binary STL parser benchmark — validates Sprint S2 throughput targets.
//!
//! Measures:
//! - Parse throughput in **MB/s** (bytes / elapsed).
//! - Parse throughput in **Mtriangles/s** (triangles / elapsed).
//!
//! Run with:
//! ```sh
//! cargo bench -p aerocore_core --bench stl_bench
//! ```

use std::io::Write;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use aerocore_core::io_mesh::stl::{load_stl, parse_stl_binary};

// ── helpers ───────────────────────────────────────────────────────────────────

/// Builds a binary STL blob with `n` triangles (in-memory, no disk I/O).
fn make_binary_stl(n: usize) -> Vec<u8> {
    // 80-byte header + 4-byte count + n * 50 bytes per triangle
    let mut buf = Vec::with_capacity(84 + n * 50);
    buf.extend_from_slice(&[0u8; 80]); // header
    buf.extend_from_slice(&(n as u32).to_le_bytes());
    for i in 0..n {
        let x = i as f32;
        // normal (0, 0, 1)
        buf.extend_from_slice(&0.0_f32.to_le_bytes());
        buf.extend_from_slice(&0.0_f32.to_le_bytes());
        buf.extend_from_slice(&1.0_f32.to_le_bytes());
        // v0
        buf.extend_from_slice(&x.to_le_bytes());
        buf.extend_from_slice(&0.0_f32.to_le_bytes());
        buf.extend_from_slice(&0.0_f32.to_le_bytes());
        // v1
        buf.extend_from_slice(&(x + 1.0).to_le_bytes());
        buf.extend_from_slice(&0.0_f32.to_le_bytes());
        buf.extend_from_slice(&0.0_f32.to_le_bytes());
        // v2
        buf.extend_from_slice(&x.to_le_bytes());
        buf.extend_from_slice(&1.0_f32.to_le_bytes());
        buf.extend_from_slice(&0.0_f32.to_le_bytes());
        // attribute byte count
        buf.extend_from_slice(&0u16.to_le_bytes());
    }
    buf
}

// ── in-memory parse benchmark ─────────────────────────────────────────────────

/// Benchmarks `parse_stl_binary` on an in-memory `&[u8]` at several sizes.
/// This isolates nom parsing from disk I/O.
fn bench_parse_stl_binary(c: &mut Criterion) {
    let mut group = c.benchmark_group("stl/parse_binary");

    for &n_tris in &[1_000usize, 10_000, 100_000, 500_000] {
        let data = make_binary_stl(n_tris);
        let bytes = data.len() as u64;
        group.throughput(Throughput::Bytes(bytes));

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{n_tris}_tris")),
            &data,
            |b, data| {
                b.iter(|| {
                    let mesh = parse_stl_binary(black_box(data)).expect("parse failed");
                    black_box(mesh);
                });
            },
        );
    }
    group.finish();
}

// ── mmap load+parse benchmark ─────────────────────────────────────────────────

/// Benchmarks `load_stl` (memmap2-backed) on a temporary file.
/// Measures end-to-end mmap open + parse throughput.
fn bench_load_stl_mmap(c: &mut Criterion) {
    let mut group = c.benchmark_group("stl/load_mmap");

    for &n_tris in &[10_000usize, 100_000, 500_000] {
        let data = make_binary_stl(n_tris);
        let bytes = data.len() as u64;
        group.throughput(Throughput::Bytes(bytes));

        // Write once to a temp file; reuse the path across iterations.
        let mut tmp = tempfile::NamedTempFile::new().expect("temp file");
        tmp.write_all(&data).expect("write");
        tmp.flush().expect("flush");
        let path = tmp.path().to_path_buf();

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{n_tris}_tris")),
            &path,
            |b, p| {
                b.iter(|| {
                    let mesh = load_stl(black_box(p)).expect("load failed");
                    black_box(mesh);
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_parse_stl_binary, bench_load_stl_mmap);
criterion_main!(benches);
