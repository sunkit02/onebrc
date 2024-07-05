use criterion::{black_box, criterion_group, criterion_main, Criterion};
use onebrc::custom_parse_float;

fn criterion_benchmark(c: &mut Criterion) {
    let file_content = std::fs::read_to_string("./data/head_one_mil_temps.txt").unwrap();
    let float_bytes = file_content.split('\n').collect::<Vec<&str>>();

    c.bench_function("std_parse_float head_one_mil", |b| {
        black_box(b.iter(|| {
            float_bytes
                .iter()
                .map(|&float_str| float_str.parse::<f64>().unwrap())
        }))
    });

    let float_bytes = file_content
        .split('\n')
        .map(|s| s.as_bytes())
        .collect::<Vec<&[u8]>>();

    c.bench_function("custom_parse_float head_one_mil", |b| {
        black_box(b.iter(|| float_bytes.iter().map(|&bytes| custom_parse_float(bytes))))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
