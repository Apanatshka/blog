use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use optimizing_directly_executable_lr_parsers::{paper, parser};

pub fn parse(c: &mut Criterion) {
    let sample_input = "a+a*(a+a)*a";
    c.bench_with_input(
        BenchmarkId::new("parse_reverse_goto", sample_input),
        &sample_input,
        |b, &str| b.iter(||
        parser::parse_reverse_goto(&mut str.chars().peekable()).expect("sample_input should parse_reverse_goto just fine")));
    c.bench_with_input(
        BenchmarkId::new("parse_asc_desc", sample_input),
        &sample_input,
        |b, &str| b.iter(||
        parser::parse_asc_desc(&mut str.chars().peekable()).expect("sample_input should parse_asc_desc just fine")));
    c.bench_with_input(
        BenchmarkId::new("parse_push_first", sample_input),
        &sample_input,
        |b, &str| b.iter(||
        parser::parse_push_first(&mut str.chars().peekable()).expect("sample_input should parse_push_first just fine")));
    c.bench_with_input(
        BenchmarkId::new("parse_minpush", sample_input),
        &sample_input,
        |b, &str| b.iter(||
        parser::parse_minpush(&mut str.chars().peekable()).expect("sample_input should parse_minpush just fine")));
    c.bench_with_input(
        BenchmarkId::new("parse_inline1", sample_input),
        &sample_input,
        |b, &str| b.iter(||
        parser::parse_inline1(&mut str.chars().peekable()).expect("sample_input should parse_inline1 just fine")));
    c.bench_with_input(
        BenchmarkId::new("parse_inline2", sample_input),
        &sample_input,
        |b, &str| b.iter(||
        parser::parse_inline2(&mut str.chars().peekable()).expect("sample_input should parse_inline2 just fine")));
    c.bench_with_input(
        BenchmarkId::new("parse_single_input_next1", sample_input),
        &sample_input,
        |b, &str| b.iter(||
        parser::parse_single_input_next1(&mut str.chars().peekable()).expect("sample_input should parse_single_input_next1 just fine")));
    c.bench_with_input(
        BenchmarkId::new("parse_single_input_next", sample_input),
        &sample_input,
        |b, &str| b.iter(||
        parser::parse_single_input_next(&mut str.chars().peekable()).expect("sample_input should parse_single_input_next just fine")));
    c.bench_with_input(
        BenchmarkId::new("paper::parse_parser_struct", sample_input),
        &sample_input,
        |b, &str| b.iter(||
        paper::parse_parser_struct(&mut str.chars().peekable()).expect("sample_input should paper::parse_parser_struct just fine")));
    c.bench_with_input(
        BenchmarkId::new("paper::parse_single_match", sample_input),
        &sample_input,
        |b, &str| b.iter(||
        paper::parse_single_match(&mut str.chars().peekable()).expect("sample_input should paper::parse_single_match just fine")));
}

criterion_group!(benches, parse);
criterion_main!(benches);
