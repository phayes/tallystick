use tallystick;

use criterion::{criterion_group, criterion_main, Benchmark, Criterion, Throughput};
use rand::prelude::*;
use std::cmp::Eq;
use std::hash::Hash;

const SAMPLE_SIZE: usize = 100;

#[cfg(feature = "nightly")]
criterion_group!(
    benches,
    borda_benchmark,
    condorcet_benchmark,
    schulze_benchmark,
    stv_benchmark,
    plurality_benchmark,
    approval_benchmark,
    score_benchmark
);

#[cfg(not(feature = "nightly"))]
criterion_group!(
    benches,
    condorcet_benchmark,
    plurality_benchmark,
    approval_benchmark,
    score_benchmark
);

criterion_main!(benches);

fn plurality_benchmark(c: &mut Criterion) {
    c.bench(
        "plurality",
        Benchmark::new("random/10K", |b| b.iter(|| plurality(random_single_votes(10_000), 10)))
            .sample_size(SAMPLE_SIZE)
            .throughput(Throughput::Elements(10_000)),
    );
}

fn approval_benchmark(c: &mut Criterion) {
    c.bench(
        "approval",
        Benchmark::new("static/10K", |b| b.iter(|| approval(static_transitive_votes(10_000), 4)))
            .sample_size(SAMPLE_SIZE)
            .throughput(Throughput::Elements(10_000)),
    );

    c.bench(
        "approval",
        Benchmark::new("random/10K", |b| b.iter(|| approval(random_transitive_votes(10_000), 10)))
            .sample_size(SAMPLE_SIZE)
            .throughput(Throughput::Elements(10_000)),
    );
}

fn score_benchmark(c: &mut Criterion) {
    c.bench(
        "score",
        Benchmark::new("random/10K", |b| b.iter(|| score(random_scored_votes(10_000), 4)))
            .sample_size(SAMPLE_SIZE)
            .throughput(Throughput::Elements(10_000)),
    );
}

fn condorcet_benchmark(c: &mut Criterion) {
    // 10K from predefined list of candidates and candidate ratios
    c.bench(
        "condorcet",
        Benchmark::new("static/10K", |b| {
            b.iter(|| {
                condorcet(
                    static_transitive_votes(10_000),
                    vec!["Memphis", "Nashville", "Chattanooga", "Knoxville"],
                )
            })
        })
        .sample_size(SAMPLE_SIZE)
        .throughput(Throughput::Elements(10_000)),
    );

    // 10K from random
    c.bench(
        "condorcet",
        Benchmark::new("random/10K", |b| {
            b.iter(|| condorcet(random_transitive_votes(10_000), (0..10).collect()))
        })
        .sample_size(SAMPLE_SIZE)
        .throughput(Throughput::Elements(10_000)),
    );
}

#[cfg(feature = "nightly")]
fn stv_benchmark(c: &mut Criterion) {
    c.bench(
        "stv",
        Benchmark::new("static/10K", |b| b.iter(|| stv(static_transitive_votes(10_000), 4)))
            .sample_size(SAMPLE_SIZE)
            .throughput(Throughput::Elements(10_000)),
    );

    c.bench(
        "stv",
        Benchmark::new("random/10K", |b| b.iter(|| stv(random_transitive_votes(10_000), 10)))
            .sample_size(SAMPLE_SIZE)
            .throughput(Throughput::Elements(10_000)),
    );
}

#[cfg(feature = "nightly")]
fn schulze_benchmark(c: &mut Criterion) {
    c.bench(
        "schulze",
        Benchmark::new("static/10K", |b| {
            b.iter(|| {
                schulze(
                    static_transitive_votes(10_000),
                    vec!["Memphis", "Nashville", "Chattanooga", "Knoxville"],
                )
            })
        })
        .sample_size(SAMPLE_SIZE)
        .throughput(Throughput::Elements(10_000)),
    );

    c.bench(
        "schulze",
        Benchmark::new("random/10K", |b| {
            b.iter(|| schulze(random_transitive_votes(10_000), (0..10).collect()))
        })
        .sample_size(SAMPLE_SIZE)
        .throughput(Throughput::Elements(10_000)),
    );
}

#[cfg(feature = "nightly")]
fn borda_benchmark(c: &mut Criterion) {
    c.bench(
        "borda",
        Benchmark::new("static/10K", |b| b.iter(|| borda(static_transitive_votes(10_000), 4)))
            .sample_size(SAMPLE_SIZE)
            .throughput(Throughput::Elements(10_000)),
    );

    c.bench(
        "borda",
        Benchmark::new("random/10K", |b| b.iter(|| borda(random_transitive_votes(10_000), 10)))
            .sample_size(SAMPLE_SIZE)
            .throughput(Throughput::Elements(10_000)),
    );
}

