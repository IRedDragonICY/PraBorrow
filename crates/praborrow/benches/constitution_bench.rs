use criterion::{Criterion, black_box, criterion_group, criterion_main};
use praborrow::core::CheckProtocol;
use praborrow::defense::Constitution;

#[derive(Constitution)]
struct BenchData {
    #[invariant(self.value >= 0)]
    value: i32,
    #[invariant(self.limit < 1000)]
    limit: i32,
}

fn bench_enforce_law(c: &mut Criterion) {
    let data = BenchData {
        value: 100,
        limit: 500,
    };

    // Warmup
    data.enforce_law().unwrap();

    c.bench_function("enforce_law_valid", |b| {
        b.iter(|| {
            black_box(&data).enforce_law().unwrap();
        })
    });
}

fn bench_enforce_law_failure(c: &mut Criterion) {
    let data = BenchData {
        value: -1,
        limit: 500,
    };

    c.bench_function("enforce_law_invalid", |b| {
        b.iter(|| {
            let result = black_box(&data).enforce_law();
            assert!(result.is_err());
        })
    });
}

criterion_group!(benches, bench_enforce_law, bench_enforce_law_failure);
criterion_main!(benches);
