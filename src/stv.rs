use std::hash::Hash;
use std::collections::HashMap;

use indexmap::IndexMap;

#[derive(Debug)]
struct WeightedVote<T: Eq + Clone + Hash> {
    weight: f64,
    remaining: Vec<T>,
}

pub enum Quota {

    /// Droop Quota. If you are unsure what to use, use this one.
    Droop,

    /// Hare quota.
    Hare,

    /// Hagenbach-Bischoff quota.
    Hagenbach
}

pub struct Tally<T: Eq + Clone + Hash> {
    running_total: HashMap<T, Vec<WeightedVote<T>>>,
    num_winners: u32,
    quota: Quota,
    candidates: HashMap<T, usize> // Map candiates to a unique integer identifiers.
}

impl<T: Eq + Clone + Hash> Tally<T>  {

    pub fn new(num_winners: u32, quota: Quota) -> Self {
        return Tally {
            running_total: HashMap::new(),
            num_winners: num_winners,
            quota: quota,
            candidates: HashMap::new()
        };
    }

    pub fn add(&mut self, selection: &Vec<T>) {
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

        if let Some(x) = self.running_total.get_mut(&choice) {
            x.push(weighted_vote);
        }
    }

    pub fn result(&mut self) -> IndexMap<T, u32> {
        let threshold = match self.quota {
            Quota::Droop => self.droop_threshold() as f64,
            Quota::Hare => self.hare_threshold() as f64,
            Quota::Hagenbach => self.hagenbach_threshold(),
        };

        let mut results: IndexMap<T, u32> = IndexMap::new();

        let mut rank: u32 = 0;
        loop {
            // If we have less candidates left than there are spots to fill, they are all winners
            if self.running_total.len() <= self.num_winners as usize - results.len() {
                for (candidate, _) in self.running_total.drain() {
                    results.insert(candidate, rank);
                }
                return results;
            }

            // Check if any candidates are over the threshold
            let mut new_winners: Vec<T> = Vec::new();
            for (candidate, votes) in self.running_total.iter() {
                let mut votecount = 0.0;
                for vote in votes.iter() {
                    votecount += vote.weight;
                }
                if votecount >= threshold {
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
                let mut winner_votes: HashMap<T, Vec<WeightedVote<T>>> = HashMap::new();
                for winner in new_winners.drain(0..) {
                    let votes = self.running_total.remove(&winner).unwrap();
                    winner_votes.insert(winner, votes);
                }
                for (winner, mut votes) in winner_votes.drain() {
                    let overvote = votes.len() as f64 - threshold;
                    let weight = overvote / votes.len() as f64;
                    
                    // TODO: feature-gate num-rational instead of f64 for determinism

                    // Redistibute to next choice
                    for vote in votes.drain(0..) {
                        self.redistribute(vote, weight);
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

                    // If there's new loosers, redistribute their excess vote.
                    if new_loosers.len() > 0 {
                    let mut looser_votes: Vec<Vec<WeightedVote<T>>> = Vec::new();
                    for looser in new_loosers.drain(0..) {
                        let votes = self.running_total.remove(&looser).unwrap();
                        looser_votes.push(votes);
                    }
                    for mut votes in looser_votes.drain(0..) {
                        // Redistibute to next choice
                        for vote in votes.drain(0..) {
                            self.redistribute(vote, 1.0);
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

    fn redistribute(&mut self, vote: WeightedVote<T>, weight: f64) {
        if vote.remaining.len() == 0 {
            return;
        }

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
            // Skip to the next choice in line if the preferred next-choice has already won or lost.
            self.redistribute(weighted_vote, 1.0);
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

                // TODO: Fix this to use candidate-ids
                self.running_total.insert(selected.clone(), vec![]);
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

    fn hagenbach_threshold(&self) -> f64 {
        self.total_votes() as f64 / (self.num_winners as f64 + 1.0)
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
        tally.add(&vec!["Alice", "Bob", "Cir"]);
        tally.add(&vec!["Alice", "Bob", "Cir"]);
        tally.add(&vec!["Alice", "Bob", "Cir"]);

        let result = tally.result();
        assert_eq!(result, indexmap!{"Alice" => 0, "Bob" => 1});
    }

    #[test]
    fn stv_wikipedia_test() {
        // From https://en.wikipedia.org/wiki/Single_transferable_vote#Counting_the_votes
        let mut tally = Tally::new(3, Quota::Droop);
        for _ in 0..4 {
            tally.add(&vec!["Orange"]);
        }
        for _ in 0..2 {
            tally.add(&vec!["Pear", "Orange"]);
        }
        for _ in 0..8 {
            tally.add(&vec!["Chocolate", "Strawberry"]);
        }
        for _ in 0..4 {
            tally.add(&vec!["Chocolate", "Sweets"]);
        }
        tally.add(&vec!["Strawberry"]);
        tally.add(&vec!["Sweets"]);

        let result = tally.result();
        assert_eq!(result, indexmap!{"Chocolate" => 0, "Orange" => 1, "Strawberry" => 2});


        // From https://en.wikipedia.org/wiki/Comparison_of_the_Hare_and_Droop_quotas
        let mut hare_tally = Tally::new(5, Quota::Hare);
        let mut droop_tally = Tally::new(5, Quota::Droop);
        for _ in 0..31 {
            hare_tally.add(&vec!["Andrea", "Carter", "Brad"]);
            droop_tally.add(&vec!["Andrea", "Carter", "Brad"]);
        }
        for _ in 0..30 {
            hare_tally.add(&vec!["Carter", "Andrea", "Brad"]);
            droop_tally.add(&vec!["Carter", "Andrea", "Brad"]);
        }
        for _ in 0..2 {
            hare_tally.add(&vec!["Brad", "Andrea", "Carter"]);
            droop_tally.add(&vec!["Brad", "Andrea", "Carter"]);
        }
        for _ in 0..20 {
            hare_tally.add(&vec!["Delilah", "Scott", "Jennifer"]);
            droop_tally.add(&vec!["Delilah", "Scott", "Jennifer"]);
        }
        for _ in 0..20 {
            hare_tally.add(&vec!["Scott", "Delilah", "Jennifer"]);
            droop_tally.add(&vec!["Scott", "Delilah", "Jennifer"]);
        }
        for _ in 0..17 {
            hare_tally.add(&vec!["Jennifer", "Delilah", "Scott"]);
            droop_tally.add(&vec!["Jennifer", "Delilah", "Scott"]);
        }

        let hare_result = hare_tally.result();
        let droop_result = droop_tally.result();

        assert_eq!(hare_result, indexmap!{"Andrea" => 0, "Carter" => 0, "Delilah" => 1, "Scott" => 1, "Jennifer" => 1});
        assert_eq!(droop_result, indexmap!{"Andrea" => 0, "Carter" => 0, "Brad" => 1, "Delilah" => 2, "Scott" => 2});


        // From https://en.wikipedia.org/wiki/Droop_quota
        let mut tally = Tally::new(2, Quota::Droop);
        for _ in 0..45 {
            tally.add(&vec!["Andrea", "Carter"]);
        }
        for _ in 0..25 {
            tally.add(&vec!["Carter"]);
        }
        for _ in 0..30 {
            tally.add(&vec!["Brad"]);
        }
        
        let result = tally.result();
        assert_eq!(result, indexmap!{"Andrea" => 0, "Carter" => 1});


        // From https://en.wikipedia.org/wiki/Hare_quota
        let mut tally = Tally::new(2, Quota::Hare);
        for _ in 0..60 {
            tally.add(&vec!["Andrea", "Carter"]);
        }
        for _ in 0..14 {
            tally.add(&vec!["Carter"]);
        }
        for _ in 0..30 {
            tally.add(&vec!["Brad", "Andrea"]);
        }
        
        let result = tally.result();
        assert_eq!(result, indexmap!{"Andrea" => 0, "Brad" => 1});


        // From https://en.wikipedia.org/wiki/Hagenbach-Bischoff_quota
        let mut tally = Tally::new(2, Quota::Hagenbach);
        for _ in 0..45 {
            tally.add(&vec!["Andrea", "Carter"]);
        }
        for _ in 0..25 {
            tally.add(&vec!["Carter"]);
        }
        for _ in 0..30 {
            tally.add(&vec!["Brad"]);
        }
        
        let result = tally.result();
        assert_eq!(result, indexmap!{"Andrea" => 0, "Carter" => 1});


        // From https://en.wikipedia.org/wiki/Hagenbach-Bischoff_quota
        let mut hagen_tally = Tally::new(7, Quota::Hagenbach);
        let mut droop_tally = Tally::new(7, Quota::Droop);
        for _ in 0..14 {
            hagen_tally.add(&vec!["Andrea", "Carter", "Brad", "Delilah"]);
            droop_tally.add(&vec!["Andrea", "Carter", "Brad", "Delilah"]);
        }
        for _ in 0..14 {
            hagen_tally.add(&vec!["Cater", "Andrea", "Brad", "Delilah"]);
            droop_tally.add(&vec!["Cater", "Andrea", "Brad", "Delilah"]);
        }
        for _ in 0..14 {
            hagen_tally.add(&vec!["Brad", "Andrea", "Cater", "Delilah"]);
            droop_tally.add(&vec!["Brad", "Andrea", "Cater", "Delilah"]);
        }
        for _ in 0..11 {
            hagen_tally.add(&vec!["Delilah", "Andrea", "Cater", "Brad"]);
            droop_tally.add(&vec!["Delilah", "Andrea", "Cater", "Brad"]);
        }
        for _ in 0..13 {
            hagen_tally.add(&vec!["Scott", "Jennifer", "Matt", "Susan"]);
            droop_tally.add(&vec!["Scott", "Jennifer", "Matt", "Susan"]);
        }
        for _ in 0..13 {
            hagen_tally.add(&vec!["Jennifer", "Scott", "Matt", "Susan"]);
            droop_tally.add(&vec!["Jennifer", "Scott", "Matt", "Susan"]);
        }
        for _ in 0..13 {
            hagen_tally.add(&vec!["Matt", "Scott", "Jennifer", "Susan"]);
            droop_tally.add(&vec!["Matt", "Scott", "Jennifer", "Susan"]);
        }
        for _ in 0..12 {
            hagen_tally.add(&vec!["Susan", "Scott", "Jennifer", "Matt"]);
            droop_tally.add(&vec!["Susan", "Scott", "Jennifer", "Matt"]);
        }

        let _hagen_result = hagen_tally.result();
        let _droop_result = droop_tally.result();

        // TODO: Return hashmap??

        //assert_eq!(hagen_result, indexmap!{"Andrea" => 0, "Carter" => 0, "Brad" => 0, "Jennifer" => 0, "Scott" => 0, "Matt" => 0, "Delilah" => 1});
        //assert_eq!(droop_result, indexmap!{"Andrea" => 0, "Carter" => 0, "Brad" => 0, "Scott" => 1, "Jennifer" => 1, "Matt" => 1, "Susan" => 1});

        // From https://en.wikipedia.org/wiki/Hagenbach-Bischoff_quota
        let mut tally = Tally::new(2, Quota::Hagenbach);
        for _ in 0..50 {
            tally.add(&vec!["Andrea", "Brad"]);
        }
        for _ in 0..150 {
            tally.add(&vec!["Andrea", "Carter"]);
        }
        for _ in 0..75 {
            tally.add(&vec!["Brad", "Carter"]);
        }
        for _ in 0..25 {
            tally.add(&vec!["Carter", "Brad"]);
        }

        let results = tally.result();
        assert_eq!(results, indexmap!{"Andrea" => 0, "Brad" => 1, "Carter" => 1});
    }
}
