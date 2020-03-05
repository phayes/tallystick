# tallystick

[![docs](https://docs.rs/tallystick/badge.svg)](https://docs.rs/tallystick)
[![crates.io](https://meritbadge.herokuapp.com/tallystick)](https://crates.io/crates/tallystick)
[![Build Status](https://travis-ci.org/phayes/tallystick.svg?branch=master)](https://travis-ci.org/phayes/tallystick)
[![codecov](https://codecov.io/gh/phayes/tallystick/branch/master/graph/badge.svg)](https://codecov.io/gh/phayes/tallystick)
[![patreon](https://img.shields.io/badge/patreon-donate-green.svg)](https://patreon.com/phayes)
[![flattr](https://img.shields.io/badge/flattr-donate-green.svg)](https://flattr.com/@phayes)


tallystick is a work-in-progress rust library for tallying votes.

_Current state is very unstable. It is not currently recommended for use. See checkboxes below for a list of features that are complete._

## Goals

1. **Ergonomic** - Easy to use.
2. **Fast** - Be the fastest general-purpose vote tallying library in the world.
3. **Secure** - Have no undefined behavior.
4. **Complete** - Support all well-known voting methods.
5. **Deterministic** - Running the same tally twice should never produce different results.
6. **Generic** - Generic over both candidate and count types.

## Features

- [ ] `wasm` support for use in the browser, or in blockchain smart contracts.
- [ ] `rational` support for perfectly-precise tallies by using rational fractions instead of floats.
- [ ] `fixed_point` support for decimal fixed-point tallies, required by some statutes.
- [ ] `ffi` support for calling from other programming languages.
- [ ] `alloc` support for embedded and other applications where there is an allocator, but no standard library.

## Supported Tally Methods

| Status¹ | Tally Method      | Supported Variants                   | Performance²    | Notes                      |
| ------- | ----------------- | ------------------------------------ | --------------- | -------------------------- |
| ✓       | Plurality         |                                      | 120 million v/s | First Past the Post (FPTP) |
| ✓       | Score             |                                      | 3 million v/s   |                            |
| ✓       | Approval          |                                      | 4 million v/s   |                            |
| ⚠       | STV               | Newland-Britton, Meek, Warren        | 3 million v/s   | Single Transferable Vote   |
|         | CPO-STV           |                                      |                 |                            |
|         | Instant Runoff    |                                      |                 |                            |
|         | Contingent        |                                      |                 |                            |
|         | Supplementary     |                                      |                 |                            |
| ✓       | Condorcet         |                                      | 2 million v/s   |                            |
|         | Copeland          |                                      |                 |                            |
| ✓       | Schulze           | Winning, Margin, Ratio               | 2 million v/s   |                            |
|         | Schulze STV       |                                      |                 |                            |
|         | Kemeny–Young      |                                      |                 |                            |
|         | Minimax           |                                      |                 |                            |
| ✓       | Borda             | Classic, Dowdall, Modified           | 3 million v/s   |                            |
|         | Borda - Nanson    | Classic, Dowdall, Modified           |                 |                            |
|         | Borda - Baldwin   | Classic, Dowdall, Modified           |                 |                            |
|         | Dodgson           | Quick, Tideman                       |                 |                            |
|         | Ranked pairs      | Margin, Winning                      |                 |                            |
|         | STAR              |                                      |                 |                            |
|         | Majority judgment |                                      |                 |                            |
|         | D'Hondt           | Sainte-Laguë, Huntington-Hill, Quota |                 |                            |

1. ✓ means done, ⚠ means in-progress, blank means not started but support is planned.
2. Single threaded performance measured in votes tallied per second. Benchmarked on a 2017 Macbook Pro.

## Contributors

- Patrick Hayes ([linkedin](https://www.linkedin.com/in/patrickdhayes/)) ([github](https://github.com/phayes)) - Available for hire.
- Kurtis Jensen ([github](https://github.com/kbuilds))
