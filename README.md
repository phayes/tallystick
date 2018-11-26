# tallyman

[![Build Status](https://travis-ci.org/phayes/tallyman.svg?branch=master)](https://travis-ci.org/phayes/tallyman)
[![codecov](https://codecov.io/gh/phayes/tallyman/branch/master/graph/badge.svg)](https://codecov.io/gh/phayes/tallyman)


tallyman is a work-in-progress rust library for tallying votes.

*Current state is very unstable. It is not currently recommended for use. See checkboxes below for a list of features that are complete.*

## Goals
1. **Fast** - Be the fastest general-purpose vote tallying library in the world.
2. **Secure** - Have no undefined behavior. 
3. **Complete** - Support all well-known voting methods.
4. **Deterministic** - Running the same tally twice should never produce different results.
5. **Generic** - Generic over candidates and vote weights. 

## Features
- [ ] `no_std` for embedded use.
- [ ] `wasm` support for use in the browser, or in blockchain smart contracts.
- [ ] `rational` support for perfectly-precise tallies by using rational fractions instead of floats.
- [ ] `rayon` support for multi-threaded tallies.

## Supported Tally Methods
- [x] Plurality (first-past-the-post)
- [x] Approval
- [x] Score
- [x] Single Transferable Vote
  - [X] Droop
  - [X] Hagenbach-Bischoff
  - [X] Hare
  - [X] Imperiali
- [ ] CPO-STV
- [ ] Instant Runoff
- [ ] Contingent
- [x] Condorcet
- [ ] Copeland
- [ ] Schulze (Winning, Margin, Ratio)
- [ ] Kemeny–Young
- [ ] Minimax (Winning, Margin, Opposition)
- [X] Borda (Classic, Dowdall, Modified)
  - [ ] Nanson
  - [ ] Baldwin
- [ ] Dodgson (Quick, Tideman)
- [ ] Ranked pairs (Margin, Winning)
- [ ] STAR
- [ ] Majority judgment
- [ ] D'Hondt (Sainte-Laguë, Huntington-Hill, Quota)

## Tally Methods

| Status¹| Tally Method      | Supported Variants                   | Performance²     | Notes  |
| -------|-------------------|--------------------------------------|------------------|--------|
| ✓      | Plurality (FPTP)  | 100 million v/s                      | 100 million v/s  |        |
| ⚠      | Score             |                                      |                  |        |
| ⚠      | Approval          |                                      |                  |        |
| ⚠      | STV               | Droop, Hagenbach-Bischoff, Hare      | 3 million v/s    |        |
|        | CPO-STV           |                                      |                  |        |
|        | Instant Runoff    |                                      |                  |        |
|        | Contingent        |                                      |                  |        |
| ⚠      | Condorcet         |                                      | 2 million v/s    |        |
|        | Copeland          |                                      |                  |        |
|        | Schulze           | Winning, Margin, Ratio               |                  |        |
|        | Kemeny–Young      |                                      |                  |        |
|        | Minimax           |                                      |                  |        |
| ⚠      | Borda             | Classic, Dowdall, Modified           |                  |        |
|        | Borda - Nanson    | Classic, Dowdall, Modified           |                  |        |
|        | Borda - Baldwin   | Classic, Dowdall, Modified           |                  |        |
|        | Dodgson           | Quick, Tideman                       |                  |        |
|        | Ranked pairs      | Margin, Winning                      |                  |        |
|        | STAR              |                                      |                  |        |
|        | Majority judgment |                                      |                  |        |
|        | D'Hondt           | Sainte-Laguë, Huntington-Hill, Quota |                  |        |


1. ✓ means done, ⚠ means in-progress, blank means not started but support is planned.
2. Performance is measured in votes tallied per second. Benchmarked on a 2017 Macbook Pro.



