use super::check_duplicate;
use super::plurality::PluralityTally;
use super::result::CountedCandidates;
use super::result::RankedWinners;
use super::Numeric;
use super::TallyError;
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
  /// Get the number of points for a candidate at a certain position on a ballot.
  ///
  /// - `candidate_position` is the position of the candidate on the marked ballot. It is `0` for the 1st candidate, `1` for the second candidate etc.
  /// - `num_candidates` is the total number of candidates in this election.
  /// - `num_marked` is the total number of candidates marked on the ballot.
  ///
  /// This method will panic if using `Dowdall` with an integer based vote-count type.
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

/// A borda tally using `u64` integers to count votes.
/// `DefaultBordaTally` is generally preferred over `BordaTally`, except when using the `Variant::Dowdall` variant.
/// Since this is an alias, refer to [`BordaTally`](struct.BordaTally.html) for method documentation.
///
/// # Example
/// ```
///    use tallyman::borda::DefaultBordaTally;
///    use tallyman::borda::Variant;
///
///    // What is your favourite colour?
///    // A vote with hexadecimal colour candidates and a single-winner.
///    let red = 0xff0000;
///    let blue = 0x00ff00;
///    let green = 0x0000ff;
///    let mut tally = DefaultBordaTally::<u32>::new(1, Variant::Borda);
///    tally.add(vec![green, blue, red]);
///    tally.add(vec![red, green, blue]);
///    tally.add(vec![blue, green, red]);
///    tally.add(vec![blue, red, green]);
///    tally.add(vec![blue, red, green]);
///
///    let winners = tally.winners().into_unranked();
///
///    // Blue wins!
///    assert!(winners[0] == 0x00ff00);
/// ```
pub type DefaultBordaTally<T> = BordaTally<T, u64>;

