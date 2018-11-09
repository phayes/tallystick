#[macro_use]
extern crate criterion;
extern crate tallyman;

use criterion::Criterion;
use criterion::Benchmark;

fn condorcet(n: u64) {
    let mut tally = tallyman::condorcet::Tally::new(1);
    
    // We will add 10 votes per n, so devide target number by 10
    let n = n / 10;
    for _ in 0..(4*n) {
        tally.add(vec!["Memphis", "Nashville", "Chattanooga", "Knoxville"]);
    }
    for _ in 0..(3*n) {
        tally.add(vec!["Nashville", "Chattanooga", "Knoxville", "Memphis"]);
    }
    for _ in 0..(2*n) {
        tally.add(vec!["Chattanooga", "Knoxville", "Nashville", "Memphis"]);
    }
    for _ in 0..(1*n) {
        tally.add(vec!["Knoxville", "Chattanooga", "Nashville", "Memphis"]);
    }

    tally.result();
}

fn stv(n: u64) {
    let mut tally = tallyman::stv::Tally::new(1, tallyman::stv::Quota::Droop);
    
    // We will add 10 votes per n, so devide target number by 10
    let n = n / 10;
    for _ in 0..(4*n) {
        tally.add(vec!["Memphis", "Nashville", "Chattanooga", "Knoxville"]);
    }
    for _ in 0..(3*n) {
        tally.add(vec!["Nashville", "Chattanooga", "Knoxville", "Memphis"]);
    }
    for _ in 0..(2*n) {
        tally.add(vec!["Chattanooga", "Knoxville", "Nashville", "Memphis"]);
    }
    for _ in 0..(1*n) {
        tally.add(vec!["Knoxville", "Chattanooga", "Nashville", "Memphis"]);
    }

    tally.result();
}


fn condorcet_benchmark(c: &mut Criterion) {
    c.bench(
        "condorcet 100,000",
        Benchmark::new("condorcet 100,000", |b| b.iter(|| condorcet(100_000)))
            .sample_size(10)
    );

    c.bench(
        "condorcet 1M",
        Benchmark::new("condorcet 1M", |b| b.iter(|| condorcet(1_000_000)))
            .sample_size(5)
    );

fn stv_benchmark(c: &mut Criterion) {
    c.bench(
        "stv 100,000",
        Benchmark::new("stv 100,000", |b| b.iter(|| stv(100_000)))
            .sample_size(10)
    );

    c.bench(
        "stv 1M",
        Benchmark::new("stv 1M", |b| b.iter(|| stv(1_000_000)))
            .sample_size(5)
    );
}

criterion_group!(benches, condorcet_benchmark, stv_benchmark);
criterion_main!(benches);