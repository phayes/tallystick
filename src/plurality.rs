use hashbrown::HashMap;
use num_traits::cast::NumCast;
use num_traits::Num;
use std::hash::Hash;
use std::ops::AddAssign;

use super::result::CountedCandidates;
use super::result::RankedWinners;

/// A plurality tally using `u64` integers to count votes.
/// `DefaultPluralityTally` is generally preferred over `PluralityTally`.
/// Since this is an alias, refer to [`PluralityTally`](struct.PluralityTally.html) for method documentation.
///
/// # Example
/// ```
///    use tallystick::plurality::DefaultPluralityTally;
///
///    // What is the loniest number?
///    // A vote with integer candidates and a single-winner.
///    let mut tally = DefaultPluralityTally::<u32>::new(1);
///    tally.add(5);
///    tally.add(0);
///    tally.add(1);
///    tally.add(1);
///    tally.add(2);
///    tally.add(3);
///
///    let winners = tally.winners().into_unranked();
///    assert!(winners[0] == 1);
/// ```
pub type DefaultPluralityTally<T> = PluralityTally<T, u64>;

/// A generic plurality tally.
///
/// Generics:
/// - `T`: The candidate type.
/// - `C`: The count type. `u64` is recommended, but can be modified to use a different type for counting votes (eg `f64` for fractional vote weights).
///
/// Example:
/// ```
///    use tallystick::plurality::PluralityTally;
///
///    // A tally with string candidates, `f64` counting, and a single winner.
///    // f64 counting lets us use fractional vote weights.
///    let mut tally = PluralityTally::<&str, f64>::new(1);
///    tally.add_weighted("Alice", 5.25); // A vote for Alice with a weight of `5.25`
///    tally.add_weighted("Bob", 0.25);   // A vote for Bob with a weight of `0.25`
///    tally.add("Carol");                // A vote for Carol with an implicit weight of `1.0`
///    let winners = tally.winners();
/// ```
pub struct PluralityTally<T, C = u64>
where
    T: Eq + Clone + Hash,                             // Candidate type
    C: Copy + PartialOrd + AddAssign + Num + NumCast, // Count type
{
    running_total: HashMap<T, C>,
    num_winners: usize,
}

