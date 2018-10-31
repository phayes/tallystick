extern crate indexmap;

use std::cmp::Ordering;
use std::cmp::Ord;
use std::hash::Hash;

pub use indexmap::IndexMap;

pub struct PluralityTally<T: Eq + Clone + Hash> {
    running_total: IndexMap<T, u32>,
}

impl<T: Eq + Clone + Hash> PluralityTally<T>  {

    pub fn new() -> Self {
        return PluralityTally {
            running_total: IndexMap::new()
        };
    }

    pub fn add(&mut self, selection: T) {
        if self.running_total.contains_key(&selection) {
            if let Some(x) = self.running_total.get_mut(&selection) {
                *x += 1;
            }
        }
        else {
            self.running_total.insert(selection.clone(), 1);
        }
    }

    pub fn result(&self) -> IndexMap<T, u32> {
        let mut result = self.running_total.clone();
        result.sort_by(sort_values_desc);
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
        let mut tally = PluralityTally::new();
        tally.add("Alice");
        tally.add("Cir");
        tally.add("Bob");
        tally.add("Alice");
        tally.add("Alice");
        tally.add("Bob");

        let result = tally.result();

        let (winner, winner_votes) = result.get_index(0).unwrap();
        let (runner_up, runner_up_votes) = result.get_index(1).unwrap();

        assert_eq!("Alice", *winner);
        assert_eq!(3, *winner_votes);
        assert_eq!("Bob", *runner_up);
        assert_eq!(2, *runner_up_votes);

        // Election for the most popular integer
        let mut tally = PluralityTally::new();
        tally.add(99);
        tally.add(100);
        tally.add(99);
        tally.add(99);
        tally.add(1);
        tally.add(1);
        tally.add(2);
        tally.add(0);

        let result = tally.result();

        let (winner, winner_votes) = result.get_index(0).unwrap();
        let (runner_up, runner_up_votes) = result.get_index(1).unwrap();

        assert_eq!(99, *winner);
        assert_eq!(3, *winner_votes);
        assert_eq!(1, *runner_up);
        assert_eq!(2, *runner_up_votes);
    }
}