/// A generic borda tally.
///
/// Generics:
/// - `T`: The candidate type.
/// - `C`: The count type. `u64` is recommended, but can be modified to use a different type for counting votes (eg `f64` for fractional vote weights). When using `Variant::Dowdall`, a float, a [`rational`] (https://rust-num.github.io/num/num_rational/index.html), or anyting that implements [`Real`](https://docs.rs/num-traits/0.2.6/num_traits/real/trait.Real.html) must be used.
///
/// Example:
/// ```
///    use tallyman::borda::BordaTally;
///    use tallyman::borda::Variant;
///
///    // A tally with string candidates, `f64` counting, and a single winner using the Dowall point system.
///    let mut tally = BordaTally::<&str, f64>::new(1, Variant::Dowdall);
///    tally.add(vec!["Alice", "Bob", "Carlos"]);
///    tally.add(vec!["Bob", "Carlos", "Alice"]);
///    tally.add(vec!["Alice", "Carlos", "Bob"]);
///    tally.add(vec!["Alice", "Bob", "Carlos"]);
///
///    let winners = tally.winners();
/// ```
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
  /// Create a new `BordaTally` with the given number of winners.
  ///
  /// If there is a tie, the number of winners might be more than `num_winners`.
  /// (See [`winners()`](#method.winners) for more information on ties.)
  pub fn new(num_winners: u32, variant: Variant) -> Self {
    return BordaTally {
      running_total: HashMap::new(),
      candidates: HashSet::new(),
      num_winners: num_winners,
      variant: variant,
    };
  }

  /// Create a new `BordaTally` with the given number of winners, and number of expected candidates.
  pub fn with_capacity(num_winners: u32, variant: Variant, expected_candidates: usize) -> Self {
    return BordaTally {
      running_total: HashMap::with_capacity(expected_candidates),
      candidates: HashSet::with_capacity(expected_candidates),
      num_winners: num_winners,
      variant: variant,
    };
  }

  /// Add a new vote
  ///
  /// Votes are represented as a vector of ranked candidates, ordered by preference.
  /// An error will only be returned if `vote` contains duplicate candidates.
  pub fn add(&mut self, vote: Vec<T>) -> Result<(), TallyError> {
    self.add_weighted(vote, C::one())
  }

  /// Add a new vote by reference
  pub fn add_ref(&mut self, vote: &[T]) -> Result<(), TallyError> {
    self.add_weighted_ref(vote, C::one())
  }

  /// Add a weighted vote.
  /// By default takes a weight as a `usize` integer, but can be customized by using `BordaTally` with a custom vote type.
  pub fn add_weighted(&mut self, vote: Vec<T>, weight: C) -> Result<(), TallyError> {
    check_duplicate(&vote)?;

    for candidate in vote.iter() {
      if !self.candidates.contains(candidate) {
        self.candidates.insert(candidate.clone());
      }
    }

    let entry = self.running_total.entry(vote);
    *entry.or_insert(C::zero()) += weight;

    Ok(())
  }

  /// Add a weighted vote by reference
  pub fn add_weighted_ref(&mut self, vote: &[T], weight: C) -> Result<(), TallyError> {
    check_duplicate(vote)?;

    for candidate in vote.iter() {
      if !self.candidates.contains(candidate) {
        self.candidates.insert(candidate.clone());
      }
    }

    let entry = self.running_total.entry(vote.to_vec());
    *entry.or_insert(C::zero()) += weight;

    Ok(())
  }

  /// Get a ranked list of winners. Winners with the same rank are tied.
  /// The number of winners might be greater than the requested `num_winners` if there is a tie.
  /// In a borda count, the winners are determine by what candidate obtains the most points.
  pub fn winners(&self) -> RankedWinners<T> {
    let mut counted = CountedCandidates::new();
    for (candidate, votecount) in self.totals().iter() {
      counted.push(candidate.clone(), *votecount);
    }
    return counted.into_ranked(self.num_winners);
  }

  /// Get a ranked list of all candidates. Candidates with the same rank are tied.
  pub fn ranked(&self) -> Vec<(T, u32)> {
    let mut counted = CountedCandidates::new();
    for (candidate, votecount) in self.totals().iter() {
      counted.push(candidate.clone(), *votecount);
    }
    return counted.into_ranked(0).into_vec();
  }

  /// Get point totals for this tally.
  ///
  /// This will return a vector with the number of borda points for each candidate.
  ///
  /// # Example
  /// ```
  ///    use tallyman::borda::DefaultBordaTally;
  ///    use tallyman::borda::Variant;
  ///
  ///    let mut tally = DefaultBordaTally::new(1, Variant::ClassicBorda);
  ///    for _ in 0..30 { tally.add(vec!["Alice", "Bob"]).unwrap() }
  ///    for _ in 0..10 { tally.add(vec!["Bob", "Alice"]).unwrap() }
  ///
  ///    for (candidate, num_points) in tally.totals().iter() {
  ///       println!("{} has {} points", candidate, num_points);
  ///    }
  ///    // Prints:
  ///    //   Alice has 70 points
  ///    //   Bob has 30 points
  /// ```
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

  /// Get a list of all candidates seen by this tally.
  /// Candidates are returned in no particular order.
  pub fn candidates(&self) -> Vec<T> {
    return self.candidates.iter().map(|x| x.clone()).collect();
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
  fn borda_test() -> Result<(), TallyError> {
    // From: https://en.wikipedia.org/wiki/Borda_count
    let mut borda_tally = DefaultBordaTally::new(1, Variant::Borda);
    borda_tally.add_weighted(vec!["Andrew", "Catherine", "Brian", "David"], 51)?;
    borda_tally.add_weighted(vec!["Catherine", "Brian", "David", "Andrew"], 5)?;
    borda_tally.add_weighted(vec!["Brian", "Catherine", "David", "Andrew"], 23)?;
    borda_tally.add_weighted(vec!["David", "Catherine", "Brian", "Andrew"], 21)?;

    assert!(borda_tally.totals() == vec![("Catherine", 205), ("Andrew", 153), ("Brian", 151), ("David", 91)]);
    assert!(borda_tally.winners().into_unranked() == vec!["Catherine"]);

    let mut classic_tally = DefaultBordaTally::new(1, Variant::ClassicBorda);
    classic_tally.add_weighted(vec!["Andrew", "Catherine", "Brian", "David"], 51)?;
    classic_tally.add_weighted(vec!["Catherine", "Brian", "David", "Andrew"], 5)?;
    classic_tally.add_weighted(vec!["Brian", "Catherine", "David", "Andrew"], 23)?;
    classic_tally.add_weighted(vec!["David", "Catherine", "Brian", "Andrew"], 21)?;

    assert!(classic_tally.totals() == vec![("Catherine", 305), ("Andrew", 253), ("Brian", 251), ("David", 191)]);
    assert!(classic_tally.winners().into_unranked() == vec!["Catherine"]);

    let mut dowdall_tally = BordaTally::<&str, f64>::new(1, Variant::Dowdall);
    dowdall_tally.add_weighted(vec!["Andrew", "Catherine", "Brian", "David"], 51.0)?;
    dowdall_tally.add_weighted(vec!["Catherine", "Brian", "David", "Andrew"], 5.0)?;
    dowdall_tally.add_weighted(vec!["Brian", "Catherine", "David", "Andrew"], 23.0)?;
    dowdall_tally.add_weighted(vec!["David", "Catherine", "Brian", "Andrew"], 21.0)?;

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
    tally.add_weighted(vec!["Memphis", "Nashville", "Chattanooga", "Knoxville"], 42)?;
    tally.add_weighted(vec!["Nashville", "Chattanooga", "Knoxville", "Memphis"], 26)?;
    tally.add_weighted(vec!["Chattanooga", "Knoxville", "Nashville", "Memphis"], 15)?;
    tally.add_weighted(vec!["Knoxville", "Chattanooga", "Nashville", "Memphis"], 17)?;
    assert!(tally.totals() == vec![("Nashville", 194), ("Chattanooga", 173), ("Memphis", 126), ("Knoxville", 107)]);
    assert!(tally.winners().into_unranked() == vec!["Nashville"]);

    // Testing Modified Borda
    let mut tally = DefaultBordaTally::new(1, Variant::ModifiedBorda);
    tally.add(vec!["Alice", "Bob", "Carlos"])?;
    tally.add(vec!["Alice", "Bob"])?;
    tally.add(vec!["Bob", "Carlos"])?;
    assert!(tally.totals() == vec![("Alice", 3), ("Bob", 2), ("Carlos", 0)]);

    let mut tally = DefaultBordaTally::new(1, Variant::ModifiedClassicBorda);
    tally.add(vec!["Alice", "Bob", "Carlos"])?;
    tally.add(vec!["Alice", "Bob"])?;
    tally.add(vec!["Bob", "Carlos"])?;
    assert!(tally.totals() == vec![("Alice", 5), ("Bob", 5), ("Carlos", 2)]);

    // Testin adding ref
    let vote_1 = vec!["Alice", "Bob", "Carlos"];
    let vote_2 = vec!["Alice", "Bob"];
    let mut tally = DefaultBordaTally::new(1, Variant::Borda);
    tally.add_ref(&vote_1)?;
    tally.add_ref(&vote_2)?;
    assert!(tally.totals() == vec![("Alice", 4), ("Bob", 2), ("Carlos", 0)]);

    Ok(())
  }

  #[test]
  #[should_panic]
  fn borda_panic_test() {
    // Dowdall should panic when using integers
    let _points: u64 = Variant::Dowdall.points(0, 4, 4);
  }
}
