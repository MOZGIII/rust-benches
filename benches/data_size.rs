use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use futures::{SinkExt, StreamExt};

fn benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_size");
    for &(batches, batch_size, time_mul) in &[
        (1_000, 1, 1),
        (1_000, 10, 1),
        (1_000, 100, 1),
        (100_000, 1, 1),
        (100_000, 10, 1),
        (100_000, 100, 1),
    ] {
        group.noise_threshold(0.01);
        group.measurement_time(Duration::from_secs(10 * time_mul));
        group.warm_up_time(Duration::from_secs(3 * time_mul));
        group.throughput(Throughput::Elements(batches as u64 * batch_size as u64));
        group.bench_with_input(
            BenchmarkId::new("tokio", format!("{}x{}", batches, batch_size)),
            &(batches, batch_size),
            |b, &(batches, batch_size)| {
                b.iter_with_setup(
                    || {
                        (
                            tokio::sync::mpsc::channel(1),
                            make_data(batches, batch_size),
                        )
                    },
                    |((tx, mut rx), input)| {
                        futures::executor::block_on(async {
                            for item in input {
                                tx.send(item).await.unwrap();
                                let data = rx.recv().await.unwrap();
                                process_data(data.into_iter());
                            }
                        });
                    },
                );
            },
        );
        group.bench_with_input(
            BenchmarkId::new("futures", format!("{}x{}", batches, batch_size)),
            &(batches, batch_size),
            |b, &(batches, batch_size)| {
                b.iter_with_setup(
                    || {
                        (
                            futures::channel::mpsc::channel(1),
                            make_data(batches, batch_size),
                        )
                    },
                    |((mut tx, mut rx), input)| {
                        futures::executor::block_on(async {
                            for item in input {
                                tx.send(item).await.unwrap();
                                let data = rx.next().await.unwrap();
                                process_data(data.into_iter());
                            }
                        });
                    },
                );
            },
        );
    }
    group.finish();
}

fn make_data(batches: usize, batch_size: usize) -> Vec<Vec<usize>> {
    (0..batches)
        .map(|_| black_box(vec![0; batch_size]))
        .collect()
}

fn process_data(data: impl Iterator<Item = usize>) -> usize {
    data.sum()
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
