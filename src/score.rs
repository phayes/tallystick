use std::hash::Hash;
use num_traits::cast::NumCast;
use num_traits::Num;
use std::ops::AddAssign;

use super::plurality;


pub type DefaultTally<T> = Tally<T, u64>;

pub struct Tally<T, C = u64>
    where T: Eq + Clone + Hash,        // Candidate
          C: Copy + Ord + AddAssign + Num + NumCast // vote count type
{
    plurality: plurality::Tally<T, C>
}

impl<T, C> Tally<T, C>
    where T: Eq + Clone + Hash,        // Candidate
          C: Copy + Ord + AddAssign + Num + NumCast // vote count type
{
    pub fn new(num_winners: u32) -> Self {
        return Tally {
            plurality: plurality::Tally::new(num_winners)
        };
    }

    pub fn add(&mut self, mut selection: Vec<(T, C)>) {
        for (vote, score) in selection.drain(0..) {
            self.plurality.add_weighted(vote, score);
        }
    }

    pub fn add_ref(&mut self, selection: &[(T, C)]) {
        for (vote, score) in selection {
            self.plurality.add_weighted_ref(vote, *score);
        }
    }

    pub fn add_weighted(&mut self, mut selection: Vec<(T, C)>, weight: C) {
        for (vote, score) in selection.drain(0..) {
            self.plurality.add_weighted(vote, weight * score);
        }
    }

    pub fn add_weighted_ref(&mut self, selection: &[(T, C)], weight: C) {
        for (vote, score) in selection {
            self.plurality.add_weighted_ref(vote, weight * *score);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_test() {

    }
}
