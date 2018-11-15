use std::ops::RangeBounds;
use num_traits::Num;

/// A RankedWinner is a winner in an election, ranked ascending (starting from zero).
/// A ranked-winner with a lower rank beats a ranked-winner with a higher rank.
/// Ranked-winners with the same rank are tied.
type RankedWinner<T> = (T, u32);

/// Winners is a list of ranked-candidates, sorted according to rank. 
/// Note that this is not totally sorted, since ranked-candidates with the same rank are tied.
#[derive(Debug, Eq, PartialEq, From, Index, IndexMut, Default)]
pub struct RankedWinners<T: Clone>(Vec<RankedWinner<T>>);


impl<T: Clone + Eq> RankedWinners<T>  {

  /// Get the number of winners.
  pub fn len(&self) -> usize {
    return self.0.len();
  }

  /// Clears the winners, returning all winner-rank pairs as an iterator.
  pub fn drain<R>(&mut self, range: R) -> std::vec::Drain<'_, RankedWinner<T>>
      where R: RangeBounds<usize>
  {
    return self.0.drain(range);
  }

  /// Transform winners into a vector of RankedWinners.
  pub fn into_vec(self) ->Vec<RankedWinner<T>> {
    return self.0;
  }

  /// Get a list of all winners, without rank.
  pub fn all(self) ->Vec<T> {
    let mut winners = Vec::<T>::with_capacity(self.len());
    for (winner, _) in self.0.iter() {
      winners.push(winner.clone());
    }
    return winners;
  }

  /// Iterate over all winner->rank pairs
  pub fn iter<'a>(&'a self) -> IterWinners<'a, T> {
    IterWinners {
        inner: self,
        pos: 0,
    }
  }

  /// Check if the given candidate exists in the set of ranked-winners
  pub fn contains(&self, candidate: &T) -> bool {
    for (winner, _rank) in self.iter() {
      if candidate == winner {
        return true;
      }
    }
    return false;
  }

  /// Get an unranked list of all winners, this consumes the winner list.
  pub fn into_unranked(mut self) -> Vec<T> {
    let mut all = Vec::new();
    for (winner, _rank) in self.drain(0..) {
      all.push(winner);
    }
    return all;
  }

  // New empty list of ranked winners
  pub(crate) fn new() -> Self {
    return RankedWinners(Vec::new());
  }

  // Push a new winner onto the end of of the list of winners
  // Make sure to call sort() before passing the Winners back to the user.
  pub(crate) fn push(&mut self, candidate: T, rank: u32) {
    self.0.push((candidate, rank));
  }

  // Sort the winners by rank.
  pub(crate) fn sort(&mut self) {
    self.0.sort_by(|a, b| a.1.cmp(&b.1));
  }

}


// Iterator for Winners
// TODO: Use some sort of a macro to auto-generate this.
// ---------------------

/// Iterator for winners
pub struct IterWinners<'a, T: Clone> {
    inner: &'a RankedWinners<T>,
    pos: usize,
}

impl<'a, T: Clone> Iterator for IterWinners<'a, T> {
    type Item = &'a RankedWinner<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.inner.0.len() {
            None
        } else {
            self.pos += 1;
            self.inner.0.get(self.pos - 1)
        }
    }
}

#[derive(Debug, Eq, PartialEq, From, Index, IndexMut, Default)]
pub(crate) struct CountedCandidates<T: Clone + Eq, C: Copy + Num + Ord>(Vec<(T, C)>);

impl<T: Clone + Eq, C: Copy + Num + Ord> CountedCandidates<T, C> {

  // New empty list of counted candidates
  pub(crate) fn new() -> Self {
    return CountedCandidates(Vec::new());
  }

  /// Get the number of winners.
  pub(crate) fn len(&self) -> usize {
    return self.0.len();
  }

  /// Transform winners into a vector of RankedWinners.
  /// Limit the number of winners by "num_winners", returned number may be over this if there is a tie
  /// Set num_winners to `0` for no limit.
  pub(crate) fn into_ranked(mut self, num_winners: u32) -> RankedWinners<T> {
    let mut ranked = RankedWinners::<T>::new();

    if self.len() == 0 {
      return ranked;
    }

    self.sort();

    let mut rank = 0;
    let mut prev = self.0[0].1;
    for (candidate, score) in self.0.drain(0..) {
      if score != prev {
        if num_winners != 0 && ranked.len() as u32 >= num_winners {
          return ranked;
        }
        rank += 1;
      }
      ranked.push(candidate, rank);
      prev = score;
    }

    return ranked;
  }

  // Push a new winner onto the end of of the list of winners
  // Make sure to call sort() before passing the Winners back to the user.
  pub(crate) fn push(&mut self, candidate: T, count: C) {
    self.0.push((candidate, count));
  }

  // Sort the candidates by tallied counts.
  pub(crate) fn sort(&mut self) {
    self.0.sort_by(|a, b| b.1.cmp(&a.1));
  }

}