// Build a tally, put votes into the tally, and compute the results.
fn condorcet<T: Eq + Clone + Hash>(votes: Vec<Vec<T>>, candidates: Vec<T>) {
    let mut tally = tallystick::condorcet::DefaultCondorcetTally::with_candidates(1, candidates);

    for vote in votes {
        tally.add(&vote).unwrap();
    }

    tally.winners();
}

fn plurality<T: Eq + Clone + Hash>(mut votes: Vec<T>, num_candidates: usize) {
    let mut tally = tallystick::plurality::PluralityTally::<T, u64>::with_capacity(1, num_candidates);

    for vote in votes.drain(..) {
        tally.add(vote);
    }

    tally.winners();
}

fn approval<T: Eq + Clone + Hash>(mut votes: Vec<Vec<T>>, num_candidates: usize) {
    let mut tally = tallystick::approval::DefaultApprovalTally::with_capacity(1, num_candidates);

    for vote in votes.drain(0..) {
        tally.add(vote);
    }

    tally.winners();
}

fn score<T: Eq + Clone + Hash>(mut votes: Vec<Vec<(T, u64)>>, num_candidates: usize) {
    let mut tally = tallystick::score::DefaultScoreTally::with_capacity(1, num_candidates);

    for vote in votes.drain(0..) {
        tally.add(vote);
    }

    tally.winners();
}

#[cfg(feature = "nightly")]
fn stv<T: Eq + Clone + Hash>(mut votes: Vec<Vec<T>>, num_candidates: usize) {
    let mut tally = tallystick::stv::DefaultTally::with_capacity(1, tallystick::Quota::Droop, num_candidates, votes.len());

    for vote in votes.drain(0..) {
        tally.add(vote);
    }

    tally.winners();
}

#[cfg(feature = "nightly")]
fn schulze<T: Eq + Clone + Hash>(votes: Vec<Vec<T>>, candidates: Vec<T>) {
    let mut tally = tallystick::schulze::DefaultSchulzeTally::with_candidates(1, tallystick::schulze::Variant::Winning, candidates);

    for vote in votes {
        tally.add(&vote).unwrap();
    }

    tally.winners();
}

#[cfg(feature = "nightly")]
fn borda<T: Eq + Clone + Hash>(mut votes: Vec<Vec<T>>, num_candidates: usize) {
    let mut tally = tallystick::borda::DefaultBordaTally::with_capacity(1, tallystick::borda::Variant::Borda, num_candidates);

    for vote in votes.drain(0..) {
        tally.add(vote).unwrap();
    }

    tally.winners();
}

fn random_transitive_votes(n: u32) -> Vec<Vec<u8>> {
    let mut rng = thread_rng();
    let mut all_votes = Vec::new();
    for _ in 0..n {
        let mut vote = Vec::<u8>::new();
        for _ in 0..rng.gen_range(0, 10) {
            let candidate = rng.gen_range(0, 10);
            if !vote.contains(&candidate) {
                vote.push(candidate);
            }
        }
        all_votes.push(vote);
    }

    return all_votes;
}

fn random_scored_votes(n: u32) -> Vec<Vec<(u8, u64)>> {
    let mut rng = thread_rng();
    let mut all_votes = Vec::new();
    for _ in 0..n {
        let mut vote = Vec::<(u8, u64)>::new();
        let mut candidates = std::collections::HashSet::new();
        for _ in 0..rng.gen_range(0, 10) {
            let candidate = rng.gen_range(0, 10);
            candidates.insert(candidate);
            let score = rng.gen_range(0, 10);
            if !candidates.contains(&candidate) {
                vote.push((candidate, score));
            }
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
    for _ in 0..(4 * n) {
        all_votes.push(vec!["Memphis", "Nashville", "Chattanooga", "Knoxville"]);
    }
    for _ in 0..(3 * n) {
        all_votes.push(vec!["Nashville", "Chattanooga", "Knoxville", "Memphis"]);
    }
    for _ in 0..(2 * n) {
        all_votes.push(vec!["Chattanooga", "Knoxville", "Nashville", "Memphis"]);
    }
    for _ in 0..(1 * n) {
        all_votes.push(vec!["Knoxville", "Chattanooga", "Nashville", "Memphis"]);
    }

    return all_votes;
}
