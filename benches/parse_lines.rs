use std::{path::PathBuf, str::FromStr};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use onebrc::process;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parse_lines head_one_mil", |b| {
        b.iter(|| {
            let path = PathBuf::from_str("./data/head_one_mil.txt").unwrap();
            black_box(process(path));
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
