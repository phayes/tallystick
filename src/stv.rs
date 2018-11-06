use std::hash::Hash;
use std::collections::HashMap;

use indexmap::IndexMap;

#[derive(Debug)]
struct WeightedVote<T: Eq + Clone + Hash + std::fmt::Debug> {
    weight: f64,
    remaining: Vec<T>,
}

pub enum Quota {

    // Droop Quota. If you are unsure what to use, use this one.
    Droop,

    // Hare quota.
    Hare,

    // Hagenbach-Bischoff quota.
    Hagenbach
}

pub struct Tally<T: Eq + Clone + Hash + std::fmt::Debug> {
    running_total: HashMap<T, Vec<WeightedVote<T>>>,
    num_winners: u32,
    quota: Quota,
    candidates: HashMap<T, usize> // Map candiates to a unique integer identifiers.
}

impl<T: Eq + Clone + Hash + std::fmt::Debug> Tally<T>  {

    pub fn new(num_winners: u32, quota: Quota) -> Self {
        return Tally {
            running_total: HashMap::new(),
            num_winners: num_winners,
            quota: quota,
            candidates: HashMap::new()
        };
    }

    pub fn add(&mut self, selection: Vec<T>) {
        if selection.is_empty() {
            return;
        }

        // Ensure all selections are in the candidates list
        self.mapped_candidates(&selection);
        
        let mut remaining = selection.clone();
        let choice = remaining.remove(0);

        let weighted_vote = WeightedVote {
            weight: 1.0,
            remaining: remaining
        };

        if self.running_total.contains_key(&choice) {
            if let Some(x) = self.running_total.get_mut(&choice) {
                x.push(weighted_vote);
            }
        }
        else {
            self.running_total.insert(choice, vec!(weighted_vote));
        }
    }

    pub fn result(&mut self) -> IndexMap<T, u32> {
        let threshold = match self.quota {
            Quota::Droop => self.droop_threshold(),
            Quota::Hare => self.hare_threshold(),
            Quota::Hagenbach => self.hagenbach_threshold(),
        };

        let mut results: IndexMap<T, u32> = IndexMap::new();

        let mut rank: u32 = 0;
        loop {
            // Check if any candidates are over the threshold
            let mut new_winners: Vec<T> = Vec::new();
            for (candidate, votes) in self.running_total.iter() {
                let mut votecount = 0.0;
                for vote in votes.iter() {
                    votecount += vote.weight;
                }
                if votecount as usize >= threshold {
                    new_winners.push(candidate.clone());
                }
            }

            // If we have enough winners, end the tally and return results.
            if (results.len() + new_winners.len()) as u32 > self.num_winners {
                for winner in new_winners.drain(0..) {
                    results.insert(winner, rank);
                }
                return results;
            }
            // If there's new winners, redistribute their excess vote.
            if new_winners.len() > 0 {
                for winner in new_winners.drain(0..) {
                    let mut votes = self.running_total.remove(&winner).unwrap();
                    let overvote = votes.len() - threshold;
                    let weight = overvote as f64 / votes.len() as f64;
                    
                    // TODO: feature-gate num-rational instead of f64 for determinism

                    // Redistibute to next choice
                    for vote in votes.drain(0..) {
                        if vote.remaining.len() > 0 {
                            let mut remaining = vote.remaining;
                            let next_choice = remaining.remove(0);
                            let weighted_vote = WeightedVote {
                                weight: weight * vote.weight,
                                remaining: remaining
                            };
                            if self.running_total.contains_key(&next_choice) {
                                if let Some(x) = self.running_total.get_mut(&next_choice) {
                                    x.push(weighted_vote);
                                }
                            }
                            else {
                                self.running_total.insert(next_choice, vec!(weighted_vote));
                            }
                        }
                    }
                    results.insert(winner, rank);
                }

                // If we have enough results, return it.
                if results.len() as u32 >= self.num_winners {
                    return results;
                }

                // We've added winners, so increase the rank and continue to the next round.
                rank = rank + 1;
                continue;
            }
            else {
                // Remove loosers and redistribute
                let mut new_loosers: Vec<T> = Vec::new();
                {
                    let mut votecounts: HashMap<&T, f64> = HashMap::new();
                    let mut least: f64 = std::f64::INFINITY;
                    for (candidate, votes) in self.running_total.iter() {
                        let mut votecount: f64 = 0.0;

                        for vote in votes.iter() {
                            votecount += vote.weight;
                        }
                        if votecount < least {
                            least = votecount;
                        }
                        votecounts.insert(&candidate, votecount);
                    }
                    for (candidate_ref, count) in votecounts.iter() {
                        if *count <= least {
                            let candidate = *candidate_ref;
                            new_loosers.push(candidate.clone());
                        }
                    }
                };

                // If the number of loosers to be removed would result in an underelection, then the loosers become winners.
                let needed_winners = self.num_winners as usize - results.len();
                let available_winners = self.running_total.len() - new_loosers.len();
                if available_winners < needed_winners {
                    for winning_loosers in new_loosers.drain(0..) {
                        results.insert(winning_loosers, rank);
                    }
                    return results;
                }

                // If there's new winners, redistribute their excess vote.
                if new_loosers.len() > 0 {
                    for looser in new_loosers.iter() {
                        let mut loosing_candidate = self.running_total.remove(looser).unwrap();
                        // Redistibute to next choice
                        for vote in loosing_candidate.drain(0..) {
                            if vote.remaining.len() > 0 {
                                let mut remaining = vote.remaining;
                                let next_choice = remaining.remove(0);
                                let weighted_vote = WeightedVote {
                                    weight: vote.weight,
                                    remaining: remaining
                                };
                                if let Some(x) = self.running_total.get_mut(&next_choice) {
                                    x.push(weighted_vote);
                                }
                            }
                        }
                    }
                }
                else {
                    // Nothign else to do, just return what we have
                    return results;
                }
            }
        }
    }

