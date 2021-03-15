//! An Instant-Runoff Voting implementation.
//!
//! This module contains the implementation for instant-runoff voting, which is sometimes called
//! ranked choice voting. More information about this voting method can be found at
//! <https://en.wikipedia.org/wiki/Instant-runoff_voting>.

use std::cmp::Ord;
use std::hash::Hash;
use std::ops::AddAssign;

use hashbrown::HashSet;
use num_traits::{cast::NumCast, Num};

use crate::{Numeric, RankedWinners, Transfer, VoteTree};

#[derive(Debug)]
struct WeightedVote<T, C>
where
    T: Eq + Clone + Hash,                                       // Candidate
    C: Copy + PartialOrd + AddAssign + Num + NumCast + Numeric, // vote count type
{
    weight: C,
    remaining: Vec<T>,
}

/// A default [`Tally`] struct, using [`u64`] as the vote count type.
pub type DefaultTally<T> = Tally<T, u64>;

/// A custom [`Tally`] struct for instant runoff voting with the option to change vote count types.
///
/// If your votes can be counted using the [`u64`] type, then the [`DefaultTally`] type may be of
/// use to you.
pub struct Tally<T, C>
where
    T: Eq + Clone + Hash,                                       // Candidate
    C: Copy + PartialOrd + AddAssign + Num + NumCast + Numeric, // vote count type
{
    running_total: VoteTree<T, C>,
    transfer: Transfer,
}

