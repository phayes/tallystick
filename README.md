


## Goals
1. **Fast** - Be the fastest vote tallying library in the world.
2. **Secure** - Have no undefined behavior. 
3. **Complete** - Support all well-known voting methods.
4. **Deterministic** - Running a tally twice should never produce different results.

## Features
1. `no_std` for embedded use[ ]
2. `wasm` support for use in the browser, or in blockchain chaincode. [ ]
3. `rational` support for perfectly-precise tallies by using rational fractions instead of floats. [ ]
4. `rayon` support for multi-threaded tallies. [ ]

## Supported Tally Methods
 1. Plurality [x]
 2. Approval [ ]
 3. Score [ ]
 4. Single Transferable Vote (Droop, Hare, Hagenbach-Bischoff) [x]
 5. Instant Runoff [ ]
 6. Contingent [ ]
 7. Condorcet [x]
 8. Copeland [ ]
 9. Schulze [ ]
 10. Kemeny–Young [ ]
 11. Minimax [ ]
 12. Borda (Nanson, Baldwin) [ ]
 13. Dodgson [ ]
 14. Ranked pairs [ ]