    // Ensure that candidates are in our list of candidates, and return an internal numeric representation of the same
    // TODO: Can we use a trait for this?
    fn mapped_candidates(&mut self, selection: &Vec<T>) -> Vec<usize> {
        let mut mapped = Vec::<usize>::new();
        for selected in selection.iter() {
            if self.candidates.contains_key(&selected) {
                mapped.push(*self.candidates.get(&selected).unwrap());
            }
            else {
                let len = self.candidates.len();
                self.candidates.insert(selected.clone(), len);
            }
        }
        return mapped;
    }

    fn total_votes(&self) -> usize {
        let mut total: usize = 0;

        for (_, candidate_votes) in self.running_total.iter() {
            total += candidate_votes.len();
        }

        return total
    }

    fn droop_threshold(&self) -> usize {
        (self.total_votes() / (self.num_winners as usize+ 1)) + 1
    }

    fn hagenbach_threshold(&self) -> usize {
        self.total_votes() / (self.num_winners as usize+ 1)
    }

    fn hare_threshold(&self) -> usize {
        (self.total_votes() / (self.num_winners as usize))
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stv_test() {
        // Election between Alice, Bob, and Cir
        let mut tally = Tally::new(2, Quota::Droop);
        tally.add(vec!["Alice", "Bob", "Cir"]);
        tally.add(vec!["Alice", "Bob", "Cir"]);
        tally.add(vec!["Alice", "Bob", "Cir"]);

        let result = tally.result();
        assert_eq!(result, indexmap!{"Alice" => 0, "Bob" => 1});
    }

    #[test]
    fn stv_wikipedia_test() {
        // From https://en.wikipedia.org/wiki/Single_transferable_vote#Counting_the_votes
        let mut tally = Tally::new(3, Quota::Droop);
        for _ in 0..4 {
            tally.add(vec!["Orange"]);
        }
        for _ in 0..2 {
            tally.add(vec!["Pear", "Orange"]);
        }
        for _ in 0..8 {
            tally.add(vec!["Chocolate", "Strawberry"]);
        }
        for _ in 0..4 {
            tally.add(vec!["Chocolate", "Sweets"]);
        }
        tally.add(vec!["Strawberry"]);
        tally.add(vec!["Sweets"]);

        let result = tally.result();
        assert_eq!(result, indexmap!{"Chocolate" => 0, "Orange" => 1, "Strawberry" => 2});
    }
}
