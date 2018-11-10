#[macro_use]
extern crate criterion;
extern crate tallyman;
extern crate rand;

use criterion::Criterion;
use criterion::Benchmark;
use criterion::Throughput;
use std::cmp::Eq;
use std::fmt::Debug;
use std::hash::Hash;
use rand::prelude::*;

fn condorcet_benchmark(c: &mut Criterion) {

    // 100K from predefined list of candidates and candidate ratios
    c.bench(
        "condorcet",
        Benchmark::new("static/100K", |b| b.iter(|| condorcet(generate_static_votes(100_000))))
            .sample_size(10).throughput(Throughput::Elements(100_000))
    );

    // 1M from predefined list of candidates and candidate ratios
    c.bench(
        "condorcet",
        Benchmark::new("static/1M", |b| b.iter(|| condorcet(generate_static_votes(1_000_000))))
            .sample_size(5).throughput(Throughput::Elements(1_000_000))
    );

    c.bench(
        "condorcet",
        Benchmark::new("random/100K", |b| b.iter(|| condorcet(generate_random_votes(100_000))))
            .sample_size(5).throughput(Throughput::Elements(100_000))
    );

    c.bench(
        "condorcet",
        Benchmark::new("random/1M", |b| b.iter(|| condorcet(generate_random_votes(1_000_000))))
            .sample_size(5).throughput(Throughput::Elements(1_000_000))
    );
}

fn stv_benchmark(c: &mut Criterion) {
    c.bench(
        "stv",
        Benchmark::new("static/100,000", |b| b.iter(|| stv(generate_static_votes(100_000))))
            .sample_size(10).throughput(Throughput::Elements(100_000))
    );

    c.bench(
        "stv",
        Benchmark::new("static/1M", |b| b.iter(|| stv(generate_static_votes(1_000_000))))
            .sample_size(5).throughput(Throughput::Elements(1_000_000))
    );

    c.bench(
        "stv",
        Benchmark::new("static/100,000", |b| b.iter(|| stv(generate_random_votes(100_000))))
            .sample_size(10).throughput(Throughput::Elements(100_000))
    );

    c.bench(
        "stv",
        Benchmark::new("static/1M", |b| b.iter(|| stv(generate_random_votes(1_000_000))))
            .sample_size(5).throughput(Throughput::Elements(1_000_000))
    );
}

// Build a tally, put votes into the tally, and compute the results.
fn condorcet<T: Eq + Clone + Hash + Debug>(mut votes: Vec<Vec<T>>) {
    let mut tally = tallyman::condorcet::Tally::<T>::new(1);

    for vote in votes.drain(0..) {
        tally.add(vote);
    }

    tally.result();
}

fn stv<T: Eq + Clone + Hash + Debug>(mut votes: Vec<Vec<T>>) {
    let mut tally = tallyman::stv::Tally::new(1, tallyman::stv::Quota::Droop);
    
    for vote in votes.drain(0..) {
        tally.add(vote);
    }

    tally.result();
}

fn generate_random_votes(n: u32) -> Vec<Vec<u8>> {
    let mut rng = thread_rng();
    let mut all_votes = Vec::new();
    for _ in 0..n {
        let mut vote = Vec::<u8>::new();
        for _ in 0..rng.gen_range(0, 10) {
            vote.push(rng.gen());
        }
        all_votes.push(vote);
    }

    return all_votes;
}

fn generate_static_votes(n: u32) -> Vec<Vec<String>> {
    // We will add 10 votes per n, so devide target number by 10
    let mut all_votes = Vec::new();
    let n = n / 10;
    for _ in 0..(4*n) {
        all_votes.push(vec!["Memphis".to_owned(), "Nashville".to_owned(), "Chattanooga".to_owned(), "Knoxville".to_owned()]);
    }
    for _ in 0..(3*n) {
        all_votes.push(vec!["Nashville".to_owned(), "Chattanooga".to_owned(), "Knoxville".to_owned(), "Memphis".to_owned()]);
    }
    for _ in 0..(2*n) {
        all_votes.push(vec!["Chattanooga".to_owned(), "Knoxville".to_owned(), "Nashville".to_owned(), "Memphis".to_owned()]);
    }
    for _ in 0..(1*n) {
        all_votes.push(vec!["Knoxville".to_owned(), "Chattanooga".to_owned(), "Nashville".to_owned(), "Memphis".to_owned()]);
    }

    return all_votes;
}

criterion_group!(benches, condorcet_benchmark, stv_benchmark);
criterion_main!(benches);