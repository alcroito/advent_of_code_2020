use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    let input = "class: 0-1 or 4-19
row: 0-5 or 8-19
seat: 0-13 or 16-19

your ticket:
11,12,13

nearby tickets:
3,9,18
15,1,5
5,14,9";
    let mut s = advent::d16_lib::parse_document(input);
    advent::d16_lib::remove_invalid_tickets(&mut s);
    c.bench_with_input(BenchmarkId::new("deduce_fields", 4), &s, |b, i| {
        b.iter(|| advent::d16_lib::deduce_fields(i))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
