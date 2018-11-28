//! TallyMan is a work-in-progress rust library for tallying votes.
//!
//! ## Compatibility
//!
//! The `tallyman` crate currently needs nightly rust. It will move to stable when [trait specialization](https://github.com/rust-lang/rust/issues/31844) is stabilized.
//!

#![warn(rust_2018_idioms)]
#![warn(missing_docs)]
#![feature(nll)]
#![feature(specialization)]

/// Plurality voting is an electoral system in which each voter is allowed to vote for only one candidate
/// and the candidate who polls the most among their counterparts (a plurality) is elected. It may be called
/// first-past-the-post (FPTP), single-choice voting, simple plurality, or relative/simple majority.
///
/// # Example
/// ```
///    use tallyman::plurality::DefaultPluralityTally;
///
///    // Election between Alice, Bob, and Cir with two winners.
///    let mut tally = DefaultPluralityTally::new(2);
///    tally.add("Alice");
///    tally.add("Cir");
///    tally.add("Bob");
///    tally.add("Alice");
///    tally.add("Alice");
///    tally.add("Bob");
///
///    let winners = tally.winners().into_unranked();
///    println!("The winners are {:?}", winners);
/// ```
pub mod plurality;

/// Approval voting is a single-winner electoral system where each voter may select ("approve") any number of
/// candidates. The winner is the most-approved candidate.
pub mod approval;

/// Score voting or "range voting" is an electoral system in which voters give each candidate a score,
/// the scores are summed, and the candidate with the highest total is elected. It has been described
/// by various other names including "evaluative voting", "utilitarian voting", and "the point system".
pub mod score;

/// The single transferable vote (STV) is a ranked choice voting system.
/// Under STV, a voter has a single vote that is initially allocated to their most preferred candidate. Votes are totalled and a quota
/// (the number of votes required to win) derived. If a candidate achieves quota, the candidate is elected and any surplus vote
/// is transferred to other candidates in proportion to the voters' stated preferences. If no candidate achieves quota,
/// the bottom candidate is eliminated with votes being transferred to other candidates as determined by the voters' stated preferences.
/// These elections, eliminations, and vote transfers continue in rounds until the correct number of candidates are elected.
pub mod stv;

/// The Condorcet method is a ranked-choice voting system that elects the candidate that would win a majority
/// of the vote in all of the head-to-head elections against each of the other candidates.
///
/// The Condorcet method isn't guarunteed to produce a single-winner due to the non-transitive nature of group choice.
pub mod condorcet;

/// The Borda count is a family of election methods in which voters rank candidates in order of preference.
/// The Borda count determines the winner by giving each candidate, for each ballot, a number of points corresponding to the number of candidates ranked lower.
/// Once all votes have been counted the candidate with the most points is the winner.
///
/// # Example
/// ```
///    use tallyman::borda::DefaultBordaTally;
///    use tallyman::borda::Variant;
///
///    // Election between Alice, Bob, and Carlos with two winners.
///    let mut tally = DefaultBordaTally::new(2, Variant::Borda);
///    tally.add(vec!["Alice", "Bob", "Carlos"]);
///    tally.add(vec!["Bob", "Carlos", "Alice"]);
///    tally.add(vec!["Alice", "Bob", "Carlos"]);
///    tally.add(vec!["Carlos", "Bob", "Alice"]);
///    tally.add(vec!["Alice", "Carlos", "Bob"]);
///
///    let winners = tally.winners().into_unranked();
///    println!("The winners are {:?}", winners);
/// ```
pub mod borda;

// Common Data Structures
// ----------------------
mod result;
pub use crate::result::RankedWinners;

mod quota;
pub use crate::quota::Quota;

mod traits;
pub use crate::traits::Numeric;

mod errors;
pub use crate::errors::TallyError;

// Common Utility Functions
// ------------------------

// Check if a vector has a duplicate
// This is critical for transitive (ranked) votes
pub(crate) fn check_duplicate<T: PartialEq>(slice: &[T]) -> Result<(), TallyError> {
  for i in 1..slice.len() {
    if slice[i..].contains(&slice[i - 1]) {
      return Err(TallyError::VoteHasDuplicateCandidates);
    }
  }
  Ok(())
}
