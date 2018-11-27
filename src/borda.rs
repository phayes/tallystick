use super::plurality::Tally as PluralityTally;
use super::result::CountedCandidates;
use super::result::RankedWinners;
use super::Numeric;
use hashbrown::HashMap;
use hashbrown::HashSet;
use num_traits::Num;
use num_traits::NumCast;
use std::hash::Hash;
use std::ops::AddAssign;

/// Specifies method used to assign points to ranked candidates.
pub enum Variant {
  /// The standard Borda count where each candidate is assigned a number of points equal to the number of candidates ranked lower than them.
  /// It is known as the "Starting at 0" Borda count since the least-significantly ranked candidate is given zero points.
  /// Each candidate is given points according to:
  ///
  /// ```number-candidates - candidate-position - 1```
  ///
  /// Example point allocation for a single ballot:
  ///
  /// | Position on ballot  | Candiate | Points |
  /// | --------------------|----------|--------|
  /// | 0                   | Alice    | 3      |
  /// | 1                   | Bob      | 2      |
  /// | 2                   | Carlos   | 1      |
  /// | 3                   | Dave     | 0      |
  Borda,

  /// The classic Borda count as defined in Jean-Charles de Borda's [original proposal](http://gerardgreco.free.fr/IMG/pdf/MA_c_moire-Borda-1781.pdf).
  /// It is known as the "Starting at 1" Borda count since the least-significantly ranked candidate is given one point.
  /// Each candidate is given points according to:
  ///
  /// ```number-candidates - candidate-position```
  ///
  /// Example point allocation for a single ballot:
  ///
  /// | Position on ballot  | Candiate | Points |
  /// | --------------------|----------|--------|
  /// | 0                   | Alice    | 4      |
  /// | 1                   | Bob      | 3      |
  /// | 2                   | Carlos   | 2      |
  /// | 3                   | Dave     | 1      |
  ClassicBorda,

  /// In the Dowdall system, the highest-ranked candidate obtains 1 point, while the 2nd-ranked candidate receives ½ a point, the 3rd-ranked candidate receives ⅓ of a point, etc.
  /// An important difference of this method from the others is that the number of points assigned to each preference does not depend on the number of candidates.
  /// Each candidate is given points according to:
  ///
  /// ```1 / (candidate-position + 1)```
  ///
  /// If Dowdall is selected, tallyman will panic if an integer count type is used in the tally. This variant should only be used with a float or rational tally.
  ///
  /// Example point allocation for a single ballot:
  ///
  /// | Position on ballot  | Candiate | Points |
  /// | --------------------|----------|--------|
  /// | 0                   | Alice    | 1      |
  /// | 1                   | Bob      | ½      |
  /// | 2                   | Carlos   | ⅓      |
  /// | 3                   | Dave     | ¼      |
  ///
  /// Example:
  /// ```
  /// use tallyman::borda::BordaTally;
  /// use tallyman::borda::Variant;
  ///
  /// // Note use of `f64` as our count type.
  /// let mut tally = BordaTally::<&str, f64>::new(1, Variant::Dowdall);
  /// tally.add(vec!["Barak Obama", "John McCain"]);
  /// tally.add(vec!["Barak Obama", "Mitt Romney"]);
  /// let _winners = tally.winners();
  /// ```
  Dowdall,

  /// In a modified Borda count, the number of points given for a voter's first and subsequent preferences is determined by the total number of candidates they have actually ranked, rather than the total number listed.
  /// This is to say, typically, on a ballot of `n` candidates, if a voter casts only `m` preferences (where `n ≥ m ≥ 1`), a first preference gets `m - 1` points, a second preference `m – 2` points, and so on.
  /// Modified Borda counts are used to counteract the problem of [bullet voting](https://en.wikipedia.org/wiki/Bullet_voting).
  /// Each candidate is given points according to:
  ///
  /// ```number-marked - candidate-position - 1```
  ModifiedBorda,

  /// A modified classic Borda count that gives the least significantly ranked candidate 1 point.
  /// Like [`ModifiedBorda`](#variant.ModifiedBorda), it only assigns points according to the number of marked candidates on each ballot, instead of the total number of candidates.
  /// Each candidate is given points according to:
  ///
  /// ```number-marked - candidate-position```
  ModifiedClassicBorda,
}

impl Variant {
  // TODO: Panic if we are using Dowdall without a Float C, specialization?
  pub fn points<C: Numeric + Num + NumCast>(&self, candidate_position: usize, num_candidates: usize, num_marked: usize) -> C {
    match self {
      Variant::Borda => C::from(num_candidates - candidate_position - 1).unwrap(),
      Variant::ClassicBorda => C::from(num_candidates - candidate_position).unwrap(),
      Variant::Dowdall => {
        if !C::fraction() {
          panic!("tallyman::borda::Variant::Dowdall cannot be used with an integer count type. Please use a float or a rational.")
        }
        C::one() / C::from(candidate_position + 1).unwrap()
      }
      Variant::ModifiedBorda => C::from(num_marked - candidate_position - 1).unwrap(),
      Variant::ModifiedClassicBorda => C::from(num_marked - candidate_position).unwrap(),
    }
  }
}

pub type DefaultBordaTally<T> = BordaTally<T, u64>;

