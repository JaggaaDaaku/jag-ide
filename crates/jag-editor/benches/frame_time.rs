// File: crates/jag-editor/benches/frame_time.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_gutter_render(c: &mut Criterion) {
    c.bench_function("gutter_baseline", |b| {
        b.iter(|| black_box(1000))
    });
}

criterion_group!(benches, benchmark_gutter_render);
criterion_main!(benches);
