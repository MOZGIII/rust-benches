use criterion::{criterion_group, criterion_main, Criterion};

async fn test_future() {
    ()
}

fn benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("block_on");
    group.bench_function("futures::executor::block_on", |b| {
        b.iter(|| futures::executor::block_on(test_future()));
    });
    group.bench_function("tokio::runtime::Runtime::block_on", |b| {
        b.iter_with_setup(
            || tokio::runtime::Runtime::new().expect("unable to build tokio runtime"),
            |rt| rt.block_on(test_future()),
        );
    });
    group.bench_function("async_std::task::block_on", |b| {
        b.iter(|| async_std::task::block_on(test_future()));
    });
    group.bench_function("smol::block_on", |b| {
        b.iter(|| smol::block_on(test_future()));
    });
    group.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
