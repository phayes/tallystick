use std::hash::Hash;
use std::ops::AddAssign;
use hashbrown::HashMap;
use num_traits::cast::NumCast;
use num_traits::Num;

use super::RankedWinners;
use indexmap::IndexMap;

#[derive(Debug)]
struct WeightedVote<T, C>
    where T: Eq + Clone + Hash,                            // Candidate
          C: Copy + PartialOrd + AddAssign + Num + NumCast // vote count type
{
    weight: C,
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

pub type DefaultTally<T> = Tally<T, f64>;

pub struct Tally<T, C>
    where T: Eq + Clone + Hash,                            // Candidate
          C: Copy + PartialOrd + AddAssign + Num + NumCast // vote count type
{
    running_total: HashMap<T, Vec<WeightedVote<T, C>>>,
    num_winners: u32,
    quota: Quota
}

impl<T, C> Tally<T, C> 
    where T: Eq + Clone + Hash,                            // Candidate
          C: Copy + PartialOrd + AddAssign + Num + NumCast // vote count type
{

    pub fn new(num_winners: u32, quota: Quota) -> Self {
        return Tally {
            running_total: HashMap::new(),
            num_winners: num_winners,
            quota: quota
        };
    }

    pub fn add(&mut self, mut selection: Vec<T>) {
        if selection.is_empty() {
            return;
        }

        let choice = selection.remove(0);

        // Ensure that the running total contains all candidates
        for candidate in selection.iter() {
            if !self.running_total.contains_key(candidate) {
                self.running_total.insert(candidate.clone(), vec![]);
            }
        }

        let weighted_vote = WeightedVote {
            weight: C::one(),
            remaining: selection
        };

        self.running_total.entry(choice).or_default().push(weighted_vote);
    }

    pub fn add_ref(&mut self, selection: &Vec<T>) {
        // Regretably, we need to store the entire selection, so just clone it
        self.add(selection.clone());
    }

    pub fn winners(&mut self) -> RankedWinners<T> {
        let threshold = match self.quota {
            Quota::Droop => self.droop_threshold(),
            Quota::Hare => self.hare_threshold(),
            Quota::Hagenbach => self.hagenbach_threshold(),
        };

        let mut winners = RankedWinners::new(self.num_winners);

        let mut rank: u32 = 0;
        loop {
            // Step 1. If we have less candidates left than there are spots to fill, they are all winners
            if self.running_total.len() <= self.num_winners as usize - winners.len() {
                for (candidate, _) in self.running_total.drain() {
                    winners.push(candidate, rank);
                }
                return winners;
            }

            // Step 2. Check if any candidates are over the threshold
            let mut new_winners: Vec<T> = Vec::new();
            for (candidate, votes) in self.running_total.iter() {
                let mut votecount = C::zero();
                for vote in votes.iter() {
                    votecount += vote.weight;
                }
                if votecount >= threshold {
                    new_winners.push(candidate.clone());
                }
            }

            // Step 3. If we have enough winners, end the tally and return results.
            if (winners.len() + new_winners.len()) as u32 >= self.num_winners {
                for winner in new_winners.drain(0..) {
                    winners.push(winner, rank);
                }
                return winners;
            }

            // Step 4. If there's new winners, redistribute their excess vote.
            if !new_winners.is_empty() {
                let mut winner_votes: HashMap<T, Vec<WeightedVote<T, C>>> = HashMap::new();
                for winner in new_winners.drain(0..) {
                    let votes = self.running_total.remove(&winner).unwrap();
                    winner_votes.insert(winner, votes);
                }
                for (winner, mut votes) in winner_votes.drain() {
                    let overvote = C::from(votes.len()).unwrap() - threshold;
                    let weight = overvote / C::from(votes.len()).unwrap();
                    
                    // Redistibute to next choice
                    for vote in votes.drain(0..) {
                        self.redistribute(vote, weight);
                    }

                    winners.push(winner, rank);
                }

                // If we have enough winners, return it.
                if winners.len() as u32 >= self.num_winners {
                    return winners;
                }

                // We've added winners, so increase the rank and continue to the next round.
                rank += 1;
                continue;
            }
            else {
                // Remove loosers and redistribute
                let mut new_loosers: Vec<T> = Vec::new();
                {
                    let mut votecounts: HashMap<&T, C> = HashMap::new();
                    let mut first = true;
                    let mut least = C::zero();
                    for (candidate, votes) in self.running_total.iter() {
                        let mut votecount = C::zero();

                        for vote in votes.iter() {
                            votecount += vote.weight;
                        }
                        if first {
                            least = votecount;
                            first = false;
                        }
                        else if votecount < least {
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
                let needed_winners = self.num_winners as usize - winners.len();
                let available_winners = self.running_total.len() - new_loosers.len();
                if available_winners < needed_winners {
                    for winning_loosers in new_loosers.drain(0..) {
                        winners.push(winning_loosers, rank);
                    }
                    return winners;
                }

                // If there's new loosers, redistribute their excess vote.
                if !new_loosers.is_empty() {
                    let mut looser_votes: Vec<Vec<WeightedVote<T, C>>> = Vec::new();
                    for looser in new_loosers.drain(0..) {
                        let votes = self.running_total.remove(&looser).unwrap();
                        looser_votes.push(votes);
                    }
                    for mut votes in looser_votes.drain(0..) {
                        // Redistibute to next choice
                        for vote in votes.drain(0..) {
                            self.redistribute(vote, C::one());
                        }
                    }
                }
                else {
                    unreachable!();
                }
            }
        }
    }

    fn redistribute(&mut self, vote: WeightedVote<T, C>, weight: C) {
        if vote.remaining.is_empty() {
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
            self.redistribute(weighted_vote, C::one());
        }
    }

    fn total_votes(&self) -> usize {
        let mut total: usize = 0;

        for (_, candidate_votes) in self.running_total.iter() {
            total += candidate_votes.len();
        }

        return total
    }

    fn droop_threshold(&self) -> C {
        let total_votes = C::from(self.total_votes()).unwrap();
        let demon =  C::from(self.num_winners).unwrap() + C::one();
        return (total_votes / demon) + C::one();
    }

    fn hagenbach_threshold(&self) -> C {
        let total_votes = C::from(self.total_votes()).unwrap();
        let demon =  C::from(self.num_winners).unwrap() + C::one();
        return total_votes / demon;
    }

    fn hare_threshold(&self) -> C {
        let total_votes = C::from(self.total_votes()).unwrap();
        let demon =  C::from(self.num_winners).unwrap();
        return total_votes / demon;
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stv_test() {
        // Election between Alice, Bob, and Cir
        let mut tally = DefaultTally::new(2, Quota::Droop);
        tally.add(vec!["Alice", "Bob", "Cir"]);
        tally.add(vec!["Alice", "Bob", "Cir"]);
        tally.add(vec!["Alice", "Bob", "Cir"]);

        let winners = tally.winners();
        assert_eq!(winners.into_vec(), vec!{("Alice", 0), ("Bob", 1)});
    }

    #[test]
    fn stv_wikipedia_test() -> Result<(), ()> {
        // From https://en.wikipedia.org/wiki/Single_transferable_vote#Counting_the_votes
        let mut tally = DefaultTally::new(3, Quota::Droop);
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

        let winners = tally.winners();
        assert_eq!(winners.into_vec(), vec!{("Chocolate", 0), ("Orange", 1), ("Strawberry", 2)});


        // From https://en.wikipedia.org/wiki/Comparison_of_the_Hare_and_Droop_quotas
        let mut hare_tally = DefaultTally::new(5, Quota::Hare);
        let mut droop_tally = DefaultTally::new(5, Quota::Droop);
        for _ in 0..31 {
            hare_tally.add(vec!["Andrea", "Carter", "Brad"]);
            droop_tally.add(vec!["Andrea", "Carter", "Brad"]);
        }
        for _ in 0..30 {
            hare_tally.add(vec!["Carter", "Andrea", "Brad"]);
            droop_tally.add(vec!["Carter", "Andrea", "Brad"]);
        }
        for _ in 0..2 {
            hare_tally.add(vec!["Brad", "Andrea", "Carter"]);
            droop_tally.add(vec!["Brad", "Andrea", "Carter"]);
        }
        for _ in 0..20 {
            hare_tally.add(vec!["Delilah", "Scott", "Jennifer"]);
            droop_tally.add(vec!["Delilah", "Scott", "Jennifer"]);
        }
        for _ in 0..20 {
            hare_tally.add(vec!["Scott", "Delilah", "Jennifer"]);
            droop_tally.add(vec!["Scott", "Delilah", "Jennifer"]);
        }
        for _ in 0..17 {
            hare_tally.add(vec!["Jennifer", "Delilah", "Scott"]);
            droop_tally.add(vec!["Jennifer", "Delilah", "Scott"]);
        }

        let hare_winners = hare_tally.winners();
        assert_eq!(hare_winners.len(), 5);
        assert_eq!(hare_winners.rank(&"Andrea").unwrap(), 0);
        assert_eq!(hare_winners.rank(&"Carter").unwrap(), 0);
        assert_eq!(hare_winners.rank(&"Delilah").unwrap(), 1);
        assert_eq!(hare_winners.rank(&"Scott").unwrap(), 1);
        assert_eq!(hare_winners.rank(&"Jennifer").unwrap(), 1);

        let droop_winners = droop_tally.winners();
        assert_eq!(droop_winners.len(), 5);
        assert_eq!(droop_winners.rank(&"Andrea").unwrap(), 0);
        assert_eq!(droop_winners.rank(&"Carter").unwrap(), 0);
        assert_eq!(droop_winners.rank(&"Brad").unwrap(), 1);
        assert_eq!(droop_winners.rank(&"Delilah").unwrap(), 2);
        assert_eq!(droop_winners.rank(&"Scott").unwrap(), 2);


        // From https://en.wikipedia.org/wiki/Droop_quota
        let mut tally = DefaultTally::new(2, Quota::Droop);
        for _ in 0..45 {
            tally.add(vec!["Andrea", "Carter"]);
        }
        for _ in 0..25 {
            tally.add(vec!["Carter"]);
        }
        for _ in 0..30 {
            tally.add(vec!["Brad"]);
        }
        
        let winners = tally.winners();
        assert_eq!(winners.into_vec(), vec!{("Andrea", 0), ("Carter", 1)});

        // From https://en.wikipedia.org/wiki/Hare_quota
        let mut tally = DefaultTally::new(2, Quota::Hare);
        for _ in 0..60 {
            tally.add(vec!["Andrea", "Carter"]);
        }
        for _ in 0..14 {
            tally.add(vec!["Carter"]);
        }
        for _ in 0..30 {
            tally.add(vec!["Brad", "Andrea"]);
        }
        
        let winners = tally.winners();
        assert_eq!(winners.into_vec(), vec!{("Andrea", 0), ("Brad", 1)});


        // From https://en.wikipedia.org/wiki/Hagenbach-Bischoff_quota
        let mut tally = DefaultTally::new(2, Quota::Hagenbach);
        for _ in 0..45 {
            tally.add(vec!["Andrea", "Carter"]);
        }
        for _ in 0..25 {
            tally.add(vec!["Carter"]);
        }
        for _ in 0..30 {
            tally.add(vec!["Brad"]);
        }
        
        let winners = tally.winners();
        assert_eq!(winners.into_vec(), vec!{("Andrea", 0), ("Carter", 1)});


        // From https://en.wikipedia.org/wiki/Hagenbach-Bischoff_quota
        let mut hagen_tally = DefaultTally::new(7, Quota::Hagenbach);
        let mut droop_tally = DefaultTally::new(7, Quota::Droop);
        for _ in 0..14 {
            hagen_tally.add(vec!["Andrea", "Carter", "Brad", "Delilah"]);
            droop_tally.add(vec!["Andrea", "Carter", "Brad", "Delilah"]);
        }
        for _ in 0..14 {
            hagen_tally.add(vec!["Carter", "Andrea", "Brad", "Delilah"]);
            droop_tally.add(vec!["Carter", "Andrea", "Brad", "Delilah"]);
        }
        for _ in 0..14 {
            hagen_tally.add(vec!["Brad", "Andrea", "Carter", "Delilah"]);
            droop_tally.add(vec!["Brad", "Andrea", "Carter", "Delilah"]);
        }
        for _ in 0..11 {
            hagen_tally.add(vec!["Delilah", "Andrea", "Carter", "Brad"]);
            droop_tally.add(vec!["Delilah", "Andrea", "Carter", "Brad"]);
        }
        for _ in 0..13 {
            hagen_tally.add(vec!["Scott", "Jennifer", "Matt", "Susan"]);
            droop_tally.add(vec!["Scott", "Jennifer", "Matt", "Susan"]);
        }
        for _ in 0..13 {
            hagen_tally.add(vec!["Jennifer", "Scott", "Matt", "Susan"]);
            droop_tally.add(vec!["Jennifer", "Scott", "Matt", "Susan"]);
        }
        for _ in 0..13 {
            hagen_tally.add(vec!["Matt", "Scott", "Jennifer", "Susan"]);
            droop_tally.add(vec!["Matt", "Scott", "Jennifer", "Susan"]);
        }
        for _ in 0..12 {
            hagen_tally.add(vec!["Susan", "Scott", "Jennifer", "Matt"]);
            droop_tally.add(vec!["Susan", "Scott", "Jennifer", "Matt"]);
        }

        
        let hagen_winners = hagen_tally.winners();
        assert_eq!(hagen_winners.len(), 7);
        assert_eq!(hagen_winners.rank(&"Andrea").unwrap(), 0);
        assert_eq!(hagen_winners.rank(&"Carter").unwrap(), 0);
        assert_eq!(hagen_winners.rank(&"Brad").unwrap(), 0);
        assert_eq!(hagen_winners.rank(&"Jennifer").unwrap(), 0);
        assert_eq!(hagen_winners.rank(&"Scott").unwrap(), 0);
        assert_eq!(hagen_winners.rank(&"Matt").unwrap(), 0);
        assert_eq!(hagen_winners.rank(&"Delilah").unwrap(), 1);

        let droop_winners = droop_tally.winners();
        assert_eq!(droop_winners.len(), 7);
        assert_eq!(droop_winners.rank(&"Andrea").unwrap(), 0);
        assert_eq!(droop_winners.rank(&"Carter").unwrap(), 0);
        assert_eq!(droop_winners.rank(&"Brad").unwrap(), 0);
        assert_eq!(droop_winners.rank(&"Scott").unwrap(), 1);
        assert_eq!(droop_winners.rank(&"Jennifer").unwrap(), 1);
        assert_eq!(droop_winners.rank(&"Matt").unwrap(), 1);
        assert_eq!(droop_winners.rank(&"Susan").unwrap(), 1);


        // From https://en.wikipedia.org/wiki/Hagenbach-Bischoff_quota
        let mut tally = DefaultTally::new(2, Quota::Hagenbach);
        for _ in 0..50 {
            tally.add(vec!["Andrea", "Brad"]);
        }
        for _ in 0..150 {
            tally.add(vec!["Andrea", "Carter"]);
        }
        for _ in 0..75 {
            tally.add(vec!["Brad", "Carter"]);
        }
        for _ in 0..25 {
            tally.add(vec!["Carter", "Brad"]);
        }

        let winners = tally.winners();
        assert_eq!(winners.len(), 3);
        assert_eq!(winners.rank(&"Andrea").unwrap(), 0);
        assert_eq!(winners.rank(&"Brad").unwrap(), 1);
        assert_eq!(winners.rank(&"Carter").unwrap(), 1);
        assert_eq!(winners.check_overflow(), true);

        Ok(())
    }
}
