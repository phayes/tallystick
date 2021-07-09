use num_traits::cast::NumCast;
use num_traits::Num;
use std::hash::Hash;
use std::ops::AddAssign;

use super::plurality::PluralityTally;
use super::result::RankedCandidate;
use super::result::RankedWinners;

/// A score tally using `u64` integers to count votes.
/// `DefaultScoreTally` is generally preferred over `ScoreTally`.
/// Since this is an alias, refer to [`ScoreTally`](struct.ScoreTally.html) for method documentation.
///
/// # Example
/// ```
///    use tallystick::score::DefaultScoreTally;
///
///    // An election for Judge
///    let mut tally = DefaultScoreTally::<&str>::new(1);
///    tally.add(vec![("Judge Judy", 5), ("Notorious RBG", 2)]);
///    tally.add(vec![("Judge Dredd", 5)]);
///    tally.add(vec![("Abe Vigoda", 6), ("Notorious RBG", 3)]);
///    tally.add(vec![("Judge Dredd", 1), ("Notorious RBG", 4)]);
///
///    let winners = tally.winners().into_unranked();
///    assert!(winners[0] == "Notorious RBG");
/// ```
pub type DefaultScoreTally<T> = ScoreTally<T, u64>;

/// A generic score tally.
///
/// Generics:
/// - `T`: The candidate type.
/// - `C`: The count and score type. `u64` is recommended, but can be modified to use a different type for counting votes (eg `f64` for fractional scores).
///
/// # Example
/// ```
///    use tallystick::score::ScoreTally;
///
///    // An election for Judge using floats as the score type.
///    let mut tally = ScoreTally::<&str, f64>::new(1);
///    tally.add(vec![("Judge Judy", 3.5), ("Notorious RBG", 2.5)]);
///    tally.add(vec![("Judge Dredd", 0.5)]);
///    tally.add(vec![("Abe Vigoda", 6.1), ("Notorious RBG", 3.2)]);
///    tally.add(vec![("Judge Dredd", 1.0), ("Notorious RBG", 4.1)]);
///
///    let winners = tally.winners().into_unranked();
///    assert!(winners[0] == "Notorious RBG");
/// ```
pub struct ScoreTally<T, C = u64>
where
    T: Eq + Clone + Hash,                             // Candidate
    C: Copy + PartialOrd + AddAssign + Num + NumCast, // Vote count type
{
    plurality: PluralityTally<T, C>,
}

