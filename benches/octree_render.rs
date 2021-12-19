use conjure::{octree::Octree, shape::CsgFunc};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use pprof::criterion::{Output, PProfProfiler};

const BOUND: f32 = 256.0;
const RADIUS: f32 = 100.0;

fn sphere_shape(bound: f32, resolution: f32, csg_func: &CsgFunc) {
    let mut octree = Octree::new(-bound, bound);
    octree.render_shape(resolution, csg_func);
}

fn bench_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("octree_render");
    let csg_func = CsgFunc::new(Box::new(move |x, y, z| {
        ((0.0 - z).powi(2) + (0.0 - x).powi(2) + (0.0 - y).powi(2)).sqrt() - RADIUS
    }));
    for depth in [2, 4, 6, 8] {
        let resolution = BOUND / 2.0_f32.powi(depth);
        group.bench_with_input(BenchmarkId::from_parameter(depth), &resolution, |b, s| {
            b.iter(|| sphere_shape(BOUND / 2.0, black_box(*s), &csg_func))
        });
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = bench_group
}
criterion_main!(benches);
