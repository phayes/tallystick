use num_traits::cast::NumCast;
use num_traits::Num;
use std::hash::Hash;
use std::ops::AddAssign;

use super::plurality::PluralityTally;
use super::result::RankedWinners;

/// An approval tally using `u64` integers to count votes.
/// `DefaultApprovalTally` is generally preferred over `ApprovalTally`.
/// Since this is an alias, refer to [`ApprovalTally`](struct.ApprovalTally.html) for method documentation.
///
/// # Example
/// ```
///    use tallystick::approval::DefaultApprovalTally;
///
///    // An election for Judge
///    let mut tally = DefaultApprovalTally::<&str>::new(1);
///    tally.add(vec!["Judge Judy", "Notorious RBG"]);
///    tally.add(vec!["Judge Dredd"]);
///    tally.add(vec!["Abe Vigoda", "Notorious RBG"]);
///    tally.add(vec!["Judge Dredd", "Notorious RBG"]);
///
///    let winners = tally.winners().into_unranked();
///    assert!(winners[0] == "Notorious RBG");
/// ```
pub type DefaultApprovalTally<T> = ApprovalTally<T, u64>;

/// A generic approval tally.
///
/// Generics:
/// - `T`: The candidate type.
/// - `C`: The count type. `u64` is recommended, but can be modified to use a different type for counting votes (eg `f64` for fractional vote weights).
///
/// # Example
/// ```
///    use tallystick::approval::ApprovalTally;
///
///    // An election for Judge using floats as the count type.
///    let mut tally = ApprovalTally::<&str, f64>::new(1);
///    tally.add_weighted(vec!["Judge Judy", "Notorious RBG"], 0.5);
///    tally.add_weighted(vec!["Judge Dredd"], 2.0);
///    tally.add_weighted(vec!["Abe Vigoda", "Notorious RBG"], 3.2);
///    tally.add_weighted(vec!["Judge Dredd", "Notorious RBG"], 1.0);
///
///    let winners = tally.winners().into_unranked();
///    assert!(winners[0] == "Notorious RBG");
/// ```
pub struct ApprovalTally<T, C = u64>
where
    T: Eq + Clone + Hash,                             // Candidate
    C: Copy + PartialOrd + AddAssign + Num + NumCast, // Vote count type
{
    plurality: PluralityTally<T, C>,
}

impl<T, C> ApprovalTally<T, C>
where
    T: Eq + Clone + Hash,                             // Candidate
    C: Copy + PartialOrd + AddAssign + Num + NumCast, // Vote count type
{
    /// Create a new `ApprovalTally` with the given number of winners.
    ///
    /// If there is a tie, the number of winners might be more than `num_winners`.
    /// (See [`winners()`](#method.winners) for more information on ties.)
    pub fn new(num_winners: u32) -> Self {
        return ApprovalTally {
            plurality: PluralityTally::new(num_winners),
        };
    }

    /// Create a new `ApprovalTally` with the given number of winners, and number of expected candidates.
    pub fn with_capacity(num_winners: u32, expected_candidates: usize) -> Self {
        return ApprovalTally {
            plurality: PluralityTally::with_capacity(num_winners, expected_candidates),
        };
    }

    /// Add a new vote
    pub fn add(&mut self, mut selection: Vec<T>) {
        for vote in selection.drain(0..) {
            self.plurality.add(vote);
        }
    }

    /// Add a vote by reference.
    pub fn add_ref(&mut self, selection: &[T]) {
        for vote in selection {
            self.plurality.add_ref(vote);
        }
    }

    /// Add a weighted vote.
    /// By default takes a weight as a `usize` integer, but can be customized by using `ApprovalTally` with a custom vote type.
    pub fn add_weighted(&mut self, mut selection: Vec<T>, weight: C) {
        for vote in selection.drain(0..) {
            self.plurality.add_weighted(vote, weight);
        }
    }

    /// Add a weighted vote by reference.
    pub fn add_weighted_ref(&mut self, selection: &[T], weight: C) {
        for vote in selection {
            self.plurality.add_weighted_ref(vote, weight);
        }
    }

    /// Get a list of all candidates seen by this tally.
    /// Candidates are returned in no particular order.
    pub fn candidates(&self) -> Vec<T> {
        return self.plurality.candidates();
    }

    /// Get a ranked list of winners. Winners with the same rank are tied.
    /// The number of winners might be greater than the requested `num_winners` if there is a tie.
    /// In approval voting, the winning candidate(s) is the one most approved by all voters.
    pub fn winners(&self) -> RankedWinners<T> {
        return self.plurality.winners();
    }

    /// Get vote totals for this tally.
    ///
    /// Each candidate has a total thhat is equal to the number of voters that approve of that candidate.
    ///
    /// If vote weights are used, then each candidates total is equal to the weighted sum of the votes that include that candidate.
    ///
    /// # Example
    /// ```
    ///    use tallystick::approval::DefaultApprovalTally;
    ///
    ///    let mut tally = DefaultApprovalTally::new(1);
    ///    tally.add_weighted(vec!["Alice", "Bob"], 30);
    ///    tally.add_weighted(vec!["Bob", "Carol"], 10);
    ///
    ///    for (candidate, num_votes) in tally.totals().iter() {
    ///       println!("{} got {} votes", candidate, num_votes);
    ///    }
    ///    // Prints:
    ///    //   Alice got 30 votes
    ///    //   Bob got 40 votes
    ///    //   Carol got 10 votes
    /// ```
    pub fn totals(&self) -> Vec<(T, C)> {
        return self.plurality.totals();
    }

    /// Get a ranked list of all candidates. Candidates with the same rank are tied.
    /// Candidates are ranked in ascending order. The highest ranked candidate has a rank of `0`.
    ///
    /// # Example
    /// ```
    ///    use tallystick::approval::DefaultApprovalTally;
    ///
    ///    let mut tally = DefaultApprovalTally::new(1);
    ///    tally.add_weighted(vec!["Alice", "Bob"], 30);
    ///    tally.add_weighted(vec!["Bob", "Carol"], 10);
    ///    
    ///    for (candidate, rank) in tally.ranked().iter() {
    ///       println!("{} has a rank of {}", candidate, rank);
    ///    }
    ///    // Prints:
    ///    //   Bob has a rank of 0
    ///    //   Alice has a rank of 1
    ///    //   Carol has a rank of 2
    /// ```
    pub fn ranked(&self) -> Vec<(T, u32)> {
        return self.plurality.ranked();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approval_lumen() {
        // From: https://courses.lumenlearning.com/wmopen-mathforliberalarts/chapter/introduction-approval-voting/

        let matrix = "The Matrix";
        let scream = "Scream";
        let titanic = "Titanic";

        let mut tally = DefaultApprovalTally::new(1);
        tally.add_weighted(vec![scream, matrix], 3);
        tally.add_weighted(vec![titanic, matrix], 2);
        tally.add(vec![titanic, scream, matrix]);
        tally.add(vec![matrix]);
        tally.add(vec![titanic, scream]);
        tally.add(vec![titanic]);
        tally.add(vec![scream]);

        let totals = tally.totals();
        assert_eq!(totals, vec![(matrix, 7), (scream, 6), (titanic, 5)]);

        let ranked = tally.ranked();
        assert_eq!(ranked, vec![(matrix, 0), (scream, 1), (titanic, 2)]);

        let winners = tally.winners();
        assert_eq!(winners.contains(&matrix), true);
        assert_eq!(winners.contains(&scream), false);
        assert_eq!(winners.contains(&titanic), false);
    }
}
