use derive_more::{From, Index, IndexMut};
use num_traits::Num;
use std::cmp::Ordering::Equal;
use std::ops::RangeBounds;

// A RankedCandidate is candidate in an election, ranked ascending (starting from zero).
// A ranked-candidate with a lower rank beats a ranked-candidate with a higher rank.
// Ranked-candidates with the same rank are tied.
#[derive(Debug, Eq, PartialEq, From, Default)]
pub struct RankedCandidate<T: Clone + Eq + PartialEq> {
    pub candidate: T,
    pub rank: usize,
}

impl<T: Eq + PartialEq + Clone> PartialEq<(T, usize)> for RankedCandidate<T> {
    fn eq(&self, other: &(T, usize)) -> bool {
        self.candidate == other.0 && self.rank == other.1
    }
}

/// `RankedWinners` is a ranked list of winning candidates, sorted according to rank.
/// Ranks are in ascending order. A `0` ranked winner is more significant than a `3` ranked winner.
/// Winners with the same rank are tied.
// TODO: implement Index, IndexMut
#[derive(Debug, Eq, PartialEq, From, Default)]
pub struct RankedWinners<T: Clone + Eq + PartialEq> {
    /// Ranked winners
    pub winners: Vec<RankedCandidate<T>>,

    /// Number of winners, this number could be less than winners.len() if there are ties in the lowest ranked winners.
    pub num_winners: usize,
}

impl<T: Clone + Eq + PartialEq> RankedWinners<T> {
    /// Get the number of winners.
    pub fn len(&self) -> usize {
        self.winners.len()
    }

    /// Check if it's empty
    pub fn is_empty(&self) -> bool {
        self.winners.is_empty()
    }

    /// Clears the winners, returning all winner-rank pairs as an iterator.
    pub fn drain<R>(&mut self, range: R) -> std::vec::Drain<'_, RankedCandidate<T>>
    where
        R: RangeBounds<usize>,
    {
        self.winners.drain(range)
    }

    /// Transform winners into a vector of winner-rank pairs.
    pub fn into_vec(self) -> Vec<RankedCandidate<T>> {
        self.winners
    }

    /// Get a list of all winners, without rank.
    pub fn all(&self) -> Vec<T> {
        let mut winners = Vec::<T>::with_capacity(self.len());
        for ranked in self.winners.iter() {
            winners.push(ranked.candidate.clone());
        }

        winners
    }

    /// Iterate over all winner->rank pairs.
    pub fn iter(&self) -> IterWinners<'_, T> {
        IterWinners { inner: self, pos: 0 }
    }

    /// Check if the given candidate exists in the set of ranked-winners.
    pub fn contains(&self, candidate: &T) -> bool {
        for ranked in self.iter() {
            if candidate == &ranked.candidate {
                return true;
            }
        }

        false
    }

    /// Get the rank of a single winner.
    pub fn rank(&self, candidate: &T) -> Option<usize> {
        for ranked in self.iter() {
            if candidate == &ranked.candidate {
                return Some(ranked.rank);
            }
        }

        None
    }

    /// Get an unranked list of all winners, this consumes the winner list.
    pub fn into_unranked(mut self) -> Vec<T> {
        let mut all = Vec::new();
        for ranked in self.drain(0..) {
            all.push(ranked.candidate);
        }

        all
    }

    /// Check if the actual number of winners is more than the wanted number of winners.
    /// This can happen if there is a tie.
    ///
    /// Not all ties result in an overflow. Only a tie of the least-significantly
    /// ranked winning candidates can result in an overflow. Consider an election
    /// that is trying to fill three seats. If only the top two candidates tie, then
    /// there is no overflow. However, if the 3rd place and 4th place candidates tie,
    /// then there will be an overflow with both candidates being equally ranked to
    /// fill the 3rd seat.
    pub fn check_overflow(&self) -> bool {
        self.len() > self.num_winners
    }

    /// Get all tied least-significantly ranked winners that overflow the wanted number of winners.
    ///
    /// If there is a tie in the least-significantly ranked winning candidates,
    /// then the actual number of winners may "overflow" the wanted number of winners, in which case this
    /// method will return a list of overlow candidates (or `None` if there is no overflow).
    ///
    /// You should always check for an overflow so you can resolve this unfortunate situation.
    pub fn overflow(&self) -> Option<Vec<T>> {
        if self.check_overflow() {
            let mut overflow = Vec::<T>::new();
            let overflow_rank = self.winners[self.winners.len() - 1].rank;
            for ranked in self.iter() {
                if ranked.rank == overflow_rank {
                    overflow.push(ranked.candidate.clone());
                }
            }
            Some(overflow)
        } else {
            None
        }
    }

    // New empty list of ranked winners
    pub(crate) fn new(num_winners: usize) -> Self {
        RankedWinners {
            winners: Vec::new(),
            num_winners: num_winners,
        }
    }

    // Push a new winner onto the end of of the list of winners
    // Make sure to call sort() before passing the Winners back to the user.
    pub(crate) fn push(&mut self, candidate: T, rank: usize) {
        self.winners.push((candidate, rank).into());
    }

    // Sort the winners by rank.
    pub(crate) fn sort(&mut self) {
        self.winners.sort_by(|a, b| a.rank.cmp(&b.rank));
    }

    // Create winners from a list of ranked candidates
    pub(crate) fn from_ranked(mut ranked: Vec<(T, usize)>, num_winners: usize) -> Self {
        let mut winners = Self::new(num_winners);
        let mut prev_rank = 0;
        for (candidate, rank) in ranked.drain(0..) {
            if winners.len() >= num_winners && rank != prev_rank {
                break;
            }
            winners.push(candidate, rank);
            prev_rank = rank;
        }
        winners.sort();

        winners
    }
}

