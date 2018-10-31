use std::hash::Hash;
use std::collections::HashMap;

#[derive(Debug)]
struct WeightedVote<T: Eq + Clone + Hash + std::fmt::Debug> {
    weight: f64,
    remaining: Vec<T>,
}

pub struct STVTally<T: Eq + Clone + Hash + std::fmt::Debug> {
    running_total: HashMap<T, Vec<WeightedVote<T>>>,
    num_winners: u32,
}

impl<T: Eq + Clone + Hash + std::fmt::Debug> STVTally<T>  {

    pub fn new(num_winners: u32) -> Self {
        return STVTally {
            running_total: HashMap::new(),
            num_winners: num_winners,
        };
    }

    pub fn add(&mut self, selection: Vec<T>) {
        if selection.is_empty() {
            return;
        }

        // TODO: Use pointers into a master-key Vec<T> (clone only once!).
        
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

    pub fn result(&mut self) -> Vec<T> {
        let threshold = self.droop_threshold();

        let mut results: Vec<T> = Vec::new();

        loop {
            // Check if any candidates are over the threshold
            let mut new_winners: Vec<T> = Vec::new();
            for (candidate, votes) in self.running_total.iter() {
                if votes.len() >= threshold { // TODO count with weight
                    new_winners.push(candidate.clone());
                }
            }

            // If we have enough winners, end the tally and return results.
            if (results.len() + new_winners.len()) as u32 > self.num_winners {
                results.append(&mut new_winners);
                return results;
            }
            // If there's new winners, redistribute their excess vote.
            if new_winners.len() > 0 {
                for winner in new_winners.drain(0..) {
                    let mut votes = self.running_total.remove(&winner).unwrap();
                    let overvote = votes.len() - threshold;
                    let weight = overvote as f64 / threshold as f64;
                    
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
                    results.push(winner);
                }

                // If we have enough results, return it.
                if results.len() as u32 >= self.num_winners {
                    return results;
                }
                continue;
            }
            else {
                // Remove loosers and redistribute
                let mut new_loosers: Vec<T> = Vec::new();
                {
                    let mut votecounts: HashMap<&T, f64> = HashMap::new();
                    let mut least: f64 = 0.0;
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
                }

                // TODO: What to do if, once we subtract the loosers, we don't have enough candidates left?

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
        let mut tally = STVTally::new(2);
        tally.add(vec!["Alice", "Bob", "Cir"]);
        tally.add(vec!["Alice", "Bob", "Cir"]);
        tally.add(vec!["Alice", "Bob", "Cir"]);

        let result = tally.result();
        assert_eq!(result, vec!["Alice", "Bob"]);
    }
}
