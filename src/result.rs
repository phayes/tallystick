use std::ops::{Index, IndexMut, RangeBounds};

#[derive(Debug, Eq, PartialEq)]
pub struct RankedWinner<T>(T, u32);

#[derive(Debug, Eq, PartialEq, From, Index, IndexMut, Default, Constructor)]
pub struct Result<T>(Vec<RankedWinner<T>>);

impl<T> Result<T>  {

  pub fn len(&self) -> usize {
    return self.0.len();
  }

  pub fn push(&mut self, value: T, rank: u32) {
    self.0.push(RankedWinner(value, rank));
  }

  pub fn drain<R>(&mut self, range: R) -> std::vec::Drain<RankedWinner<T>>
      where R: RangeBounds<usize>
  {
    return self.0.drain(range);
  }

  pub fn into_vec(self) ->Vec<RankedWinner<T>> {
    return self.0;
  }

}

