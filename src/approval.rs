use std::hash::Hash;
use num_traits::cast::NumCast;
use num_traits::Num;
use std::ops::AddAssign;

use super::plurality;


pub type DefaultTally<T> = Tally<T, usize>;

pub struct Tally<T, C = usize>
    where T: Eq + Clone + Hash,        // Candidate
          C: Copy + Ord + AddAssign + Num + NumCast // Vote count type
{
    plurality: plurality::Tally<T, C>
}

impl<T, C> Tally<T, C>
    where T: Eq + Clone + Hash,        // Candidate
          C: Copy + Ord + AddAssign + Num + NumCast // Vote count type
{
    pub fn new(num_winners: u32) -> Self {
        return Tally {
            plurality: plurality::Tally::new(num_winners)
        };
    }

    pub fn add(&mut self, mut selection: Vec<T>) {
        for vote in selection.drain(0..) {
            self.plurality.add(vote);
        }
    }

    pub fn add_ref(&mut self, selection: &Vec<T>) {
        for vote in selection {
            self.plurality.add_ref(vote);
        }
    }

    pub fn add_weighted(&mut self, mut selection: Vec<T>, weight: C) {
        for vote in selection.drain(0..) {
            self.plurality.add_weighted(vote, weight);
        }
    }

    pub fn add_weighted_ref(&mut self, selection: &Vec<T>, weight: C) {
        for vote in selection {
            self.plurality.add_weighted_ref(vote, weight);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approval_test() {

    }
}
