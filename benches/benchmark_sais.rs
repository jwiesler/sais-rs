use std::fs::File;
use std::io::Read;
use std::ops::AddAssign;
use std::time::{Duration, Instant};

use criterion::{criterion_group, criterion_main, Criterion};

use sais_rs::sort;

fn sort_benchmark(c: &mut Criterion) {
    const FILES: &[&str] = &[
        "gauntlet_corpus/abac",
        "gauntlet_corpus/abba",
        "gauntlet_corpus/book1x20",
        "gauntlet_corpus/fib_s14730352",
        "gauntlet_corpus/fss9",
        "gauntlet_corpus/fss10",
        "gauntlet_corpus/houston",
        "gauntlet_corpus/paper5x80",
        "gauntlet_corpus/test1",
        "gauntlet_corpus/test2",
        "gauntlet_corpus/test3",
    ];

    for &name in FILES {
        let mut text = Vec::new();
        File::open(name).unwrap().read_to_end(&mut text).unwrap();

        c.bench_function(&format!("sais-{}", name), |b| {
            b.iter_custom(|iterations| {
                let mut duration = Duration::from_secs(0);
                let mut indices = vec![Default::default(); text.len()];
                let mut types = vec![Default::default(); text.len()];
                let mut buckets = vec![0u32; 256];
                for _ in 0..iterations {
                    indices.fill(Default::default());
                    types.fill(Default::default());
                    buckets.resize(256, Default::default());
                    buckets.fill(Default::default());

                    let start = Instant::now();
                    sort(&text, &mut indices, &mut types, &mut buckets);
                    duration.add_assign(start.elapsed())
                }
                duration
            })
        });
    }
}

criterion_group!(
    name = benches;
    config = Criterion::default().warm_up_time(Duration::from_secs(10)).measurement_time(Duration::from_secs(20));
    targets = sort_benchmark
);
criterion_main!(benches);
