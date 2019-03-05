use super::check_duplicate;
use super::RankedWinners;
use super::TallyError;
use hashbrown::HashMap;
use num_traits::cast::NumCast;
use num_traits::Num;
use petgraph::algo::tarjan_scc;
use petgraph::graph::NodeIndex;
use petgraph::Graph;
use std::hash::Hash;
use std::ops::AddAssign;

/// A condorcet tally using `u64` integers to count votes.
/// `DefaultCondorcetTally` is generally preferred over `CondorcetTally`, except when using vote weights that contains fractions.
/// Since this is an alias, refer to [`CondorcetTally`](struct.CondorcetTally.html) for method documentation.
///
/// # Example
/// ```
///    use tallystick::condorcet::DefaultCondorcetTally;
///
///    // What is your favourite colour?
///    // A vote with hexadecimal colour candidates and a single-winner.
///    let red = 0xff0000;
///    let blue = 0x00ff00;
///    let green = 0x0000ff;
///    let mut tally = DefaultCondorcetTally::new(1);
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
pub type DefaultCondorcetTally<T> = CondorcetTally<T, u64>;

/// A generic condorcet tally.
///
/// Generics:
/// - `T`: The candidate type.
/// - `C`: The count type. `u64` is recommended, but can be modified to use a different type for counting votes (eg `f64` for fractional vote weights).
///
/// Example:
/// ```
///    use tallystick::condorcet::CondorcetTally;
///
///    // A tally with string candidates, one winner, and `f64` counting.
///    let mut tally = CondorcetTally::<&str, f64>::new(1);
///    tally.add(vec!["Alice", "Bob", "Carlos"]);
///    tally.add(vec!["Bob", "Carlos", "Alice"]);
///    tally.add(vec!["Alice", "Carlos", "Bob"]);
///    tally.add(vec!["Alice", "Bob", "Carlos"]);
///
///    let winners = tally.winners();
/// ```
pub struct CondorcetTally<T, C = u64>
where
    T: Eq + Clone + Hash,                             // Candidate type
    C: Copy + PartialOrd + AddAssign + Num + NumCast, // Count type
{
    running_total: HashMap<(usize, usize), C>,
    num_winners: u32,
    candidates: HashMap<T, usize>, // Map candiates to a unique integer identifiers
}

