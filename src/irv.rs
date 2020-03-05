use super::Numeric;
use super::RankedWinners;
use crate::Transfer;
use crate::VoteTree;
use hashbrown::HashSet;
use num_traits::cast::NumCast;
use num_traits::Num;
use std::cmp::Ord;
use std::hash::Hash;
use std::ops::AddAssign;

#[derive(Debug)]
struct WeightedVote<T, C>
where
    T: Eq + Clone + Hash,                                       // Candidate
    C: Copy + PartialOrd + AddAssign + Num + NumCast + Numeric, // vote count type
{
    weight: C,
    remaining: Vec<T>,
}

pub type DefaultTally<T> = Tally<T, u64>;

pub struct Tally<T, C>
where
    T: Eq + Clone + Hash,                                       // Candidate
    C: Copy + PartialOrd + AddAssign + Num + NumCast + Numeric, // vote count type
{
    running_total: VoteTree<T, C>,
    expected_votes: Option<usize>, // Expected votes *per candidate*.
    transfer: Transfer,
}

impl<T, C> Tally<T, C>
where
    T: Eq + Clone + Hash,                                             // Candidate
    C: Copy + PartialOrd + Ord + AddAssign + Num + NumCast + Numeric, // vote count type
{
    pub fn new(transfer: Transfer) -> Self {
        Tally {
            running_total: VoteTree::new(),
            expected_votes: None,
            transfer: transfer,
        }
    }

    pub fn with_capacity(transfer: Transfer, expected_candidates: usize, expected_votes: usize) -> Self {
        Tally {
            // TODO: VoteTree::with_capacity()
            running_total: VoteTree::new(),
            expected_votes: Some((expected_votes / expected_candidates) * 2),
            transfer: transfer,
        }
    }

    pub fn add_weighted(&mut self, selection: Vec<T>, weight: C) {
        self.running_total.add(&selection, weight);
    }

    pub fn add(&mut self, selection: Vec<T>) {
        // TODO: split add and add_ref in VoteTree
        self.running_total.add(&selection, C::one());
    }

    pub fn add_ref(&mut self, selection: &[T]) {
        // TODO: split add and add_ref in VoteTree
        self.running_total.add(selection, C::one());
    }

    pub fn winners(&mut self) -> RankedWinners<T> {
        let zero = C::zero();
        let two = C::one() + C::one();
        let max = C::max_value();

        let candidates = self.running_total.candidates();
        let mut eliminated = HashSet::new();

        let mut winners = RankedWinners::new(1);

        loop {
            // First Eagerly assign tally passing through eliminated candidates
            let (excess, mut score) = self.running_total.assign_votes(&eliminated);

            //Scores lack elements for which there is not direct vote; we have to supply
            //for elimination to verbosely mention them.
            //Otherwise, some candidate may appear on one round after it should be eliminated.
            for c in candidates.iter() {
                if !eliminated.contains(c) {
                    score.entry(c.clone()).or_insert(C::zero());
                }
            }

            // If there are no more valid candidates, return early
            if score.is_empty() {
                return winners;
            }

            // Check for case where all remaining candidates are tied
            let mut all_counts = Vec::new();
            let mut all_tied = true;
            for s in score.iter() {
                for check_count in all_counts.iter() {
                    if s.1 != check_count {
                        all_tied = false;
                        break;
                    }
                }
                all_counts.push(s.1.clone());
            }
            if all_tied {
                for (cand, _) in score {
                    winners.push(cand, 0);
                }
                return winners;
            }

            // Possible winner must have best score which is better than quota
            // TODO: Is this `two` correct?
            let quota = (self.running_total.count - excess) / two;
            let max_score = score.values().max().unwrap_or(&zero);
            for (cand, count) in score.iter() {
                if count == max_score && *count > quota {
                    winners.push(cand.clone(), 0);
                }
            }

            if !winners.is_empty() {
                return winners;
            } else {
                // We need another round; to this end, we need to select a set of
                // candidates with a minimal score, and eliminate one of them
                let min_score = score.values().min().unwrap_or(&max);
                let mut loosers = Vec::<T>::new();
                for (cand, count) in score.iter() {
                    if count == min_score {
                        loosers.push(cand.clone());
                    }
                }
                if loosers.is_empty() {
                    // No winner
                    return winners;
                }

                // Remove all loosers
                for looser in loosers.drain(..) {
                    eliminated.insert(looser);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::TallyError;

    #[test]
    fn irv_wikipedia_1() -> Result<(), TallyError> {
        // See: https://en.wikipedia.org/wiki/Instant-runoff_voting

        let mut tally = DefaultTally::new(Transfer::Meek);
        tally.add_weighted(vec!["Memphis", "Nashville", "Chattanooga", "Chattanooga"], 42);
        tally.add_weighted(vec!["Nashville", "Chattanooga", "Knoxville", "Memphis"], 26);
        tally.add_weighted(vec!["Chattanooga", "Knoxville", "Nashville", "Nashville"], 15);
        tally.add_weighted(vec!["Knoxville", "Chattanooga", "Nashville", "Memphis"], 17);

        // Verify winners
        let winners = tally.winners();
        assert!(winners.contains(&"Knoxville"));

        // TODO: RANKED via reverse elimination

        Ok(())
    }
}
