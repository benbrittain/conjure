use conjure::{octree::Octree, shape::CsgFunc};
use criterion::{black_box, BenchmarkId, Criterion, criterion_main, criterion_group};
use pprof::criterion::{Output, PProfProfiler};

fn octree(resolution: f32) {
    let radius = 100.0;
    let csg_func = CsgFunc::new(Box::new(move |x, y, z| {
        (((0.0 - z) * (0.0 - z)) + ((0.0 - x) * (0.0 - x)) + ((0.0 - y) * (0.0 - y))).sqrt()
            - radius
    }));
    let mut octree = Octree::new(-128.0, 128.0);
    octree.render_shape(resolution, &csg_func);
}

fn bench_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("Octree resolutions");

    for s in &[0.5, 1.0, 2.0, 10.0, 20.0] {
        group.bench_with_input(BenchmarkId::from_parameter(s), s, |b, s| {
            b.iter(|| octree(black_box(*s)))
        });
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = bench_group
}
criterion_main!(benches);
