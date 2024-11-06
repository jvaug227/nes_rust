use criterion::{criterion_group, criterion_main, Criterion};

fn bench_lookup_table(c: &mut Criterion) {
    // let lookup_table = nes_rust::C6502::create_lookup_table();
    c.bench_function("bench_lookup_table", |b| {
        b.iter(|| {
            std::hint::black_box(for i in 1..=255usize {
                // let _s = lookup_table[i];
            });
        });
    });
}

criterion_group!(benches, bench_lookup_table);
criterion_main!(benches);
