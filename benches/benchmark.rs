#[macro_use]
extern crate criterion;
extern crate tallyman;
extern crate rand;

use criterion::Criterion;
use criterion::Benchmark;
use criterion::Throughput;
use std::cmp::Eq;
use std::hash::Hash;
use rand::prelude::*;

criterion_group!(benches, condorcet_benchmark, stv_benchmark, plurality_benchmark);
criterion_main!(benches);


fn plurality_benchmark(c: &mut Criterion) {
    c.bench(
        "plurality",
        Benchmark::new("random/100K", |b| b.iter(|| plurality(random_single_votes(100_000), 10)))
            .sample_size(5).throughput(Throughput::Elements(100_000))
    );

    c.bench(
        "plurality",
        Benchmark::new("random/1M", |b| b.iter(|| plurality(random_single_votes(1_000_000), 10)))
            .sample_size(5).throughput(Throughput::Elements(1_000_000))
    );
}


fn condorcet_benchmark(c: &mut Criterion) {

    // 100K from predefined list of candidates and candidate ratios
    c.bench(
        "condorcet",
        Benchmark::new("static/100K", |b| b.iter(|| condorcet(static_transitive_votes(100_000), 10)))
            .sample_size(10).throughput(Throughput::Elements(100_000))
    );

    // 1M from predefined list of candidates and candidate ratios
    c.bench(
        "condorcet",
        Benchmark::new("static/1M", |b| b.iter(|| condorcet(static_transitive_votes(1_000_000), 4)))
            .sample_size(5).throughput(Throughput::Elements(1_000_000))
    );

    c.bench(
        "condorcet",
        Benchmark::new("random/100K", |b| b.iter(|| condorcet(random_transitive_votes(100_000), 10)))
            .sample_size(5).throughput(Throughput::Elements(100_000))
    );

    c.bench(
        "condorcet",
        Benchmark::new("random/1M", |b| b.iter(|| condorcet(random_transitive_votes(1_000_000), 10)))
            .sample_size(5).throughput(Throughput::Elements(1_000_000))
    );
}

fn stv_benchmark(c: &mut Criterion) {
    c.bench(
        "stv",
        Benchmark::new("static/100,000", |b| b.iter(|| stv(static_transitive_votes(100_000) ,4)))
            .sample_size(10).throughput(Throughput::Elements(100_000))
    );

    c.bench(
        "stv",
        Benchmark::new("static/1M", |b| b.iter(|| stv(static_transitive_votes(1_000_000), 4)))
            .sample_size(5).throughput(Throughput::Elements(1_000_000))
    );

    c.bench(
        "stv",
        Benchmark::new("random/100,000", |b| b.iter(|| stv(random_transitive_votes(100_000), 10)))
            .sample_size(10).throughput(Throughput::Elements(100_000))
    );

    c.bench(
        "stv",
        Benchmark::new("random/1M", |b| b.iter(|| stv(random_transitive_votes(1_000_000), 10)))
            .sample_size(5).throughput(Throughput::Elements(1_000_000))
    );
}

// Build a tally, put votes into the tally, and compute the results.
fn condorcet<T: Eq + Clone + Hash>(mut votes: Vec<Vec<T>>, num_candidates: usize) {
    let mut tally = tallyman::condorcet::Tally::<T>::with_capacity(1, num_candidates);

    for vote in votes.drain(0..) {
        tally.add(vote);
    }

    tally.winners();
}

fn stv<T: Eq + Clone + Hash + std::fmt::Debug>(mut votes: Vec<Vec<T>>, num_candidates: usize) {
    let mut tally = tallyman::stv::DefaultTally::with_capacity(1, tallyman::stv::Quota::Droop, num_candidates, votes.len());
    
    for vote in votes.drain(0..) {
        tally.add(vote);
    }

    tally.winners();
}

fn plurality<T: Eq + Clone + Hash>(votes: Vec<T>, num_candidates: usize) {
    let mut tally = tallyman::plurality::DefaultTally::with_capacity(1, num_candidates);
    
    for vote in votes.iter() {
        tally.add(vote);
    }

    tally.winners();
}

fn random_transitive_votes(n: u32) -> Vec<Vec<u8>> {
    let mut rng = thread_rng();
    let mut all_votes = Vec::new();
    for _ in 0..n {
        let mut vote = Vec::<u8>::new();
        for _ in 0..rng.gen_range(0, 10) {
            vote.push(rng.gen_range(0, 10));
        }
        all_votes.push(vote);
    }

    return all_votes;
}

fn random_single_votes(n: u32) -> Vec<u8> {
    let mut rng = thread_rng();
    let mut all_votes = Vec::new();
    for _ in 0..n {
        all_votes.push(rng.gen_range(0, 10));
    }
    return all_votes;
}

fn static_transitive_votes(n: u32) -> Vec<Vec<&'static str>> {
    // We will add 10 votes per n, so devide target number by 10
    let mut all_votes = Vec::new();
    let n = n / 10;
    for _ in 0..(4*n) {
        all_votes.push(vec!["Memphis", "Nashville", "Chattanooga", "Knoxville"]);
    }
    for _ in 0..(3*n) {
        all_votes.push(vec!["Nashville", "Chattanooga", "Knoxville", "Memphis"]);
    }
    for _ in 0..(2*n) {
        all_votes.push(vec!["Chattanooga", "Knoxville", "Nashville", "Memphis"]);
    }
    for _ in 0..(1*n) {
        all_votes.push(vec!["Knoxville", "Chattanooga", "Nashville", "Memphis"]);
    }

    return all_votes;
}