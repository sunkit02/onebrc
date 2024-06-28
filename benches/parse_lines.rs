use std::{
    fs::File,
    io::{BufReader, Seek},
};

use criterion::{criterion_group, criterion_main, Criterion};
use onebrc::{parse_lines, BUFFER_SIZE};

fn criterion_benchmark(c: &mut Criterion) {
    let file = File::open("./data/head_one_mil.txt").unwrap();
    c.bench_function("parse_lines head_one_mil", |b| {
        b.iter(|| {
            let mut file2 = file.try_clone().unwrap();
            let reader = BufReader::with_capacity(BUFFER_SIZE, file.try_clone().unwrap());
            parse_lines(reader);
            file2.seek(std::io::SeekFrom::Start(0)).unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
