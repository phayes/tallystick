# TallyMan

TallyMan is a work-in-progress rust library for tallying votes in an election.

Current state: Very unstable. Not currently recommended for use. See checkboxes below for a list of features that are complete.

## Goals
1. **Fast** - Be the fastest vote tallying library in the world.
2. **Secure** - Have no undefined behavior. 
3. **Complete** - Support all well-known voting methods.
4. **Deterministic** - Running a tally twice should never produce different results.

## Features
- [ ] `no_std` for embedded use [-]
- [ ] `wasm` support for use in the browser, or in blockchain chaincode. [ ]
- [ ] `rational` support for perfectly-precise tallies by using rational fractions instead of floats. [ ]
- [ ] `rayon` support for multi-threaded tallies. [ ]

## Supported Tally Methods
- [x] Plurality [x]
- [ ] Approval
- [ ] Score
- [x] Single Transferable Vote (Droop, Hare, Hagenbach-Bischoff)
- [ ] Instant Runoff
- [ ] Contingent
- [x] Condorcet
- [ ] Copeland
- [ ] Schulze
- [ ] Kemeny–Young
- [ ] Minimax
- [ ] Borda (Nanson, Baldwin)
- [ ] Dodgson
- [ ] Ranked pairs