impl<T, C> Tally<T, C>
where
    T: Eq + Clone + Hash,                                             // Candidate
    C: Copy + PartialOrd + Ord + AddAssign + Num + NumCast + Numeric, // vote count type
{
    /// Creates a new [`Tally`] instance using a given vote [`Transfer`] method.
    ///
    /// # Examples
    ///
    /// ```
    /// use tallystick::{irv::Tally, Transfer};
    ///
    /// let tally: Tally<i32, u64> = Tally::new(Transfer::Meek);
    /// ```
    pub fn new(transfer: Transfer) -> Self {
        Tally {
            running_total: VoteTree::new(),
            transfer,
        }
    }

    /// Creates a new [`Tally`] instance with a predefined set of candidates.
    ///
    /// # Examples
    ///
    /// ```
    /// use tallystick::{irv::Tally, Transfer};
    ///
    /// let candidates = vec![3, 1, 2, 4];
    /// let tally: Tally<i32, u64> = Tally::with_candidates(Transfer::Meek, candidates);
    /// ```
    pub fn with_candidates(transfer: Transfer, candidates: Vec<T>) -> Self {
        Tally {
            running_total: VoteTree::with_candidates(candidates),
            transfer,
        }
    }

    /// Adds votes to the tally from a [`Vec`], weighting them by the given value. This can be used
    /// to allow some ballots/votes to count more or less than others.
    ///
    /// # Examples
    ///
    /// ```
    /// use tallystick::{irv::Tally, Transfer};
    ///
    /// let mut tally: Tally<i32, u64> = Tally::new(Transfer::Meek);
    /// let votes = vec![3, 1, 2, 4];
    ///
    /// tally.add_weighted(votes, 3);
    /// ```
    pub fn add_weighted(&mut self, selection: Vec<T>, weight: C) {
        self.running_total.add(&selection, weight);
    }

    /// Adds votes to the tally from a [`Vec`], consuming it in the process.
    ///
    /// # Examples
    ///
    /// ```
    /// use tallystick::{irv::Tally, Transfer};
    ///
    /// let mut tally: Tally<i32, u64> = Tally::new(Transfer::Meek);
    /// let votes = vec![3, 1, 2, 4];
    ///
    /// tally.add(votes);
    /// ```
    pub fn add(&mut self, selection: Vec<T>) {
        // TODO: split add and add_ref in VoteTree
        self.running_total.add(&selection, C::one());
    }

    /// Adds votes to the tally from a [`slice`]. As a [`Vec`] can be dereference into a [`slice`],
    /// this allows votes to be added without consuming the [`Vec`] unlike [`Tally::add`].
    ///
    /// # Examples
    ///
    /// ```
    /// use tallystick::{irv::Tally, Transfer};
    ///
    /// let mut tally: Tally<i32, u64> = Tally::new(Transfer::Meek);
    /// let votes = vec![3, 1, 2, 4];
    ///
    /// tally.add_ref(&votes);
    /// ```
    pub fn add_ref(&mut self, selection: &[T]) {
        // TODO: split add and add_ref in VoteTree
        self.running_total.add(selection, C::one());
    }

    /// Calculates the current state of the votes and produces a ranked list of candidates.
    ///
    /// Each candidate that has been voted for is contained within the returned [`Vec`], alongside
    /// a ranking stating where they currently stand in the totals. If two candidates are even with
    /// each other, they will have the same rank. The [`Vec`] will be sorted based on the ranking
    /// of each candidate in ascending order, with lower ranking implying more votes.
    ///
    /// # Examples
    ///
    /// ```
    /// use tallystick::{irv::Tally, Transfer};
    ///
    /// let mut tally: Tally<i32, u64> = Tally::new(Transfer::Meek);
    /// let ballots = vec![
    ///     vec![3, 1, 2],
    ///     vec![3, 2],
    ///     vec![2, 3],
    ///     vec![3, 1, 2],
    /// ];
    ///
    /// for ballot in ballots {
    ///     tally.add_ref(&ballot);
    /// }
    ///
    /// let ranked = tally.tally_ranked();
    /// let expected = vec![
    ///     (3, 0),
    ///     (2, 1),
    ///     (1, 2),
    /// ];
    ///
    /// assert_eq!(ranked, expected);
    /// ```
    pub fn tally_ranked(&self) -> Vec<(T, usize)> {
        let max = C::max_value();

        let candidates = self.running_total.candidates();
        let mut inverse_ranked = Vec::<(T, usize)>::with_capacity(candidates.len());
        let mut inverse_rank = 0;
        let mut eliminated = HashSet::new();

        loop {
            // First Eagerly assign tally passing through eliminated candidates
            let (_excess, mut score) = self.running_total.assign_votes(&eliminated);

            // Scores lack elements for which there is not direct vote; we have to supply
            // for elimination to verbosely mention them.
            // Otherwise, some candidate may appear on one round after it should be eliminated.
            for c in candidates.iter() {
                if !eliminated.contains(c) {
                    score.entry(c.clone()).or_insert(C::zero());
                }
            }

            // If there are no more valid candidates, return early
            if score.is_empty() {
                return inverse_ranked;
            }

            // Check for case where all remaining candidates are tied
            let mut all_counts = Vec::new();
            let mut all_tied = true;
            for s in score.iter() {
                for check_count in all_counts.iter() {
                    if s.1 != check_count {
                        all_tied = false;
                        break;
                    }
                }
                all_counts.push(s.1.clone());
            }
            if all_tied {
                for (cand, _) in score {
                    inverse_ranked.push((cand, inverse_rank));
                }
                break;
            }

            // Calculate the worst performing candidates and add them to the reverse ranking
            let min_score = score.values().min().unwrap_or(&max);
            let mut loosers = Vec::<T>::new();
            for (cand, count) in score.iter() {
                if count == min_score {
                    loosers.push(cand.clone());
                }
            }
            if loosers.is_empty() {
                // No one left
                break;
            }

            // Remove all loosers
            for looser in loosers.drain(..) {
                inverse_ranked.push((looser.clone(), inverse_rank));
                eliminated.insert(looser);
            }

            inverse_rank += 1;
        }

        let num_ranked = inverse_ranked.len();
        let mut ranked = Vec::<(T, usize)>::with_capacity(num_ranked);
        for (cand, inverse_rank) in inverse_ranked.drain(..).rev() {
            ranked.push((cand, num_ranked - inverse_rank - 1));
        }
        ranked
    }

    /// Wraps the result of [`Tally::tally_ranked`] in a [`RankedWinners`] with 1 winner.
    ///
    /// # Examples
    ///
    /// ```
    /// use tallystick::{RankedWinners, irv::Tally, Transfer};
    ///
    /// let mut tally: Tally<i32, u64> = Tally::new(Transfer::Meek);
    /// let ballots = vec![
    ///     vec![3, 1, 2, 4],
    /// ];
    ///
    /// for ballot in ballots {
    ///     tally.add_ref(&ballot);
    /// }
    ///
    /// let winners = tally.tally_winners();
    /// let ranked_winners = RankedWinners::from((vec![(3, 2)], 1));
    ///
    /// assert_eq!(winners, ranked_winners);
    /// ```
    pub fn tally_winners(&self) -> RankedWinners<T> {
        RankedWinners::from_ranked(self.tally_ranked(), 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::TallyError;

    #[test]
    fn irv_wikipedia_1() -> Result<(), TallyError> {
        // See: https://en.wikipedia.org/wiki/Instant-runoff_voting

        let mut tally = DefaultTally::new(Transfer::Meek);
        tally.add_weighted(vec!["Memphis", "Nashville", "Chattanooga", "Chattanooga"], 42);
        tally.add_weighted(vec!["Nashville", "Chattanooga", "Knoxville", "Memphis"], 26);
        tally.add_weighted(vec!["Chattanooga", "Knoxville", "Nashville", "Nashville"], 15);
        tally.add_weighted(vec!["Knoxville", "Chattanooga", "Nashville", "Memphis"], 17);

        // Verify winners
        let winners = tally.tally_winners();
        assert!(winners.contains(&"Knoxville"));

        // Verify ranking
        let ranked = tally.tally_ranked();
        assert_eq!(ranked[0].0, "Knoxville");
        assert_eq!(ranked[0].1, 0);
        assert_eq!(ranked[1].0, "Memphis");
        assert_eq!(ranked[1].1, 1);
        assert_eq!(ranked[2].0, "Nashville");
        assert_eq!(ranked[2].1, 2);
        assert_eq!(ranked[3].0, "Chattanooga");
        assert_eq!(ranked[3].1, 3);

        Ok(())
    }
}
