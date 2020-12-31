use std::time::Duration;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use futures::{SinkExt, StreamExt};

async fn tokio_runtime_tokio_channels(amount: usize, depth: usize) {
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    for _ in 0..depth {
        let (new_tx, new_rx) = tokio::sync::mpsc::channel(1);
        tokio::spawn(async move {
            loop {
                let val = match rx.recv().await {
                    Some(val) => val,
                    None => break,
                };
                new_tx.send(val).await.unwrap();
            }
        });
        rx = new_rx;
    }

    tokio::spawn(async move {
        for num in 0..amount {
            tx.send(num).await.unwrap();
        }
    });
    for num in 0..amount {
        let val = rx.recv().await;
        assert_eq!(num, val.unwrap());
    }
    assert!(rx.recv().await.is_none());
}

async fn tokio_runtime_futures_channels(amount: usize, depth: usize) {
    let (mut tx, mut rx) = futures::channel::mpsc::channel(0);
    for _ in 0..depth {
        let (mut new_tx, new_rx) = futures::channel::mpsc::channel(0);
        tokio::spawn(async move {
            loop {
                let val = match rx.next().await {
                    Some(val) => val,
                    None => break,
                };
                new_tx.send(val).await.unwrap();
            }
        });
        rx = new_rx;
    }

    tokio::spawn(async move {
        for num in 0..amount {
            tx.send(num).await.unwrap();
        }
    });
    for num in 0..amount {
        let val = rx.next().await;
        assert_eq!(num, val.unwrap());
    }
    assert!(rx.next().await.is_none());
}

fn benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("channels_cascade");
    for &(amount, depth, time_mul) in &[
        (10_000, 0, 1),
        (100_000, 0, 10),
        (10_000, 1, 1),
        (10_000, 2, 1),
        (10_000, 10, 1),
        (100_000, 10, 10),
        (10_000, 50, 1),
        (10_000, 100, 1),
    ] {
        group.noise_threshold(0.025);
        group.measurement_time(Duration::from_secs(40 * time_mul));
        group.warm_up_time(Duration::from_secs(6 * time_mul));
        group.throughput(Throughput::Elements(amount as u64));
        group.bench_with_input(
            BenchmarkId::new(
                "tokio_runtime/tokio_channels",
                format!("depth-{}/amount-{}", depth, amount),
            ),
            &(amount, depth),
            |b, &(amount, depth)| {
                b.iter_with_setup(
                    || tokio::runtime::Runtime::new().expect("unable to build tokio runtime"),
                    |rt| rt.block_on(tokio_runtime_tokio_channels(amount, depth)),
                );
            },
        );
        group.bench_with_input(
            BenchmarkId::new(
                "tokio_runtime/futures_channels",
                format!("{}/{}", depth, amount),
            ),
            &(amount, depth),
            |b, &(amount, depth)| {
                b.iter_with_setup(
                    || tokio::runtime::Runtime::new().expect("unable to build tokio runtime"),
                    |rt| rt.block_on(tokio_runtime_futures_channels(amount, depth)),
                );
            },
        );
    }
    group.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
