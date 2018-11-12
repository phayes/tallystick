extern crate indexmap;

use std::cmp::Ordering;
use std::cmp::Ord;
use std::hash::Hash;
use hashbrown::HashMap;
use num_traits::cast::NumCast;
use num_traits::Num;
use std::ops::AddAssign;
use indexmap::IndexMap;

pub struct WeightedTally<T, W = usize>
    where T: Eq + Clone + Hash, // Candidate
          W: AddAssign + Num + NumCast // Weight
{
    running_total: HashMap<T, W>,
    num_winners: u32
}

pub type Tally<T> = WeightedTally<T, usize>;

impl<T, W> WeightedTally<T, W>
    where T: Eq + Clone + Hash, // Candidate
          W: AddAssign + Num + NumCast // Weight
{

    pub fn new(num_winners: u32) -> Self {
        return WeightedTally {
            running_total: HashMap::new(),
            num_winners: num_winners
        };
    }

    pub fn add(&mut self, selection: T) {
        self.add_weighted(selection, W::one());
    }

    pub fn add_ref(&mut self, selection: &T) {
        self.add_weighted_ref(selection, W::one());
    }

    pub fn add_weighted(&mut self, selection: T, weight: W) {
        *self.running_total.entry(selection).or_insert(W::zero()) += weight;
    }

    pub fn add_weighted_ref(&mut self, selection: &T, weight: W) {
        if self.running_total.contains_key(&selection) {
            if let Some(x) = self.running_total.get_mut(&selection) {
                *x += weight;
            }
        }
        else {
            self.running_total.insert(selection.clone(), weight);
        }
    }

    pub fn result(&self) -> IndexMap<T, usize> {
        let mut result: IndexMap<T, usize> = IndexMap::new();

        for (candidate, votecount) in self.running_total.iter() {
            result.insert(candidate.clone(), votecount.to_usize().unwrap());
        }

        // Sort the results
        result.sort_by(sort_values_desc);

        // Replace votecounts with ranks
        let mut rank = 0;
        let mut previous_votecount: usize = 0;
        for (_, result_value) in result.iter_mut() {
            let votecount: usize = *result_value;
            *result_value = rank;
            if votecount != previous_votecount {
                rank += 1;
            }
            previous_votecount = votecount;
        }

        // Remove unelected
        // TODO: There must be a better way to do this (without clone)
        let mut to_remove: Vec<T> = Vec::new();
        let mut previous_rank: usize = 0;
        let mut elected = 0;
        for (candidate, rank) in result.iter() {
            if elected >= self.num_winners && *rank != previous_rank {
                to_remove.push(candidate.clone());
            }
            elected += 1;
            previous_rank = *rank;
        }
        for candidate in to_remove.iter() {
            result.remove(candidate);
        }

        return result;
    }
}

fn sort_values_desc<K, V: Ord>(_k1: &K, v1: &V, _k2: &K, v2: &V) -> Ordering {
    return v2.cmp(v1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plurality_test() {

        // Election between Alice, Bob, and Cir
        let mut tally = Tally::new(2);
        tally.add("Alice");
        tally.add("Cir");
        tally.add("Bob");
        tally.add("Alice");
        tally.add("Alice");
        tally.add("Bob");

        let result = tally.result();

        let (winner, winner_rank) = result.get_index(0).unwrap();
        let (runner_up, runner_up_rank) = result.get_index(1).unwrap();

        assert_eq!("Alice", *winner);
        assert_eq!(0, *winner_rank);
        assert_eq!("Bob", *runner_up);
        assert_eq!(1, *runner_up_rank);

        // Election for the most popular integer
        let mut tally = Tally::new(2);
        tally.add(99);
        tally.add(100);
        tally.add(99);
        tally.add(99);
        tally.add(1);
        tally.add(1);
        tally.add(2);
        tally.add(0);

        let result = tally.result();

        let (winner, winner_rank) = result.get_index(0).unwrap();
        let (runner_up, runner_up_rank) = result.get_index(1).unwrap();

        assert_eq!(99, *winner);
        assert_eq!(0, *winner_rank);
        assert_eq!(1, *runner_up);
        assert_eq!(1, *runner_up_rank);
    }
}
