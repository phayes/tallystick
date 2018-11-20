use std::hash::Hash;
use std::ops::AddAssign;
use hashbrown::HashMap;
use num_traits::cast::NumCast;
use num_traits::Num;
use petgraph::Graph;
use petgraph::graph::NodeIndex;
use petgraph::algo::tarjan_scc;
use super::RankedWinners;

pub type DefaultTally<T> = Tally<T, u64>;

pub struct Tally<T, C = u64>
    where T: Eq + Clone + Hash,                            // Candidate type
          C: Copy + PartialOrd + AddAssign + Num + NumCast // Count type
{
    running_total: HashMap<(usize, usize), C>,
    num_winners: u32,
    candidates: HashMap<T, usize> // Map candiates to a unique integer identifiers
}

impl<T, C> Tally<T, C>
    where
          T: Eq + Clone + Hash,                            // Candidate type
          C: Copy + PartialOrd + AddAssign + Num + NumCast // Count type
{

    pub fn new(num_winners: u32) -> Self {
        return Tally {
            running_total: HashMap::new(),
            num_winners: num_winners,
            candidates: HashMap::new()
        };
    }

    pub fn add(&mut self, selection: Vec<T>) {
       self.add_weighted_ref(&selection, C::one());
    }

    pub fn add_ref(&mut self, selection: &Vec<T>) {
        self.add_weighted_ref(selection, C::one());
    }

    pub fn add_weighted(&mut self, selection: Vec<T>, weight: C) {
        self.add_weighted_ref(&selection, weight);
    }

    pub fn add_weighted_ref(&mut self, selection: &Vec<T>, weight: C) {
        // TODO: ensure votes are transitive.
        if selection.is_empty() {
            return;
        }

        let selection = self.mapped_candidates(&selection);

        for (i, candidate) in selection.iter().enumerate() {
            let mut j = i + 1;
            while let Some(candidate_2) = selection.get(j) {
                *self.running_total.entry((*candidate, *candidate_2)).or_insert(C::zero()) += weight;
                j += 1;
            }
        }
    }

    pub fn reset(&mut self) {
        self.running_total = HashMap::new();
        self.candidates = HashMap::new();
    }
    
    pub fn winners(&mut self) -> RankedWinners<T> {
        // Compute smith-sets using Tarjan's strongly connected components algorithm.
        let graph = self.build_graph();
        let smith_sets = tarjan_scc(&graph);

        // Inverse the candidate map, cloned candidates will be moved into the winners list.
        let mut candidates = HashMap::<usize, T>::with_capacity(self.candidates.len());
        for (candidate, i) in self.candidates.iter() {
            candidates.insert(*i, candidate.clone());
        }

        // Add to winners list.
        let mut winners = RankedWinners::new(self.num_winners);
        let mut rank = 0;
        for smith_set in smith_sets.iter() {
            if winners.len() as u32 >= self.num_winners {
                break;
            }

            // We need to add all members of a smith set at the same time,
            // even if it means more winners than needed. All members of a smith_set
            // have the same rank.
            for graph_id in smith_set.iter() {
                let candidate_internal_id = graph.node_weight(*graph_id).unwrap();
                let candidate = candidates.remove(candidate_internal_id).unwrap();
                winners.push(candidate, rank);
            }
            rank += 1;
        }
        
        return winners;
    }

    crate fn build_graph(&mut self) -> Graph<usize, ()> {
        let mut graph = Graph::<usize, ()>::with_capacity(self.candidates.len(), self.candidates.len()^2);

        // Add all candidates
        let mut graph_ids = HashMap::<usize, NodeIndex>::new();
        for (_, candidate) in self.candidates.iter() {
            graph_ids.insert(*candidate, graph.add_node(*candidate));
        }

        let zero = C::zero();
        for ((candidate_1, candidate_2), votecount_1) in self.running_total.iter() {
            let votecount_2 = self.running_total.get(&(*candidate_2, *candidate_1)).unwrap_or(&zero);

            // Only add if candidate_1 vs candidate_2 votecount is larger than candidate_2 vs candidate_1 votecount
            // Otherwise we will catch it when we come around to it again.
            if votecount_1 >= votecount_2 {
                let candidate_1_id = graph_ids.get(candidate_1).unwrap();
                let candidate_2_id = graph_ids.get(candidate_2).unwrap();
                graph.add_edge(*candidate_2_id, *candidate_1_id, ());
            }
        }

        return graph;
    }

    // Ensure that candidates are in our list of candidates, and return an internal numeric representation of the same
    fn mapped_candidates(&mut self, selection: &Vec<T>) -> Vec<usize> {
        let mut mapped = Vec::<usize>::new();
        for selected in selection.iter() {
            if self.candidates.contains_key(&selected) {
                mapped.push(*self.candidates.get(&selected).unwrap());
            }
            else {
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

    #[test]
    fn condorcet_test() {
        // Election between Alice, Bob, and Cir
        let mut tally = DefaultTally::new(2);
        tally.add(vec!["Alice", "Bob", "Cir"]);
        tally.add(vec!["Alice", "Bob", "Cir"]);
        tally.add(vec!["Alice", "Bob", "Cir"]);

        let winners = tally.winners();
        assert_eq!(winners.into_vec(), vec!{("Alice", 0), ("Bob", 1)});

        // Test a full voting paradox
        let mut tally = DefaultTally::new(1);
        tally.add(vec!["Alice", "Bob", "Cir"]);
        tally.add(vec!["Bob", "Cir", "Alice"]);
        tally.add(vec!["Cir", "Alice", "Bob"]);

        let winners = tally.winners();
        assert_eq!(winners.rank(&"Alice").unwrap(), 0);
        assert_eq!(winners.rank(&"Bob").unwrap(), 0);
        assert_eq!(winners.rank(&"Cir").unwrap(), 0);
    }
    
    #[test]
    fn condorcet_wikipedia_test() {
        // From: https://en.wikipedia.org/wiki/Condorcet_method
        let mut tally = DefaultTally::new(4);
        tally.add_weighted(vec!["Memphis", "Nashville", "Chattanooga", "Knoxville"], 42);
        tally.add_weighted(vec!["Nashville", "Chattanooga", "Knoxville", "Memphis"], 26);
        tally.add_weighted(vec!["Chattanooga", "Knoxville", "Nashville", "Memphis"], 15);
        tally.add_weighted(vec!["Knoxville", "Chattanooga", "Nashville", "Memphis"], 17);

        let winners = tally.winners();
        assert_eq!(winners.into_vec(), vec!{("Nashville", 0), ("Chattanooga", 1), ("Knoxville", 2), ("Memphis", 3)});
    }
}
