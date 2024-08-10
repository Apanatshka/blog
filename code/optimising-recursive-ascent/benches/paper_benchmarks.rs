use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use optimizing_directly_executable_lr_parsers::paper;

pub fn parse(c: &mut Criterion) {
    let sample_input = "a+a*(a+a)*a";
    c.bench_with_input(
        BenchmarkId::new("parse", sample_input),
        &sample_input,
        |b, &str| b.iter(||
        paper::parse(&mut str.chars().peekable()).expect("sample_input should parse just fine")));
    c.bench_with_input(
        BenchmarkId::new("parse_reverse_goto", sample_input),
        &sample_input,
        |b, &str| b.iter(||
        paper::parse_reverse_goto(&mut str.chars().peekable()).expect("sample_input should parse_reverse_goto just fine")));
    c.bench_with_input(
        BenchmarkId::new("parse_chain_elim", sample_input),
        &sample_input,
        |b, &str| b.iter(||
        paper::parse_chain_elim(&mut str.chars().peekable()).expect("sample_input should parse_chain_elim just fine")));
    c.bench_with_input(
        BenchmarkId::new("parse_minpush", sample_input),
        &sample_input,
        |b, &str| b.iter(||
        paper::parse_minpush(&mut str.chars().peekable()).expect("sample_input should parse_minpush just fine")));
    c.bench_with_input(
        BenchmarkId::new("parse_max_inline", sample_input),
        &sample_input,
        |b, &str| b.iter(||
        paper::parse_max_inline(&mut str.chars().peekable()).expect("sample_input should parse_max_inline just fine")));
}

criterion_group!(benches, parse);
criterion_main!(benches);