impl<T, C> PluralityTally<T, C>
where
    T: Eq + Clone + Hash,                             // Candidate type
    C: Copy + PartialOrd + AddAssign + Num + NumCast, // Count type
{
    /// Create a new `PluralityTally` with the given number of winners.
    ///
    /// If there is a tie, the number of winners might be more than `num_winners`.
    /// (See [`winners()`](#method.winners) for more information on ties.)
    pub fn new(num_winners: usize) -> Self {
        PluralityTally {
            running_total: HashMap::new(),
            num_winners: num_winners,
        }
    }

    /// Create a new `PluralityTally` with the given number of winners, and number of expected candidates.
    pub fn with_capacity(num_winners: usize, expected_candidates: usize) -> Self {
        PluralityTally {
            running_total: HashMap::with_capacity(expected_candidates),
            num_winners: num_winners,
        }
    }

    /// Add a new vote
    pub fn add(&mut self, vote: T) {
        self.add_weighted(vote, C::one());
    }

    /// Add a vote by reference.
    pub fn add_ref(&mut self, vote: &T) {
        self.add_weighted_ref(vote, C::one());
    }

    /// Add a weighted vote.
    /// By default takes a weight as a `usize` integer, but can be customized by using `PluralityTally` with a custom vote type.
    pub fn add_weighted(&mut self, vote: T, weight: C) {
        *self.running_total.entry(vote).or_insert(C::zero()) += weight;
    }

    /// Add a weighted vote by reference.
    pub fn add_weighted_ref(&mut self, vote: &T, weight: C) {
        if self.running_total.contains_key(&vote) {
            if let Some(x) = self.running_total.get_mut(&vote) {
                *x += weight;
            }
        } else {
            self.running_total.insert(vote.clone(), weight);
        }
    }

    /// Get a list of all candidates seen by this tally.
    /// Candidates are returned in no particular order.
    pub fn candidates(&self) -> Vec<T> {
        self.running_total.keys().cloned().collect()
    }

    /// Get a ranked list of winners. Winners with the same rank are tied.
    /// The number of winners might be greater than the requested `num_winners` if there is a tie.
    ///
    /// # Example
    /// ```
    ///    use tallystick::plurality::DefaultPluralityTally;
    ///
    ///    let mut tally = DefaultPluralityTally::new(2); // We ideally want only 2 winnners
    ///    tally.add_weighted("Alice", 3);
    ///    tally.add_weighted("Cir", 2);
    ///    tally.add_weighted("Bob", 2);
    ///    tally.add("Dave"); // implicit weight of 1
    ///
    ///    let winners = tally.winners();
    ///
    ///    println!("We have {} winners", winners.len());
    ///    // Prints: "We have 3 winners" (due to Cir and Bob being tied)
    ///
    ///    for (winner, rank) in winners.iter() {
    ///       println!("{} has a rank of {}", winner, rank);
    ///    }
    ///    // Prints:
    ///    //   Alice has a rank of 0
    ///    //   Bob has a rank of 1
    ///    //   Cir has a rank of 1
    /// ```
    pub fn winners(&self) -> RankedWinners<T> {
        self.get_counted().into_ranked(self.num_winners)
    }

    /// Get vote totals for this tally.
    ///
    /// # Example
    /// ```
    ///    use tallystick::plurality::DefaultPluralityTally;
    ///
    ///    let mut tally = DefaultPluralityTally::new(1);
    ///    for _ in 0..30 { tally.add("Alice") }
    ///    for _ in 0..10 { tally.add("Bob") }
    ///
    ///    for (candidate, num_votes) in tally.totals().iter() {
    ///       println!("{} got {} votes", candidate, num_votes);
    ///    }
    ///    // Prints:
    ///    //   Alice got 30 votes
    ///    //   Bob got 10 votes
    /// ```
    pub fn totals(&self) -> Vec<(T, C)> {
        self.get_counted().into_vec()
    }

    /// Get a ranked list of all candidates. Candidates with the same rank are tied.
    /// Candidates are ranked in ascending order. The highest ranked candidate has a rank of `0`.
    ///
    /// # Example
    /// ```
    ///    use tallystick::plurality::DefaultPluralityTally;
    ///
    ///    let mut tally = DefaultPluralityTally::new(1);
    ///    for _ in 0..50 { tally.add("Alice") }
    ///    for _ in 0..40 { tally.add("Bob") }
    ///    for _ in 0..30 { tally.add("Carlos") }
    ///    
    ///    for (candidate, rank) in tally.ranked().iter() {
    ///       println!("{} has a rank of {}", candidate, rank);
    ///    }
    ///    // Prints:
    ///    //   Alice has a rank of 0
    ///    //   Bob has a rank of 1
    ///    //   Carlos has a rank of 2
    /// ```
    pub fn ranked(&self) -> Vec<(T, usize)> {
        self.get_counted().into_ranked(0).into_vec()
    }

    // Get the running total as CountedCandidates.
    pub(crate) fn get_counted(&self) -> CountedCandidates<T, C> {
        let mut counted = CountedCandidates::new();
        for (candidate, votecount) in self.running_total.iter() {
            counted.push(candidate.clone(), *votecount);
        }

        counted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plurality_test() {
        // Election between Alice, Bob, and Cir
        let mut tally = PluralityTally::new(2);
        tally.add("Alice");
        tally.add("Cir");
        tally.add("Bob");
        tally.add("Alice");
        tally.add("Alice");
        tally.add("Bob");

        assert_eq!(tally.candidates().len(), 3);
        assert_eq!(tally.totals(), vec![("Alice", 3), ("Bob", 2), ("Cir", 1)]);
        assert_eq!(tally.ranked(), vec![("Alice", 0), ("Bob", 1), ("Cir", 2)]);

        let winners = tally.winners();
        assert_eq!(winners.contains(&"Alice"), true);
        assert_eq!(winners.contains(&"Bob"), true);
        assert_eq!(winners.contains(&"Cir"), false);
        assert_eq!(winners.contains(&"Rando"), false);

        // Election for the most popular integer
        let mut tally = DefaultPluralityTally::new(1);
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

        // Create an election with capacity
        let mut tally = DefaultPluralityTally::with_capacity(1, 2);
        let candidate_id_1 = 123;
        let candidate_id_2 = 456;
        tally.add_ref(&candidate_id_1);
        tally.add_ref(&candidate_id_2);

        let winners = tally.winners();
        assert_eq!(winners.contains(&candidate_id_1), true);
        assert_eq!(winners.contains(&candidate_id_2), true);
    }
}
