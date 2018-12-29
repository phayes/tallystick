use num_traits::cast::NumCast;
use num_traits::Num;
use std::hash::Hash;
use std::ops::AddAssign;

use super::plurality::PluralityTally;
use super::result::RankedWinners;

pub type DefaultScoreTally<T> = ScoreTally<T, u64>;

pub struct ScoreTally<T, C = u64>
where
    T: Eq + Clone + Hash,                      // Candidate
    C: Copy + Ord + AddAssign + Num + NumCast, // vote count type
{
    plurality: PluralityTally<T, C>,
}

impl<T, C> ScoreTally<T, C>
where
    T: Eq + Clone + Hash,                      // Candidate
    C: Copy + Ord + AddAssign + Num + NumCast, // vote count type
{
    /// Create a new `ScoreTally` with the given number of winners.
    ///
    /// If there is a tie, the number of winners might be more than `num_winners`.
    /// (See [`winners()`](#method.winners) for more information on ties.)
    pub fn new(num_winners: u32) -> Self {
        return ScoreTally {
            plurality: PluralityTally::new(num_winners),
        };
    }

    /// Create a new `ApprovalTally` with the given number of winners, and number of expected candidates.
    pub fn with_capacity(num_winners: u32, expected_candidates: usize) -> Self {
        return ScoreTally {
            plurality: PluralityTally::with_capacity(num_winners, expected_candidates),
        };
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
        return self.plurality.candidates();
    }

    pub fn winners(&self) -> RankedWinners<T> {
        return self.plurality.winners();
    }

    pub fn totals(&self) -> Vec<(T, C)> {
        return self.plurality.totals();
    }

    pub fn ranked(&self) -> Vec<(T, u32)> {
        return self.plurality.ranked();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_test() {}
}
