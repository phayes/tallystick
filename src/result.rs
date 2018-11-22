use std::ops::RangeBounds;
use num_traits::Num;
use std::cmp::Ordering::Equal;

// A RankedWinner is a winner in an election, ranked ascending (starting from zero).
// A ranked-winner with a lower rank beats a ranked-winner with a higher rank.
// Ranked-winners with the same rank are tied.
type RankedWinner<T> = (T, u32);

/// `RankedWinners` is a ranked list of winning candidates, sorted according to rank. 
/// Ranks are in ascending order. A `0` ranked winner is more significant than a `3` ranked winner.
/// Winners with the same rank are tied.
// TODO: implement Index, IndexMut
#[derive(Debug, Eq, PartialEq, From, Default)]
pub struct RankedWinners<T: Clone> {
  winners: Vec<RankedWinner<T>>,
  num_winners: u32
}

impl<T: Clone + Eq> RankedWinners<T>  {

  /// Get the number of winners.
  pub fn len(&self) -> usize {
    return self.winners.len();
  }

  /// Check if it's empty
  pub fn is_empty(&self) -> bool {
    return self.winners.is_empty();
  }

  /// Clears the winners, returning all winner-rank pairs as an iterator.
  pub fn drain<R>(&mut self, range: R) -> std::vec::Drain<'_, RankedWinner<T>>
      where R: RangeBounds<usize>
  {
    return self.winners.drain(range);
  }

  /// Transform winners into a vector of winner-rank pairs.
  pub fn into_vec(self) ->Vec<RankedWinner<T>> {
    return self.winners;
  }

  /// Get a list of all winners, without rank.
  pub fn all(self) ->Vec<T> {
    let mut winners = Vec::<T>::with_capacity(self.len());
    for (winner, _) in self.winners.iter() {
      winners.push(winner.clone());
    }
    return winners;
  }

  /// Iterate over all winner->rank pairs
  pub fn iter(&self) -> IterWinners<'_, T> {
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

  /// Get the rank of a single winner
  pub fn rank(&self, candidate: &T) -> Option<u32> {
    for (winner, rank) in self.iter() {
      if candidate == winner {
        return Some(*rank);
      }
    }
    return None;
  }

  /// Get an unranked list of all winners, this consumes the winner list.
  pub fn into_unranked(mut self) -> Vec<T> {
    let mut all = Vec::new();
    for (winner, _rank) in self.drain(0..) {
      all.push(winner);
    }
    return all;
  }

  /// Check if the actual number of winners is more than the wanted number of winners.
  /// This can happen if there is a tie.
  /// 
  /// Not all ties result in an overflow. Only a tie of the least-significantly
  /// ranked winning candidates results in an overflow. Consider an election
  /// that is trying to fill three seats. If only the top two candidates tie, then
  /// there is no overflow. However, if the 3rd place and 4th place candidates tie,
  /// then there will be an overflow with both candidates being equally ranked to
  /// fill the 3rd seat.
  pub fn check_overflow(&self) -> bool {
    return self.len() > self.num_winners as usize;
  }

  /// Get all tied least-significantly ranked candidates that overflow the wanted number of winners.
  /// 
  /// If there is a tie in the least-significantly ranked winning candidates,
  /// then the actual number of winners will "overflow" the wanted number of winners, and this
  /// method will return a list of overlow candidates (or `None` if there is no overflow).
  /// 
  /// You should always check for an overflow so you can resolve this unfortunate situation.
  pub fn overflow(&self) -> Option<Vec<T>> {
    if self.check_overflow() {
      let mut overflow = Vec::<T>::new();
      let overflow_rank = self.winners[self.winners.len() -1].1;
      for (candidate, rank) in self.iter() {
        if *rank == overflow_rank {
          overflow.push(candidate.clone());
        }
      }
      return Some(overflow);
    }
    else {
      return None;
    }
  }

  // New empty list of ranked winners
  pub(crate) fn new(num_winners: u32) -> Self {
    return RankedWinners{
      winners: Vec::new(),
      num_winners: num_winners
    };
  }

  // Push a new winner onto the end of of the list of winners
  // Make sure to call sort() before passing the Winners back to the user.
  pub(crate) fn push(&mut self, candidate: T, rank: u32) {
    self.winners.push((candidate, rank));
  }

  // Sort the winners by rank.
  pub(crate) fn sort(&mut self) {
    self.winners.sort_by(|a, b| a.1.cmp(&b.1));
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
    return CountedCandidates(Vec::new());
  }

  /// Get the number of winners.
  pub(crate) fn len(&self) -> usize {
    self.0.len()
  }

  /// Transform winners into a vector of RankedWinners.
  /// Limit the number of winners by "num_winners", returned number may be over this if there is a tie
  /// Set num_winners to `0` for no limit.
  pub(crate) fn into_ranked(mut self, num_winners: u32) -> RankedWinners<T> {
    let mut ranked = RankedWinners::<T>::new(num_winners);

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

  /// Sort the candidates by tallied counts.
  // TODO: better handling of uncomparible (eg NaN) types
  //       one possibility is to check ordering against ::zero(), and order the offending value last. 
  pub(crate) fn sort(&mut self) {
    self.0.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Equal));
  }

}