impl<T, C> ScoreTally<T, C>
where
    T: Eq + Clone + Hash,                             // Candidate
    C: Copy + PartialOrd + AddAssign + Num + NumCast, // Vote count type
{
    /// Create a new `ScoreTally` with the given number of winners.
    ///
    /// If there is a tie, the number of winners might be more than `num_winners`.
    /// (See [`winners()`](#method.winners) for more information on ties.)
    pub fn new(num_winners: usize) -> Self {
        ScoreTally {
            plurality: PluralityTally::new(num_winners),
        }
    }

    /// Create a new `ScoreTally` with the given number of winners, and number of expected candidates.
    pub fn with_capacity(num_winners: usize, expected_candidates: usize) -> Self {
        ScoreTally {
            plurality: PluralityTally::with_capacity(num_winners, expected_candidates),
        }
    }

    /// Add a new vote
    pub fn add(&mut self, mut selection: Vec<(T, C)>) {
        for (vote, score) in selection.drain(0..) {
            self.plurality.add_weighted(vote, score);
        }
    }

    /// Add a vote by reference.
    pub fn add_ref(&mut self, selection: &[(T, C)]) {
        for (vote, score) in selection {
            self.plurality.add_weighted_ref(vote, *score);
        }
    }

    /// Add a weighted vote.
    /// By default takes a weight as a `usize` integer, but can be customized by using `ApprovalTally` with a custom vote type.
    pub fn add_weighted(&mut self, mut selection: Vec<(T, C)>, weight: C) {
        for (vote, score) in selection.drain(0..) {
            self.plurality.add_weighted(vote, weight * score);
        }
    }

    /// Add a weighted vote by reference.
    pub fn add_weighted_ref(&mut self, selection: &[(T, C)], weight: C) {
        for (vote, score) in selection {
            self.plurality.add_weighted_ref(vote, weight * *score);
        }
    }

    /// Get a list of all candidates seen by this tally.
    /// Candidates are returned in no particular order.
    pub fn candidates(&self) -> Vec<T> {
        self.plurality.candidates()
    }

    /// Get a ranked list of winners. Winners with the same rank are tied.
    /// The number of winners might be greater than the requested `num_winners` if there is a tie.
    /// In score voting, the winning candidate(s) is the one with the highest total score.
    pub fn winners(&self) -> RankedWinners<T> {
        self.plurality.winners()
    }

    /// Get vote totals for this tally.
    ///
    /// Each candidate has a total thhat is equal to the sum of all scores for that candidate.
    ///
    /// # Example
    /// ```
    ///    use tallystick::score::DefaultScoreTally;
    ///
    ///    let mut tally = DefaultScoreTally::new(1);
    ///    tally.add(vec![("Alice", 30), ("Bob", 10)]);
    ///    tally.add(vec![("Bob", 10), ("Carol", 5)]);
    ///
    ///    for (candidate, score) in tally.totals().iter() {
    ///       println!("{} got a score of {}", candidate, score);
    ///    }
    ///    // Prints:
    ///    //   Alice got a score of 30
    ///    //   Bob got a score of 20
    ///    //   Carol got a score of 5
    /// ```
    pub fn totals(&self) -> Vec<(T, C)> {
        self.plurality.totals()
    }

    /// Get a ranked list of all candidates. Candidates with the same rank are tied.
    /// Candidates are ranked in ascending order. The highest ranked candidate has a rank of `0`.
    ///
    /// # Example
    /// ```
    ///    use tallystick::score::DefaultScoreTally;
    ///
    ///    let mut tally = DefaultScoreTally::new(1);
    ///    tally.add(vec![("Alice", 30), ("Bob", 10)]);
    ///    tally.add(vec![("Bob", 10), ("Carol", 5)]);
    ///    
    ///    for ranked in tally.ranked().iter() {
    ///       println!("{} has a rank of {}", ranked.candidate, ranked.rank);
    ///    }
    ///    // Prints:
    ///    //   Alice has a rank of 0
    ///    //   Bob has a rank of 1
    ///    //   Carol has a rank of 2
    /// ```
    pub fn ranked(&self) -> Vec<RankedCandidate<T>> {
        self.plurality.ranked()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_basic() {
        let mut tally = ScoreTally::new(1);
        tally.add(vec![("Alice", 10), ("Bob", 4)]);
        tally.add_ref(&vec![("Alice", 2), ("Bob", 2)]);
        tally.add_weighted_ref(&vec![("Alice", 1), ("Bob", 1)], 5);

        let candidates = tally.candidates();
        assert_eq!(candidates.len(), 2);

        let totals = tally.totals();
        assert_eq!(totals, vec![("Alice", 17), ("Bob", 11)]);

        let ranked = tally.ranked();
        assert_eq!(ranked, vec![("Alice", 0), ("Bob", 1)]);

        let winners = tally.winners();
        assert_eq!(winners.is_empty(), false);
        assert_eq!(winners.check_overflow(), false);
        assert_eq!(winners.overflow(), Option::None);
        assert_eq!(winners.all(), vec!["Alice"]);
    }

    #[test]
    fn score_wikipedia() {
        // From: https://en.wikipedia.org/wiki/Score_voting

        let mut tally = ScoreTally::with_capacity(1, 4);
        tally.add_weighted(vec![("Memphis", 10), ("Nashville", 4), ("Chattanooga", 2), ("Knoxville", 0)], 42);
        tally.add_weighted(vec![("Memphis", 0), ("Nashville", 10), ("Chattanooga", 4), ("Knoxville", 2)], 26);
        tally.add_weighted(vec![("Memphis", 0), ("Nashville", 6), ("Chattanooga", 10), ("Knoxville", 6)], 15);
        tally.add_weighted(vec![("Memphis", 0), ("Nashville", 5), ("Chattanooga", 7), ("Knoxville", 10)], 17);

        let candidates = tally.candidates();
        assert_eq!(candidates.len(), 4);

        let totals = tally.totals();
        assert_eq!(
            totals,
            vec![("Nashville", 603), ("Chattanooga", 457), ("Memphis", 420), ("Knoxville", 312)]
        );

        let ranked = tally.ranked();
        assert_eq!(ranked, vec![("Nashville", 0), ("Chattanooga", 1), ("Memphis", 2), ("Knoxville", 3)]);

        let winners = tally.winners();
        assert_eq!(winners.is_empty(), false);
        assert_eq!(winners.check_overflow(), false);
        assert_eq!(winners.overflow(), Option::None);
        assert_eq!(winners.all(), vec!["Nashville"]);
    }
}
