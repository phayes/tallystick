use num_traits::Num;
use num_traits::Float;

/// A quota defines how many votes are required to win an election in relation to the total number of votes cast.
pub enum Quota {

    /// Droop quota. It is defined as:
    /// 
    /// ```floor((total-votes / (total-seats + 1)) + 1```
    /// 
    /// In single-winner elections, it's often known as "fifty percent plus one". 
    /// The Droop quota is always an integer.
    /// 
    /// See [wikipedia](https://en.wikipedia.org/wiki/Droop_quota) for more details.
    Droop,

    /// Hagenbach-Bischoff quota.
    /// 
    /// Also known as the "Newland-Britton quota" or the "exact Droop quota", it is defined as:
    /// 
    /// ```total-votes / (total-seats + 1)```
    /// 
    /// It differs from the Droop quota in that the quota often contains a fraction. In single-winner elections, 
    /// the first candidate to achieve more than 50% of the vote wins. This system is best used when fractional 
    /// votes are being used, or in a transferable-vote system where votes are redistributed fractionally.
    /// 
    /// See [wikipedia](https://en.wikipedia.org/wiki/Hagenbach-Bischoff_quota) for more details.
    Hagenbach,
  
    /// Hare quota.
    /// 
    /// It is defined as:
    /// 
    /// ```total-votes / total-seats```
    /// 
    /// In single-winner elections, it is equal to fifty percent of the vote. 
    /// It is generally not recommended and is included for completeness.
    /// 
    /// See [wikipedia](https://en.wikipedia.org/wiki/Hare_quota) for more details.
    Hare
}

impl Quota {
  /// Compute the threshold needed to be elected for the given quota.
  /// 
  /// Note that total-votes should be the number of votes counted in the tally.
  /// It should not include invalid votes that were not added the tally.
  /// For weighted tallies, it should be the sum of all weights.
  /// 
  /// *This method will panic if `Quota::Hagenbach` is used with an integer count type.*
  pub fn threshold<C: Num + Floorable>(&self, total_votes: C, num_winners: C) -> C {
    match self {
      Quota::Droop => (total_votes / (num_winners + C::one())).floor() + C::one(),
      Quota::Hagenbach => {
        if !C::is_float() {
          panic!("tallyman::Quota::Hagenbach cannot be used with an integer count type. Please use a float or a rational.")
        }
        total_votes / (num_winners + C::one())
      },
      Quota::Hare => total_votes / num_winners,
    }
  }
}

pub trait Floorable {
    fn floor(self) -> Self;
    fn is_float() -> bool;
}

impl<T> Floorable for T {
    default fn floor(self) -> Self {
        self
    }
    default fn is_float() -> bool {
        false
    }
}

impl<T> Floorable for T
where
    T: Float
{
    fn floor(self) -> Self {
        self.floor()
    }
    fn is_float() -> bool {
        true
    }
}