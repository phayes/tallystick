use hashbrown::HashMap;
use num_traits::{Num, NumCast};
use petgraph::Graph;

use super::condorcet::CondorcetTally;
use super::errors::TallyError;
use super::plurality::PluralityTally;
use super::result::CountedCandidates;
use super::result::RankedWinners;
use super::Numeric;
use std::hash::Hash;
use std::ops::AddAssign;

/// Schulze Variants.
///
/// Each variant represents a different way to measure the strength of a link.
pub enum Variant {
    /// Strength of a link is measured by its support. You should use this variant if you are unsure.
    ///
    /// When the strength of the link `ef` is measured by winning votes, then its strength is measured primarily by its support `N[e,f]`.
    Winning,

    /// The strength of a link is measured by the difference between its support and opposition.
    ///
    /// When the strength of the link `ef` is measured by margin, then its strength is the difference `N[e,f] – N[f,e]` between its support `N[e,f]` and its opposition `N[f,e]`.
    Margin,

    /// The strength of a link is measured by the ratio of its support and opposition.
    ///
    /// When the strength of the link `ef` is measured by ratio, then its strength is the ratio `N[e,f] / N[f,e]` between its support `N[e,f]` and its opposition `N[f,e]`.
    ///
    /// If Ratio is selected, tallystick will panic if an integer count type is used in the tally. This variant should only be used with a float tally.
    Ratio,
}

/// A schulze tally using `u64` integers to count votes.
/// `DefaultSchulzeTally` is generally preferred over `SchulzeTally`.
/// Since this is an alias, refer to [`Schulze`](struct.Schulze.html) for method documentation.
///
/// # Example
/// ```
///    use tallystick::schulze::DefaultSchulzeTally;
///    use tallystick::schulze::Variant;
///
///    // An election for Judge
///
///    // TODO: "Abe Vigoda" not implicitly added at the end....
///    let mut tally = DefaultSchulzeTally::with_candidates(1, Variant::Winning, vec!["Notorious RBG", "Judge Judy", "Abe Vigoda", "Judge Dredd"]);
///    tally.add(&vec!["Notorious RBG", "Judge Judy"]);
///    tally.add(&vec!["Judge Dredd"]);
///    tally.add(&vec!["Abe Vigoda", "Notorious RBG"]);
///    tally.add(&vec!["Notorious RBG", "Judge Dredd"]);
///
///    let winners = tally.winners().into_unranked();
///    assert!(winners[0] == "Notorious RBG");
/// ```
pub type DefaultSchulzeTally<T> = SchulzeTally<T, u64>;

/// A generic schulze tally.
///
/// Generics:schulze
/// - `T`: The candidate type.
/// - `C`: The count type. `u64` is recommended, but can be modified to use a different type for counting votes (eg `f64` for fractional vote weights).
///
/// # Example
/// ```
///    use tallystick::schulze::SchulzeTally;
///    use tallystick::schulze::Variant;
///
///    // An election for Judge using floats as the count type.
///    let mut tally = SchulzeTally::<&str, f64>::new(1, Variant::Ratio);
///    tally.add_candidates(vec!["Notorious RBG", "Judge Judy", "Abe Vigoda", "Judge Dredd"]);
///    tally.add_weighted(&vec!["Notorious RBG", "Judge Judy", "Judge Dredd", "Abe Vigoda"], 0.5);
///    tally.add_weighted(&vec!["Judge Dredd", "Abe Vigoda", "Notorious RBG", "Judge Judy"], 2.0);
///    tally.add_weighted(&vec!["Abe Vigoda", "Notorious RBG", "Judge Judy", "Judge Dredd"], 3.2);
///    tally.add_weighted(&vec!["Notorious RBG", "Judge Dredd", "Judge Judy", "Abe Vigoda"], 4.0);
///    tally.add_weighted(&vec!["Judge Judy", "Notorious RBG", "Judge Dredd", "Abe Vigoda"], 0.2);
///
///    let winners = tally.winners().into_unranked();
///    assert!(winners[0] == "Notorious RBG");
/// ```
pub struct SchulzeTally<T, C = u64>
where
    T: Eq + Clone + Hash,                             // Candidate
    C: Copy + PartialOrd + AddAssign + Num + NumCast, // Vote count type
{
    variant: Variant,
    condorcet: CondorcetTally<T, C>,
}