// Iterator for Winners
// TODO: Use some sort of a macro to auto-generate this.
// ---------------------

/// Iterator for winners
pub struct IterWinners<'a, T: Clone + Eq + PartialEq> {
    inner: &'a RankedWinners<T>,
    pos: usize,
}

impl<'a, T: Clone + Eq + PartialEq> Iterator for IterWinners<'a, T> {
    type Item = &'a RankedCandidate<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.inner.winners.len() {
            None
        } else {
            self.pos += 1;
            self.inner.winners.get(self.pos - 1)
        }
    }
}

#[derive(Debug, Eq, PartialEq, From, Index, IndexMut, Default)]
pub(crate) struct CountedCandidates<T: Clone + Eq, C: Copy + Num + PartialOrd>(Vec<(T, C)>);

impl<T: Clone + Eq, C: Copy + Num + PartialOrd> CountedCandidates<T, C> {
    // New empty list of counted candidates
    pub(crate) fn new() -> Self {
        CountedCandidates(Vec::new())
    }

    /// Get the number of winners.
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    /// Transform candidates into a vector of RankedWinners.
    /// Limit the number of winners by "num_winners", returned number may be over this if there is a tie
    /// Set num_winners to `0` for no limit.
    pub(crate) fn into_ranked(mut self, num_winners: usize) -> RankedWinners<T> {
        let mut ranked = RankedWinners::<T>::new(num_winners);

        if self.len() == 0 {
            return ranked;
        }

        self.sort();

        let mut rank = 0;
        let mut prev = self.0[0].1;
        for (candidate, score) in self.0.drain(0..) {
            if score != prev {
                if num_winners != 0 && ranked.len() >= num_winners {
                    return ranked;
                }
                rank += 1;
            }
            ranked.push(candidate, rank);
            prev = score;
        }

        ranked
    }

    // Transform into a vector
    pub(crate) fn into_vec(mut self) -> Vec<(T, C)> {
        self.sort();
        self.0
    }

    // Push a new winner onto the end of of the list of winners
    // Make sure to call sort() before passing the Winners back to the user.
    pub(crate) fn push(&mut self, candidate: T, count: C) {
        self.0.push((candidate, count));
    }

    /// Sort the candidates by tallied counts.
    // TODO: better handling of uncomparible (eg NaN) types
    //       one possibility is to check ordering against ::zero(), and order the offending value last.
    pub(crate) fn sort(&mut self) {
        self.0.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Equal));
    }
}
