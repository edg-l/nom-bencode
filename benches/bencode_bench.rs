use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nom_bencode::*;

static SOURCE_BYTES_22KB: &[u8] = include_bytes!("../test-assets/big-buck-bunny.torrent");
static SOURCE_BYTES_113KB: &[u8] = include_bytes!("../test-assets/private.torrent");
static SOURCE_BYTES_218KB: &[u8] = include_bytes!("../test-assets/multi-file.torrent");

fn parse_source(src: &[u8]) -> Result<Vec<Value>, Err<BencodeError<&[u8]>>> {
    nom_bencode::parse(src)
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("bencode torrent 22kb", |b| {
        b.iter(|| parse_source(black_box(SOURCE_BYTES_22KB)))
    });
    c.bench_function("bencode torrent 113kb", |b| {
        b.iter(|| parse_source(black_box(SOURCE_BYTES_113KB)))
    });
    c.bench_function("bencode torrent 218kb", |b| {
        b.iter(|| parse_source(black_box(SOURCE_BYTES_218KB)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