impl<T, C> SchulzeTally<T, C>
where
    T: Eq + Clone + Hash,                             // Candidate
    C: Copy + PartialOrd + AddAssign + Num + NumCast, // Vote count type
{
    /// Create a new `SchulzeTally` with the given number of winners.
    ///
    /// If there is a tie, the number of winners might be more than `num_winners`.
    /// (See [`winners()`](#method.winners) for more information on ties.)
    ///
    /// This may panic if `Variant::Ratio` is used with an integer count type. (A float count type should be used instead).
    pub fn new(num_winners: usize, variant: Variant) -> Self {
        Self::check_types(&variant);
        SchulzeTally {
            variant: variant,
            condorcet: CondorcetTally::new(num_winners),
        }
    }

    /// Create a new `SchulzeTally` with the given number of winners, and number of expected candidates.
    ///
    /// This may panic if `Variant::Ratio` is used with an integer count type. (A float count type should be used instead).
    pub fn with_candidates(num_winners: usize, variant: Variant, candidates: Vec<T>) -> Self {
        Self::check_types(&variant);
        SchulzeTally {
            variant: variant,
            condorcet: CondorcetTally::with_candidates(num_winners, candidates),
        }
    }

    /// Make this tally an unchecked tally, forgoing vote validity checking
    ///
    /// When using an unchecked tally, all vote adding methods will return Ok(), so you may elide checking for errors.
    pub fn unchecked(mut self) -> Self {
        self.condorcet = self.condorcet.unchecked();
        self
    }

    /// Add a candidate to the tally.
    pub fn add_candidate(&mut self, candidate: T) {
        self.condorcet.add_candidate(candidate);
    }

    /// Add some candidates to the tally.
    pub fn add_candidates(&mut self, candidates: Vec<T>) {
        self.condorcet.add_candidates(candidates);
    }

    /// Add a vote.
    pub fn add(&mut self, selection: &[T]) -> Result<(), TallyError> {
        self.condorcet.add(selection)
    }

    /// Add a weighted vote.
    ///
    /// By default takes a weight as a `usize` integer, but can be customized by using `SchulzeTally` with a custom count type.
    pub fn add_weighted(&mut self, selection: &[T], weight: C) -> Result<(), TallyError> {
        self.condorcet.add_weighted(selection, weight)
    }

    /// Add a new ranked vote
    ///
    /// A ranked vote is a list of tuples of (candidate, rank), where rank is ascending.
    /// Two candidates with the same rank are equal in preference.
    pub fn ranked_add(&mut self, vote: &[(T, u32)]) -> Result<(), TallyError> {
        self.condorcet.ranked_add(vote)
    }

    /// Add a ranked weighted vote.
    /// By default takes a weight as a `usize` integer, but can be customized by using `CondorcetTally` with a custom count type.
    ///
    /// A ranked vote is a list of tuples of (candidate, rank), where rank is ascending.
    /// Two candidates with the same rank are equal in preference.
    pub fn ranked_add_weighted(&mut self, vote: &[(T, u32)], weight: C) -> Result<(), TallyError> {
        self.condorcet.ranked_add_weighted(vote, weight)
    }

    /// Get a list of all candidates seen by this tally.
    /// Candidates are returned in no particular order.
    pub fn candidates(&self) -> Vec<T> {
        self.condorcet.candidates()
    }

    /// Get total counts for this tally.
    /// Totals are returned as a list of pairwise comparisons
    /// For a pairwise comparison `((T1, T2), C)`, `C` is the number of votes where candidate `T1` is preferred over candidate `T2`.
    ///
    /// # Example
    /// ```
    ///    use tallystick::schulze::DefaultSchulzeTally;
    ///    use tallystick::schulze::Variant;
    ///
    ///    let mut tally = DefaultSchulzeTally::with_candidates(1, Variant::Winning, vec!["Alice", "Bob"]);
    ///    for _ in 0..30 { tally.add(&vec!["Alice", "Bob"]); }
    ///    for _ in 0..10 { tally.add(&vec!["Bob", "Alice"]); }
    ///
    ///    for ((candidate1, candidate2), num_votes) in tally.totals().iter() {
    ///       println!("{} is preferred over {} {} times", candidate1, candidate2, num_votes);
    ///    }
    ///    // Prints:
    ///    //   Alice is preferred over Bob 30 times
    ///    //   Bob is preferred over Alice 10 times
    /// ```
    pub fn totals(&self) -> Vec<((T, T), C)> {
        self.condorcet.totals()
    }

    /// Computes the strongest path between all candidates.
    ///
    /// This is a well-known problem in graph theory sometimes called the widest path problem.
    /// This function uses a variant of the Floyd–Warshall algorithm.
    ///
    /// See: [https://en.wikipedia.org/wiki/Schulze_method#Implementations](https://en.wikipedia.org/wiki/Schulze_method#Implementations)
    pub fn strongest_paths(&self) -> Vec<((T, T), C)> {
        let zero = C::zero();
        let mut p = HashMap::<(usize, usize), C>::new();
        for i in self.condorcet.candidates.values() {
            for j in self.condorcet.candidates.values() {
                if i != j {
                    let dij = self.condorcet.running_total.get(&(*i, *j)).unwrap_or(&zero);
                    let dji = self.condorcet.running_total.get(&(*j, *i)).unwrap_or(&zero);

                    if dij > dji {
                        let strength = match self.variant {
                            Variant::Winning => *dij,
                            Variant::Margin => *dij - *dji,
                            Variant::Ratio => {
                                if dji != &zero {
                                    *dij / *dji
                                } else {
                                    C::max_value()
                                }
                            }
                        };
                        p.insert((*i, *j), strength);
                    } else {
                        p.insert((*i, *j), zero);
                    }
                }
            }
        }

        for i in self.condorcet.candidates.values() {
            for j in self.condorcet.candidates.values() {
                if i != j {
                    for k in self.condorcet.candidates.values() {
                        if i != k && j != k {
                            //p[j,k] := max ( p[j,k], min ( p[j,i], p[i,k] ) )
                            let pji = p.get(&(*j, *i)).unwrap_or(&zero);
                            let pik = p.get(&(*i, *k)).unwrap_or(&zero);
                            let pjk = p.get(&(*j, *k)).unwrap_or(&zero);

                            let min = if pji < pik { pji } else { pik };
                            let max = if pjk > min { *pjk } else { *min };
                            p.insert((*j, *k), max);
                        }
                    }
                }
            }
        }

        let mut strongest = Vec::<((T, T), C)>::with_capacity(self.condorcet.running_total.len());

        // Invert the candidate map.
        let mut candidates = HashMap::<usize, T>::with_capacity(self.condorcet.candidates.len());
        for (candidate, i) in self.condorcet.candidates.iter() {
            candidates.insert(*i, candidate.clone());
        }

        for ((candidate1, candidate2), strength) in p.iter() {
            // Ok to unwrap here since candidates must exist.
            let candidate1 = candidates.get(candidate1).unwrap().clone();
            let candidate2 = candidates.get(candidate2).unwrap().clone();
            strongest.push(((candidate1, candidate2), *strength));
        }

        strongest
    }

    pub(crate) fn get_counted(&self) -> CountedCandidates<T, C> {
        let mut strongest = self.strongest_paths();

        // Convert strongest to a hashmap
        let mut strongest_hash = HashMap::<(T, T), C>::with_capacity(strongest.len());
        for ((candidate_1, candidate_2), strength) in strongest.drain(..) {
            strongest_hash.insert((candidate_1, candidate_2), strength);
        }

        // Make a little plurality tally for counting up pairwise strength competition.
        let mut running_total = PluralityTally::with_capacity(self.condorcet.num_winners, self.condorcet.candidates.len());

        let zero = C::zero();
        for ((candidate_1, candidate_2), strength_1) in strongest_hash.iter() {
            // Cloning here is dumb, but unable to construct a key tuple otherwise
            let strength_2 = strongest_hash.get(&(candidate_2.clone(), candidate_1.clone())).unwrap_or(&zero);
            if strength_1 >= strength_2 {
                running_total.add_ref(candidate_1);
            } else {
                // Add it with a weight of zero
                running_total.add_weighted_ref(candidate_1, C::zero());
            }
        }

        running_total.get_counted()
    }

    /// Get a ranked list of all candidates. Candidates with the same rank are tied.
    /// Candidates are ranked in ascending order. The highest ranked candidate has a rank of `0`.
    pub fn ranked(&self) -> Vec<(T, usize)> {
        self.get_counted().into_ranked(0).into_vec()
    }

    /// Get a ranked list of winners. Winners with the same rank are tied.
    /// The number of winners might be greater than the requested `num_winners` if there is a tie.
    pub fn winners(&self) -> RankedWinners<T> {
        self.get_counted().into_ranked(self.condorcet.num_winners)
    }

    /// Build a graph representing all pairwise competitions between all candidates.
    ///
    /// Each candidate is assigned a node, vertexes between nodes contain a tuple of counts.
    /// Vertexes are directional, leading from the more preferred candidate to the less prefered candidate.
    /// The first element of tuple is the number votes where the first candidate is prefered to the second.
    /// The second element of the tuple is the number of votes where the second candidate is prefered to the first.
    /// The first element in the tuple is always greater than or equal to the second element in the tuple.
    ///
    /// If both candidates are equally prefered, two vertexes are created, one going in each direction.
    ///
    /// <img src="https://raw.githubusercontent.com/phayes/tallystick/master/docs/pairwise-graph.png" height="320px">
    /// Image Source: [https://arxiv.org/pdf/1804.02973.pdf](https://arxiv.org/pdf/1804.02973.pdf)
    pub fn build_graph(&self) -> Graph<T, (C, C)> {
        self.condorcet.build_graph()
    }

    // Check to make sure that if we are using ratio, we have a bounded and fractional type
    fn check_types(variant: &Variant) {
        if let Variant::Ratio = variant {
            if !C::fraction() || C::max_value() == C::zero() {
                panic!("tallystick::schulze: Variant::Ratio must be used with a type that is bounded and fractional.");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util;
    use std::io::Cursor;

    #[test]
    fn schulze_basic() -> Result<(), TallyError> {
        let mut tally = DefaultSchulzeTally::<&str>::new(1, Variant::Winning);
        tally.add_candidates(vec!["Notorious RBG", "Judge Judy", "Judge Dredd", "Abe Vigoda"]);

        tally.add(&vec!["Notorious RBG", "Judge Judy"])?;
        tally.add(&vec!["Judge Dredd"])?;
        tally.add(&vec!["Abe Vigoda", "Notorious RBG"])?;
        tally.add(&vec!["Notorious RBG", "Judge Dredd"])?;

        assert_eq!(tally.winners().into_unranked()[0], "Notorious RBG");

        Ok(())
    }

    #[test]
    fn schulze_wikipedia() -> Result<(), TallyError> {
        // See: https://en.wikipedia.org/wiki/Schulze_method

        let mut tally = DefaultSchulzeTally::with_candidates(1, Variant::Winning, vec!["A", "B", "C", "D", "E"]);
        tally.add_weighted(&vec!["A", "C", "B", "E", "D"], 5)?;
        tally.add_weighted(&vec!["A", "D", "E", "C", "B"], 5)?;
        tally.add_weighted(&vec!["B", "E", "D", "A", "C"], 8)?;
        tally.add_weighted(&vec!["C", "A", "B", "E", "D"], 3)?;
        tally.add_weighted(&vec!["C", "A", "E", "B", "D"], 7)?;
        tally.add_weighted(&vec!["C", "B", "A", "D", "E"], 2)?;
        tally.add_weighted(&vec!["D", "C", "E", "B", "A"], 7)?;
        tally.add_weighted(&vec!["E", "B", "A", "D", "C"], 8)?;

        // Verify totals
        let totals = tally.totals();
        for (pairwise, total) in totals.iter() {
            match pairwise {
                ("A", "B") => assert_eq!(*total, 20),
                ("A", "C") => assert_eq!(*total, 26),
                ("A", "D") => assert_eq!(*total, 30),
                ("A", "E") => assert_eq!(*total, 22),

                ("B", "A") => assert_eq!(*total, 25),
                ("B", "C") => assert_eq!(*total, 16),
                ("B", "D") => assert_eq!(*total, 33),
                ("B", "E") => assert_eq!(*total, 18),

                ("C", "A") => assert_eq!(*total, 19),
                ("C", "B") => assert_eq!(*total, 29),
                ("C", "D") => assert_eq!(*total, 17),
                ("C", "E") => assert_eq!(*total, 24),

                ("D", "A") => assert_eq!(*total, 15),
                ("D", "B") => assert_eq!(*total, 12),
                ("D", "C") => assert_eq!(*total, 28),
                ("D", "E") => assert_eq!(*total, 14),

                ("E", "A") => assert_eq!(*total, 23),
                ("E", "B") => assert_eq!(*total, 27),
                ("E", "C") => assert_eq!(*total, 21),
                ("E", "D") => assert_eq!(*total, 31),

                _ => panic!("Invalid schulze total pairwise"),
            }
        }

        // Verify strongest paths:
        let strongest = tally.strongest_paths();
        for (pairwise, strength) in strongest.iter() {
            match pairwise {
                ("A", "B") => assert_eq!(*strength, 28),
                ("A", "C") => assert_eq!(*strength, 28),
                ("A", "D") => assert_eq!(*strength, 30),
                ("A", "E") => assert_eq!(*strength, 24),

                ("B", "A") => assert_eq!(*strength, 25),
                ("B", "C") => assert_eq!(*strength, 28),
                ("B", "D") => assert_eq!(*strength, 33),
                ("B", "E") => assert_eq!(*strength, 24),

                ("C", "A") => assert_eq!(*strength, 25),
                ("C", "B") => assert_eq!(*strength, 29),
                ("C", "D") => assert_eq!(*strength, 29),
                ("C", "E") => assert_eq!(*strength, 24),

                ("D", "A") => assert_eq!(*strength, 25),
                ("D", "B") => assert_eq!(*strength, 28),
                ("D", "C") => assert_eq!(*strength, 28),
                ("D", "E") => assert_eq!(*strength, 24),

                ("E", "A") => assert_eq!(*strength, 25),
                ("E", "B") => assert_eq!(*strength, 28),
                ("E", "C") => assert_eq!(*strength, 28),
                ("E", "D") => assert_eq!(*strength, 31),

                _ => panic!("Invalid schulze strength pairwise"),
            }
        }

        // Verify ranking
        let ranked = tally.ranked();
        assert_eq!(ranked, vec![("E", 0), ("A", 1), ("C", 2), ("B", 3), ("D", 4)]);

        Ok(())
    }

    #[test]
    fn schulze_favorite_betrayal() -> Result<(), TallyError> {
        // See: https://rangevoting.org/WinningVotes.html

        // Original scenario
        let mut tally = DefaultSchulzeTally::with_candidates(1, Variant::Winning, vec!["A", "B", "C"]);
        tally.add_weighted(&vec!["B", "C", "A"], 9)?;
        tally.add_weighted(&vec!["C", "A", "B"], 6)?;
        tally.add_weighted(&vec!["A", "B", "C"], 5)?;
        assert_eq!(tally.winners().into_unranked()[0], "B");

        // Strategic vote change fully-betraying C with winning variant - betrayal works
        let mut tally = DefaultSchulzeTally::with_candidates(1, Variant::Winning, vec!["A", "B", "C"]);
        tally.add_weighted(&vec!["B", "C", "A"], 9)?;
        tally.add_weighted(&vec!["A", "C", "B"], 6)?;
        tally.add_weighted(&vec!["A", "B", "C"], 5)?;
        assert_eq!(tally.winners().into_unranked()[0], "A");

        // Strategic vote change fully-betraying C with margin variant - betrayal works
        let mut tally = DefaultSchulzeTally::with_candidates(1, Variant::Margin, vec!["A", "B", "C"]);
        tally.add_weighted(&vec!["B", "C", "A"], 9)?;
        tally.add_weighted(&vec!["A", "C", "B"], 6)?;
        tally.add_weighted(&vec!["A", "B", "C"], 5)?;
        assert_eq!(tally.winners().into_unranked()[0], "A");

        // Strategic vote change partly-betraying C with winning - betrayal works
        let mut tally = DefaultSchulzeTally::with_candidates(1, Variant::Winning, vec!["A", "B", "C"]);
        tally.add_weighted(&vec!["B", "C", "A"], 9)?;
        tally.ranked_add_weighted(&vec![("A", 0), ("C", 0), ("B", 1)], 6)?;
        tally.add_weighted(&vec!["A", "B", "C"], 5)?;
        assert_eq!(tally.winners().into_unranked()[0], "A");

        // Strategic vote change partly-betraying C with margin - betrayal fails
        let mut tally = DefaultSchulzeTally::with_candidates(1, Variant::Margin, vec!["A", "B", "C"]);
        tally.add_weighted(&vec!["B", "C", "A"], 9)?;
        tally.ranked_add_weighted(&vec![("A", 0), ("C", 0), ("B", 1)], 6)?;
        tally.add_weighted(&vec!["A", "B", "C"], 5)?;
        assert_eq!(tally.winners().into_unranked()[0], "B");

        Ok(())
    }

    #[test]
    fn schulze_example_1() -> Result<(), TallyError> {
        // See: https://arxiv.org/pdf/1804.02973.pdf (example 1)
        let candidates = vec!["A".to_string(), "B".to_string(), "C".to_string(), "D".to_string()];

        let votes_raw = "
    A > C > D > B * 8
    B > A > D > C * 2
    C > D > B > A * 4
    D > B > A > C * 4
    D > C > B > A * 3
    ";

        let votes = Cursor::new(votes_raw);
        let mut votes = util::read_votes(votes).unwrap();

        let mut tally = DefaultSchulzeTally::with_candidates(1, Variant::Winning, candidates.clone());
        for (vote, weight) in votes.drain(..) {
            let vote = vote.into_ranked();
            tally.ranked_add_weighted(&vote, weight)?;
        }

        assert_eq!(tally.winners().into_unranked()[0], "D".to_string());

        Ok(())
    }

    #[test]
    fn schulze_example_2() -> Result<(), TallyError> {
        // See: https://arxiv.org/pdf/1804.02973.pdf (example 2)
        let candidates = vec!["A".to_string(), "B".to_string(), "C".to_string(), "D".to_string()];

        let votes_raw = "
    A > C > D > B * 3
    B > A > C > D * 9
    C > D > A > B * 8
    D > A > B > C * 5
    D > B > C > A * 5
    ";

        let votes = Cursor::new(votes_raw);
        let mut votes = util::read_votes(votes).unwrap();

        let mut tally = DefaultSchulzeTally::with_candidates(1, Variant::Winning, candidates.clone());
        for (vote, weight) in votes.drain(..) {
            let vote = vote.into_ranked();
            tally.ranked_add_weighted(&vote, weight)?;
        }

        assert_eq!(tally.winners().into_unranked()[0], "C".to_string());

        Ok(())
    }

    #[test]
    fn schulze_example_3() -> Result<(), TallyError> {
        // See: https://arxiv.org/pdf/1804.02973.pdf (example 3)
        let candidates = vec!["A".to_string(), "B".to_string(), "C".to_string(), "D".to_string(), "E".to_string()];

        let votes_raw = "
    A > C > B > E > D * 5
    A > D > E > C > B * 5
    B > E > D > A > C * 8
    C > A > B > E > D * 3
    C > A > E > B > D * 7
    C > B > A > D > E * 2
    D > C > E > B > A * 7
    E > B > A > D > C * 8
    ";

        let votes = Cursor::new(votes_raw);
        let mut votes = util::read_votes(votes).unwrap();

        let mut tally = DefaultSchulzeTally::with_candidates(1, Variant::Winning, candidates.clone());
        for (vote, weight) in votes.drain(..) {
            let vote = vote.into_ranked();
            tally.ranked_add_weighted(&vote, weight)?;
        }

        assert_eq!(tally.winners().into_unranked()[0], "E".to_string());

        Ok(())
    }

    #[test]
    fn schulze_example_4() -> Result<(), TallyError> {
        // See: https://arxiv.org/pdf/1804.02973.pdf (example 4)
        let candidates = vec!["A".to_string(), "B".to_string(), "C".to_string(), "D".to_string()];

        let votes_raw = "
    A > B > C > D * 3
    C > B > D > A * 2
    D > A > B > C * 2
    D > B > C > A * 2
    ";

        let votes = Cursor::new(votes_raw);
        let mut votes = util::read_votes(votes).unwrap();

        let mut tally = DefaultSchulzeTally::with_candidates(1, Variant::Winning, candidates.clone());
        for (vote, weight) in votes.drain(..) {
            let vote = vote.into_ranked();
            tally.ranked_add_weighted(&vote, weight)?;
        }

        // Verify winners - "B" and "D" are tied.
        let winners = tally.winners().into_unranked();
        assert!(winners.contains(&"B".to_owned()) && winners.contains(&"D".to_owned()));
        assert!(winners.len() == 2);

        Ok(())
    }

    #[test]
    fn schulze_example_5() -> Result<(), TallyError> {
        // See Example 5: https://arxiv.org/pdf/1804.02973.pdf

        let mut tally = DefaultSchulzeTally::with_candidates(1, Variant::Winning, vec!["a", "b", "c", "d"]);
        tally.add_weighted(&vec!["a", "b", "c", "d"], 12)?;
        tally.add_weighted(&vec!["a", "d", "b", "c"], 6)?;
        tally.add_weighted(&vec!["b", "c", "d", "a"], 9)?;
        tally.add_weighted(&vec!["c", "d", "a", "b"], 15)?;
        tally.add_weighted(&vec!["d", "b", "a", "c"], 21)?;

        // Verify ranking - "a" and "b" are tied.
        let ranked = tally.ranked();
        assert!(ranked == vec![("d", 0), ("a", 1), ("b", 1), ("c", 2)] || ranked == vec![("d", 0), ("b", 1), ("a", 1), ("c", 2)]);

        Ok(())
    }

    #[test]
    fn schulze_example_6() -> Result<(), TallyError> {
        // See: https://arxiv.org/pdf/1804.02973.pdf (example 6)
        let candidates = vec!["A".to_string(), "B".to_string(), "C".to_string(), "D".to_string()];

        let votes_raw = "
    A > C > D * 6
    B > A > D
    C > B > D * 3
    D > B > A * 3
    D > C > B * 2
    ";

        let votes = Cursor::new(votes_raw);
        let mut votes = util::read_votes(votes).unwrap();

        let mut tally = DefaultSchulzeTally::with_candidates(1, Variant::Winning, candidates.clone());
        for (vote, weight) in votes.drain(..) {
            let vote = vote.into_ranked();
            tally.ranked_add_weighted(&vote, weight)?;
        }

        // Verify winners - "A" and "D" are tied.
        let winners = tally.winners().into_unranked();
        assert!(winners.contains(&"A".to_owned()) && winners.contains(&"D".to_owned()));
        assert!(winners.len() == 2);

        Ok(())
    }

    #[test]
    fn schulze_example_7() -> Result<(), TallyError> {
        // See: https://arxiv.org/pdf/1804.02973.pdf (example 7)
        let candidates = vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
            "D".to_string(),
            "E".to_string(),
            "F".to_string(),
        ];

        let votes_raw = "
    A > D > E > B > C > F * 3
    B > F > E > C > D > A * 3
    C > A > B > F > D > E * 4
    D > B > C > E > F > A * 1
    D > E > F > A > B > C * 4
    E > C > B > D > F > A * 2
    F > A > C > D > B > E * 2
    ";

        let votes = Cursor::new(votes_raw);
        let mut votes = util::read_votes(votes).unwrap();

        let mut tally = DefaultSchulzeTally::with_candidates(1, Variant::Winning, candidates.clone());
        for (vote, weight) in votes.drain(..) {
            let vote = vote.into_ranked();
            tally.ranked_add_weighted(&vote, weight)?;
        }

        // Verify winners - "A"
        assert_eq!(tally.winners().into_unranked()[0], "A".to_string());

        // Add additional votes
        tally.add_weighted(
            &vec![
                "A".to_string(),
                "E".to_string(),
                "F".to_string(),
                "C".to_string(),
                "B".to_string(),
                "D".to_string(),
            ],
            2,
        )?;

        // Verify winners - "D"
        assert_eq!(tally.winners().into_unranked()[0], "D".to_string());

        Ok(())
    }

    #[test]
    fn schulze_example_10() -> Result<(), TallyError> {
        // See: https://github.com/julien-boudry/Condorcet/blob/master/Tests/lib/Algo/Methods/SchulzeTest.php#L219-L252
        // See: https://arxiv.org/pdf/1804.02973.pdf (Example 10)

        let candidates = vec!["A".to_string(), "B".to_string(), "C".to_string(), "D".to_string()];

        let votes_raw = "
    A > B > C > D * 6
    A = B * 8
    A = C * 8
    A = C > D * 18
    A = C = D * 8
    B * 40
    C > B > D * 4
    C > D > A * 9
    C = D * 8
    D > A > B * 14
    D > B > C * 11
    D > C > A * 4";

        let votes = Cursor::new(votes_raw);
        let votes = util::read_votes(votes).unwrap();

        // Margin
        let mut tally = DefaultSchulzeTally::with_candidates(1, Variant::Margin, candidates.clone());
        for (vote, weight) in votes.iter() {
            match vote {
                util::ParsedVote::Ranked(v) => tally.ranked_add_weighted(v, *weight)?,
                util::ParsedVote::Unranked(v) => tally.add_weighted(v, *weight)?,
            }
        }
        assert_eq!(tally.winners().into_unranked()[0], "A".to_string());

        // Winning
        let mut tally = DefaultSchulzeTally::with_candidates(1, Variant::Winning, candidates.clone());
        for (vote, weight) in votes.iter() {
            match vote {
                util::ParsedVote::Ranked(v) => tally.ranked_add_weighted(v, *weight)?,
                util::ParsedVote::Unranked(v) => tally.add_weighted(v, *weight)?,
            }
        }
        assert_eq!(tally.winners().into_unranked()[0], "D".to_string());

        // Ratio
        let votes = Cursor::new(votes_raw);
        let votes = util::read_votes(votes).unwrap(); // reparse votes as f64
        let mut tally = SchulzeTally::<_, f64>::with_candidates(1, Variant::Ratio, candidates.clone());
        for (vote, weight) in votes.iter() {
            match vote {
                util::ParsedVote::Ranked(v) => tally.ranked_add_weighted(v, *weight)?,
                util::ParsedVote::Unranked(v) => tally.add_weighted(v, *weight)?,
            }
        }
        assert_eq!(tally.winners().into_unranked()[0], "B".to_string());

        Ok(())
    }
}
