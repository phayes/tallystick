use std::cmp::Ord;
use std::hash::Hash;
use hashbrown::HashMap;
use num_traits::cast::NumCast;
use num_traits::Num;
use std::ops::AddAssign;

use super::result::RankedWinners;
use super::result::CountedCandidates;

/// A simple plurality (first-past-the-post) tally
/// # Examples
///
/// ```
///    use tallyman::plurality::Tally;
///
///    // Election between Alice, Bob, and Cir
///    let mut tally = Tally::new(2);
///    tally.add("Alice");
///    tally.add("Cir");
///    tally.add("Bob");
///    tally.add("Alice");
///    tally.add("Alice");
///    tally.add("Bob");
/// 
///    let winners = tally.winners().into_unranked();
///    println!("The winners are {:?}", winners);
/// ```
pub type Tally<T> = CustomTally<T, usize>;


/// Use CustomTally when you want to customize the type used to count the votes.
/// By default votes are counted in `usize`, but you may want to use `f64` to allow fractionally weighted votes etc.
pub struct CustomTally<T, C = usize>
    where T: Eq + Clone + Hash, // Candidate
          C: Copy + Ord + AddAssign + Num + NumCast // Count type
{
    running_total: HashMap<T, C>,
    num_winners: u32
}


impl<T, C> CustomTally<T, C>
    where T: Eq + Clone + Hash, // Candidate
          C: Copy + Ord + AddAssign + Num + NumCast // Count type
{

    pub fn new(num_winners: u32) -> Self {
        return CustomTally {
            running_total: HashMap::new(),
            num_winners: num_winners
        };
    }

    pub fn add(&mut self, selection: T) {
        self.add_weighted(selection, C::one());
    }

    pub fn add_ref(&mut self, selection: &T) {
        self.add_weighted_ref(selection, C::one());
    }

    pub fn add_weighted(&mut self, selection: T, weight: C) {
        *self.running_total.entry(selection).or_insert(C::zero()) += weight;
    }

    pub fn add_weighted_ref(&mut self, selection: &T, weight: C) {
        if self.running_total.contains_key(&selection) {
            if let Some(x) = self.running_total.get_mut(&selection) {
                *x += weight;
            }
        }
        else {
            self.running_total.insert(selection.clone(), weight);
        }
    }

    pub fn winners(&self) -> RankedWinners<T> {
        let mut counted = CountedCandidates::new();
        for (candidate, votecount) in self.running_total.iter() {
            counted.push(candidate.clone(), *votecount);
        }
        return counted.into_ranked(self.num_winners);
    }
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

        let winners = tally.winners();

        assert_eq!(winners.contains(&"Alice"), true);
        assert_eq!(winners.contains(&"Bob"), true);
        assert_eq!(winners.contains(&"Cir"), false);
        assert_eq!(winners.contains(&"Rando"), false);

        // Election for the most popular integer
        let mut tally = Tally::new(1);
        tally.add(99);
        tally.add(100);
        tally.add(99);
        tally.add(99);
        tally.add(1);
        tally.add(1);
        tally.add(2);
        tally.add(0);

        let winners = tally.winners();

        assert_eq!(winners.contains(&99), true);
        assert_eq!(winners.contains(&100), false);
        assert_eq!(winners.contains(&1), false);
        assert_eq!(winners.contains(&2), false);
        assert_eq!(winners.contains(&1000), false);
    }
}
