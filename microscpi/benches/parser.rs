use criterion::{criterion_group, criterion_main, Criterion};
use microscpi::tokens::{ScanResult, Tokenizer};

fn bench_tree_lookup(c: &mut Criterion) {
    let test = b"A:REALLY:DEEP:PATH:INSIDE:THE:COMMAND:TREE?";
    let data = &test[..];
    let mut tokenizer = Tokenizer::new(data);

    c.bench_function("benches", |b| {
        b.iter(|| {
            std::hint::black_box(
                while let ScanResult::Ok(_token) = tokenizer.next_token() {}
           );
        });
    });

    if data.len() > 0 {
        panic!("Not all data has been read!");
    }
}

criterion_group!(
    benches,
    bench_tree_lookup,
);
criterion_main!(benches);