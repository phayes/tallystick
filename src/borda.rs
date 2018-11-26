use std::hash::Hash;
use hashbrown::HashMap;
use hashbrown::HashSet;
use num_traits::Num;
use num_traits::cast::NumCast;
use std::ops::AddAssign;
use super::result::CountedCandidates;
use super::result::RankedWinners;
use super::plurality::Tally as PluralityTally;

pub enum Borda {

  /// Standard Borda (least ranked candidate gets zero points)
  Borda,

  /// Standard Borda starting at 1 (classic border)
  ClassicBorda,

  Dowdall,

  ModifiedBorda,

  ModifiedClassicBorda
}

impl Borda {
  // TODO: Panic if we are using Dowdall without a Float C, specialization?
  pub fn points<C: Num + NumCast>(&self, candidate_position: usize, num_candidates: usize, num_marked: usize) -> C {
    match self {
      Borda::Borda => C::from(num_candidates - candidate_position -1).unwrap(),
      Borda::ClassicBorda => C::from(num_candidates - candidate_position).unwrap(),
      Borda::Dowdall => C::from(num_candidates).unwrap() / C::from(candidate_position + 1).unwrap(),
      Borda::ModifiedBorda => C::from(num_marked - candidate_position -1).unwrap(),
      Borda::ModifiedClassicBorda => C::from(num_marked - candidate_position).unwrap(),
    }
  }
}


pub type DefaultTally<T> = Tally<T, u64>;

pub struct Tally<T, C = u64>
    where T: Eq + Clone + Hash,        // Candidate
          C: Copy + PartialOrd + AddAssign + Num + NumCast // Vote count type
{
    running_total: HashMap<Vec<T>, C>,
    candidates: HashSet<T>,
    num_winners: u32,
    variation: Borda
}

impl<T, C> Tally<T, C>
    where T: Eq + Clone + Hash,        // Candidate
          C: Copy + PartialOrd + AddAssign + Num + NumCast // Vote count type
{
    pub fn new(num_winners: u32, variation: Borda) -> Self {
        return Tally {
            running_total: HashMap::new(),
            candidates: HashSet::new(),
            num_winners: num_winners,
            variation: variation
        };
    }

    pub fn add(&mut self, selection: Vec<T>) {
      self.add_weighted(selection, C::one());
    }

    pub fn add_ref(&mut self, selection: &[T]) {
        self.add_weighted_ref(selection, C::one());
    }

    pub fn add_weighted(&mut self, selection: Vec<T>, weight: C) {
      for candidate in selection.iter() {
        if !self.candidates.contains(candidate) {
          self.candidates.insert(candidate.clone());
        }
      }

      let entry = self.running_total.entry(selection);
      *entry.or_insert(C::zero()) += weight;
    }

    pub fn add_weighted_ref(&mut self, selection: &[T], weight: C) {
      for candidate in selection.iter() {
        if !self.candidates.contains(candidate) {
          self.candidates.insert(candidate.clone());
        }
      }

      let entry = self.running_total.entry(selection.to_vec());
      *entry.or_insert(C::zero()) += weight;
    }

    pub fn winners(&self) -> RankedWinners<T> {
      // Make a little plurality tally and use borda points as weights
      let mut plurality = PluralityTally::with_capacity(self.num_winners, self.candidates.len());
      for (selection, votecount) in self.running_total.iter() {
        let num_marked = selection.len();
        for (position, candidate) in selection.iter().enumerate() {
          let points: C = self.variation.points(position, self.candidates.len(), num_marked);
          plurality.add_weighted_ref(candidate, *votecount * points);
        }
      }
      return plurality.winners();
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn borda_test() {
      
    }
}
