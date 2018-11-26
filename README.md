# TallyMan

TallyMan is a work-in-progress rust library for tallying votes.

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
- [x] Plurality
- [x] Approval
- [x] Score
- [x] Single Transferable Vote (Droop, Hare, Hagenbach-Bischoff)
- [ ] CPO-STV
- [ ] Instant Runoff
- [ ] Contingent
- [x] Condorcet
- [ ] Copeland
- [ ] Schulze
- [ ] Kemeny–Young
- [ ] Minimax
- [X] Borda
  - [ ] Nanson
  - [ ] Baldwin
- [ ] Dodgson
- [ ] Ranked pairs
- [ ] STAR
- [ ] Majority judgment
- [ ] D'Hondt (Sainte-Laguë, Huntington-Hill, Quota)

## Performance

| Tally Method  | Performance (votes per second)  | Notes  |
| --------------|---------------------------------|--------|
| Plurality     | 100 million v/s                 |        |
| STV           | 3 million v/s                   |        |
| Condorcet     | 2 million v/s                   |        |