pub struct BordaTally<T, C = u64>
where
  T: Eq + Clone + Hash,                             // Candidate
  C: Copy + PartialOrd + AddAssign + Num + NumCast, // Vote count type
{
  running_total: HashMap<Vec<T>, C>,
  candidates: HashSet<T>,
  num_winners: u32,
  variant: Variant,
}

impl<T, C> BordaTally<T, C>
where
  T: Eq + Clone + Hash,                             // Candidate
  C: Copy + PartialOrd + AddAssign + Num + NumCast, // Vote count type
{
  pub fn new(num_winners: u32, variant: Variant) -> Self {
    return BordaTally {
      running_total: HashMap::new(),
      candidates: HashSet::new(),
      num_winners: num_winners,
      variant: variant,
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
    let mut counted = CountedCandidates::new();
    for (candidate, votecount) in self.totals().iter() {
      counted.push(candidate.clone(), *votecount);
    }
    return counted.into_ranked(self.num_winners);
  }

  pub fn totals(&self) -> Vec<(T, C)> {
    // Make a little plurality tally and use borda points as weights
    let mut plurality = PluralityTally::with_capacity(self.num_winners, self.candidates.len());
    for (selection, votecount) in self.running_total.iter() {
      let num_marked = selection.len();
      for (position, candidate) in selection.iter().enumerate() {
        let points: C = self.variant.points(position, self.candidates.len(), num_marked);
        plurality.add_weighted_ref(candidate, *votecount * points);
      }
    }

    return plurality.totals();
  }
}

pub type DefaultNansonTally<T> = NansonTally<T, u64>;

pub struct NansonTally<T, C = u64>
where
  T: Eq + Clone + Hash,                             // Candidate
  C: Copy + PartialOrd + AddAssign + Num + NumCast, // Vote count type
{
  borda: BordaTally<T, C>,
}

pub type DefaultBaldwinTally<T> = BaldwinTally<T, u64>;

pub struct BaldwinTally<T, C = u64>
where
  T: Eq + Clone + Hash,                             // Candidate
  C: Copy + PartialOrd + AddAssign + Num + NumCast, // Vote count type
{
  borda: BordaTally<T, C>,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn borda_test() {
    // From: https://en.wikipedia.org/wiki/Borda_count
    let mut borda_tally = DefaultBordaTally::new(1, Variant::Borda);
    borda_tally.add_weighted(vec!["Andrew", "Catherine", "Brian", "David"], 51);
    borda_tally.add_weighted(vec!["Catherine", "Brian", "David", "Andrew"], 5);
    borda_tally.add_weighted(vec!["Brian", "Catherine", "David", "Andrew"], 23);
    borda_tally.add_weighted(vec!["David", "Catherine", "Brian", "Andrew"], 21);

    assert!(borda_tally.totals() == vec![("Catherine", 205), ("Andrew", 153), ("Brian", 151), ("David", 91)]);
    assert!(borda_tally.winners().into_unranked() == vec!["Catherine"]);

    let mut classic_tally = DefaultBordaTally::new(1, Variant::ClassicBorda);
    classic_tally.add_weighted(vec!["Andrew", "Catherine", "Brian", "David"], 51);
    classic_tally.add_weighted(vec!["Catherine", "Brian", "David", "Andrew"], 5);
    classic_tally.add_weighted(vec!["Brian", "Catherine", "David", "Andrew"], 23);
    classic_tally.add_weighted(vec!["David", "Catherine", "Brian", "Andrew"], 21);

    assert!(classic_tally.totals() == vec![("Catherine", 305), ("Andrew", 253), ("Brian", 251), ("David", 191)]);
    assert!(classic_tally.winners().into_unranked() == vec!["Catherine"]);

    let mut dowdall_tally = BordaTally::<&str, f64>::new(1, Variant::Dowdall);
    dowdall_tally.add_weighted(vec!["Andrew", "Catherine", "Brian", "David"], 51.0);
    dowdall_tally.add_weighted(vec!["Catherine", "Brian", "David", "Andrew"], 5.0);
    dowdall_tally.add_weighted(vec!["Brian", "Catherine", "David", "Andrew"], 23.0);
    dowdall_tally.add_weighted(vec!["David", "Catherine", "Brian", "Andrew"], 21.0);

    assert!(
      dowdall_tally.totals()
        == vec![
          ("Andrew", 63.25),
          ("Catherine", 52.5),
          ("Brian", 49.5),
          ("David", 43.08 + (0.01 / 3.0))
        ]
    );
    assert!(dowdall_tally.winners().into_unranked() == vec!["Andrew"]);

    let mut tally = DefaultBordaTally::new(1, Variant::Borda);
    tally.add_weighted(vec!["Memphis", "Nashville", "Chattanooga", "Knoxville"], 42);
    tally.add_weighted(vec!["Nashville", "Chattanooga", "Knoxville", "Memphis"], 26);
    tally.add_weighted(vec!["Chattanooga", "Knoxville", "Nashville", "Memphis"], 15);
    tally.add_weighted(vec!["Knoxville", "Chattanooga", "Nashville", "Memphis"], 17);
    assert!(tally.totals() == vec![("Nashville", 194), ("Chattanooga", 173), ("Memphis", 126), ("Knoxville", 107)]);
    assert!(tally.winners().into_unranked() == vec!["Nashville"]);
  }
}