impl<T, C> CondorcetTally<T, C>
where
    T: Eq + Clone + Hash,                             // Candidate type
    C: Copy + PartialOrd + AddAssign + Num + NumCast, // Count type
{
    /// Create a new `CondorcetTally` with the given number of winners.
    ///
    /// If there is a tie, the number of winners might be more than `num_winners`.
    /// (See [`winners()`](#method.winners) for more information on ties.)
    pub fn new(num_winners: u32) -> Self {
        return CondorcetTally {
            running_total: HashMap::new(),
            num_winners: num_winners,
            candidates: HashMap::new(),
        };
    }

    /// Create a new `CondorcetTally` with the given number of winners, and number of expected candidates.
    pub fn with_capacity(num_winners: u32, expected_candidates: usize) -> Self {
        return CondorcetTally {
            running_total: HashMap::with_capacity(expected_candidates ^ 2),
            num_winners: num_winners,
            candidates: HashMap::with_capacity(expected_candidates),
        };
    }

    /// Add a new vote
    pub fn add(&mut self, vote: Vec<T>) -> Result<(), TallyError> {
        self.add_weighted_ref(&vote, C::one())
    }

    /// Add a vote by reference.
    pub fn add_ref(&mut self, vote: &[T]) -> Result<(), TallyError> {
        self.add_weighted_ref(vote, C::one())
    }

    /// Add a weighted vote.
    /// By default takes a weight as a `usize` integer, but can be customized by using `CondorcetTally` with a custom count type.
    pub fn add_weighted(&mut self, vote: Vec<T>, weight: C) -> Result<(), TallyError> {
        self.add_weighted_ref(&vote, weight)
    }

    /// Add a weighted vote by reference.
    pub fn add_weighted_ref(&mut self, vote: &[T], weight: C) -> Result<(), TallyError> {
        if vote.is_empty() {
            return Ok(());
        }
        check_duplicate(vote)?;

        let selection = self.mapped_candidates(&vote);

        for (i, candidate) in selection.iter().enumerate() {
            let mut j = i + 1;
            while let Some(candidate_2) = selection.get(j) {
                *self.running_total.entry((*candidate, *candidate_2)).or_insert(C::zero()) += weight;
                j += 1;
            }
        }

        Ok(())
    }

    /// Get total counts for this tally.
    /// Totals are returned as a list of pairwise comparisons
    /// For a pairwise comparison `((T1, T2), C)`, `C` is the number of votes where candidate `T1` is preferred over candidate `T2`.
    ///
    /// # Example
    /// ```
    ///    use tallystick::condorcet::DefaultCondorcetTally;
    ///
    ///    let mut tally = DefaultCondorcetTally::new(1);
    ///    for _ in 0..30 { tally.add(vec!["Alice", "Bob"]).unwrap() }
    ///    for _ in 0..10 { tally.add(vec!["Bob", "Alice"]).unwrap() }
    ///
    ///    for ((candidate1, candidate2), num_votes) in tally.totals().iter() {
    ///       println!("{} is preferred over {} {} times", candidate1, candidate2, num_votes);
    ///    }
    ///    // Prints:
    ///    //   Alice is preferred over Bob 30 times
    ///    //   Bob is preferred over Alice 10 times
    /// ```
    // TODO: 'C' could just be a regular integer (usize?).
    pub fn totals(&self) -> Vec<((T, T), C)> {
        let mut totals = Vec::<((T, T), C)>::with_capacity(self.running_total.len());

        // Invert the candidate map.
        let mut candidates = HashMap::<usize, T>::with_capacity(self.candidates.len());
        for (candidate, i) in self.candidates.iter() {
            candidates.insert(*i, candidate.clone());
        }

        for ((candidate1, candidate2), count) in self.running_total.iter() {
            // Ok to unwrap here since candidates must exist.
            let candidate1 = candidates.get(candidate1).unwrap().clone();
            let candidate2 = candidates.get(candidate2).unwrap().clone();
            totals.push(((candidate1, candidate2), *count));
        }

        return totals;
    }

    /// Get a ranked list of all candidates. Candidates with the same rank are tied.
    /// Candidates are ranked in ascending order. The highest ranked candidate has a rank of `0`.
    ///
    /// # Example
    /// ```
    ///    use tallystick::condorcet::DefaultCondorcetTally;
    ///
    ///    let mut tally = DefaultCondorcetTally::new(1);
    ///    for _ in 0..50 { tally.add(vec!["Alice", "Bob", "Carlos"]); }
    ///    for _ in 0..40 { tally.add(vec!["Bob", "Carlos", "Alice"]); }
    ///    for _ in 0..30 { tally.add(vec!["Carlos", "Alice", "Bob"]); }
    ///    
    ///    for (candidate, rank) in tally.ranked().iter() {
    ///       println!("{} has a rank of {}", candidate, rank);
    ///    }
    ///    // Prints:
    ///    //   Alice has a rank of 0
    ///    //   Bob has a rank of 1
    ///    //   Carlos has a rank of 2
    /// ```
    pub fn ranked(&self) -> Vec<(T, u32)> {
        // Compute smith-sets using Tarjan's strongly connected components algorithm.
        let graph = self.build_graph();
        let smith_sets = tarjan_scc(&graph);

        // Invert the candidate map, cloned candidates will be moved into the winners list.
        let mut candidates = HashMap::<usize, T>::with_capacity(self.candidates.len());
        for (candidate, i) in self.candidates.iter() {
            candidates.insert(*i, candidate.clone());
        }

        // Add to ranked list.
        let mut ranked = Vec::<(T, u32)>::with_capacity(self.candidates.len());
        for (rank, smith_set) in smith_sets.iter().enumerate() {
            // We need to add all members of a smith set at the same time,
            // even if it means more winners than needed. All members of a smith_set
            // have the same rank.

            // TODO: Check performance difference between cloning here and using a stable graph (where we can remove_node())
            for graph_id in smith_set.iter() {
                let candidate = graph.node_weight(*graph_id).unwrap(); // Safe to unwrap here since graph should always contain a node-weight at this graph-id.
                ranked.push((candidate.clone(), rank as u32));
            }
        }

        return ranked;
    }

    /// Get a ranked list of winners. Winners with the same rank are tied.
    /// The number of winners might be greater than the requested `num_winners` if there is a tie.
    ///
    /// # Example
    /// ```
    ///    use tallystick::condorcet::DefaultCondorcetTally;
    ///
    ///    let mut tally = DefaultCondorcetTally::new(2); // We ideally want only 2 winnners
    ///    tally.add_weighted(vec!["Alice"], 3);
    ///    tally.add_weighted(vec!["Bob", "Carlos", "Alice"], 2);
    ///    tally.add_weighted(vec!["Carlos", "Alice", "Bob"], 2);
    ///    tally.add(vec!["Dave"]); // implicit weight of 1
    ///
    ///    let winners = tally.winners();
    ///
    ///    println!("We have {} winners", winners.len());
    ///    // Prints: "We have 3 winners" (due to Carlos and Bob being tied)
    ///    
    ///    // Check for ties that overflow the wanted number of winners
    ///    if winners.check_overflow() {
    ///        println!("There are more winners than seats.")
    ///    }
    ///    if let Some(overflow_winners) = winners.overflow() {
    ///        println!("We need to resolve the following overflowing tie between:");
    ///        for overflow_winner in overflow_winners {
    ///            println!("\t {}", overflow_winner);
    ///        }
    ///    }
    ///    
    ///    // Print all winners by rank
    ///    for (winner, rank) in winners.iter() {
    ///        println!("{} has a rank of {}", winner, rank);
    ///    }
    ///    // Prints:
    ///    //   Alice has a rank of 0
    ///    //   Bob has a rank of 1
    ///    //   Carlos has a rank of 1
    /// ```
    pub fn winners(&self) -> RankedWinners<T> {
        return RankedWinners::from_ranked(self.ranked(), self.num_winners);
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
        let mut graph = Graph::<T, (C, C)>::with_capacity(self.candidates.len(), self.candidates.len() ^ 2);

        // Add all candidates
        let mut graph_ids = HashMap::<usize, NodeIndex>::new();
        for (candidate, candidate_id) in self.candidates.iter() {
            graph_ids.insert(*candidate_id, graph.add_node(candidate.clone()));
        }

        let zero = C::zero();
        for ((candidate_1, candidate_2), votecount_1) in self.running_total.iter() {
            let votecount_2 = self.running_total.get(&(*candidate_2, *candidate_1)).unwrap_or(&zero);

            // Only add if candidate_1 vs candidate_2 votecount is larger than candidate_2 vs candidate_1 votecount
            // Otherwise we will catch it when we come around to it again.
            if votecount_1 >= votecount_2 {
                let candidate_1_id = graph_ids.get(candidate_1).unwrap(); // Safe to unwrap since graph-ids contain all candidates.
                let candidate_2_id = graph_ids.get(candidate_2).unwrap();
                graph.add_edge(*candidate_2_id, *candidate_1_id, (*votecount_1, *votecount_2));
            }
        }

        return graph;
    }

    /// Get a list of all candidates seen by this tally.
    /// Candidates are returned in no particular order.
    pub fn candidates(&self) -> Vec<T> {
        return self.candidates.iter().map(|(k, _v)| k.clone()).collect();
    }

    // Ensure that candidates are in our list of candidates, and return an internal numeric representation of the same
    fn mapped_candidates(&mut self, selection: &[T]) -> Vec<usize> {
        let mut mapped = Vec::<usize>::new();
        for selected in selection.iter() {
            if self.candidates.contains_key(&selected) {
                mapped.push(*self.candidates.get(&selected).unwrap()); // Safe to unwrap here since we just checked it one-line above with contains_key()
            } else {
                let len = self.candidates.len();
                self.candidates.insert(selected.clone(), len);
                mapped.push(len);
            }
        }
        return mapped;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::hashset;
    use std::collections::HashSet;
    use std::iter::FromIterator;

    #[test]
    fn condorcet_basic() -> Result<(), TallyError> {
        // Election between Alice, Bob, and Carol
        let mut tally = DefaultCondorcetTally::new(2);
        tally.add(vec!["Alice", "Bob", "Carol"])?;
        tally.add(vec!["Alice", "Bob", "Carol"])?;
        tally.add(vec!["Alice", "Bob", "Carol"])?;

        let totals = tally.totals();
        let totals = HashSet::from_iter(totals.iter().cloned()); // As a hashset.
        assert_eq!(
            totals,
            hashset![(("Alice", "Bob"), 3), (("Bob", "Carol"), 3), (("Alice", "Carol"), 3),]
        );

        let winners = tally.winners();
        assert_eq!(winners.into_vec(), vec! {("Alice", 0), ("Bob", 1)});

        // Test a non-transitive voting paradox
        let mut tally = DefaultCondorcetTally::new(1);
        tally.add(vec!["Alice", "Bob", "Carol"])?;
        tally.add(vec!["Bob", "Carol", "Alice"])?;
        tally.add(vec!["Carol", "Alice", "Bob"])?;

        let winners = tally.winners();
        assert_eq!(winners.rank(&"Alice").unwrap(), 0);
        assert_eq!(winners.rank(&"Bob").unwrap(), 0);
        assert_eq!(winners.rank(&"Carol").unwrap(), 0);

        Ok(())
    }

    #[test]
    fn condorcet_wikipedia() -> Result<(), TallyError> {
        // From: https://en.wikipedia.org/wiki/Condorcet_method
        let mut tally = DefaultCondorcetTally::with_capacity(4, 4);
        tally.add_weighted(vec!["Memphis", "Nashville", "Chattanooga", "Knoxville"], 42)?;
        tally.add_weighted(vec!["Nashville", "Chattanooga", "Knoxville", "Memphis"], 26)?;
        tally.add_weighted(vec!["Chattanooga", "Knoxville", "Nashville", "Memphis"], 15)?;
        tally.add_weighted(vec!["Knoxville", "Chattanooga", "Nashville", "Memphis"], 17)?;

        let candidates = tally.candidates();
        let candidates = HashSet::from_iter(candidates.iter().cloned()); // As a hashset
        assert_eq!(candidates, hashset!["Memphis", "Nashville", "Chattanooga", "Knoxville"]);

        let totals = tally.totals();
        let totals = HashSet::from_iter(totals.iter().cloned()); // As a hashset
        assert_eq!(
            totals,
            hashset![
                (("Memphis", "Nashville"), 42),
                (("Nashville", "Memphis"), 58),
                (("Memphis", "Chattanooga"), 42),
                (("Chattanooga", "Memphis"), 58),
                (("Memphis", "Knoxville"), 42),
                (("Knoxville", "Memphis"), 58),
                (("Nashville", "Chattanooga"), 68),
                (("Chattanooga", "Nashville"), 32),
                (("Nashville", "Knoxville"), 68),
                (("Knoxville", "Nashville"), 32),
                (("Chattanooga", "Knoxville"), 83),
                (("Knoxville", "Chattanooga"), 17),
            ]
        );

        let winners = tally.winners();
        assert_eq!(
            winners.into_vec(),
            vec! {("Nashville", 0), ("Chattanooga", 1), ("Knoxville", 2), ("Memphis", 3)}
        );

        Ok(())
    }

    #[test]
    fn condorcet_graph() -> Result<(), TallyError> {
        // From: https://arxiv.org/pdf/1804.02973.pdf

        // Example 1:
        let mut tally = DefaultCondorcetTally::new(1);
        tally.add_weighted(vec!["a", "c", "d", "b"], 8)?;
        tally.add_weighted(vec!["b", "a", "d", "c"], 2)?;
        tally.add_weighted(vec!["c", "d", "b", "a"], 4)?;
        tally.add_weighted(vec!["d", "b", "a", "c"], 4)?;
        tally.add_weighted(vec!["d", "c", "b", "a"], 3)?;

        let graph = tally.build_graph();
        assert_eq!(graph.node_count(), 4);
        assert_eq!(graph.edge_count(), 6);

        for index in graph.node_indices() {
            let candidate = *graph.node_weight(index).unwrap();
            for edge in graph.edges(index).map(|e| e.weight()) {
                match candidate {
                    "a" => assert!(*edge == (13, 8) || *edge == (11, 10) || *edge == (14, 7)),
                    "b" => assert!(*edge == (13, 8) || *edge == (15, 6) || *edge == (19, 2)),
                    "c" => assert!(*edge == (12, 9) || *edge == (15, 6) || *edge == (14, 7)),
                    "d" => assert!(*edge == (12, 9) || *edge == (11, 10) || *edge == (19, 2)),
                    _ => panic!("Invalid candidate"),
                }
            }
        }

        Ok(())
    }
}
