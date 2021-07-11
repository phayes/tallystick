use super::Numeric;
use super::RankedCandidate;
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
            transfer: transfer,
        }
    }

    /// Create a new `irv::Tally` with the provided candidates
    pub fn with_candidates(transfer: Transfer, candidates: Vec<T>) -> Self {
        Tally {
            running_total: VoteTree::with_candidates(candidates),
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

    pub fn tally_ranked(&self) -> Vec<RankedCandidate<T>> {
        let max = C::max_value();

        let candidates = self.running_total.candidates();
        let mut inverse_ranked = Vec::<RankedCandidate<T>>::with_capacity(candidates.len());
        let mut inverse_rank = 0;
        let mut eliminated = HashSet::new();

        loop {
            // First Eagerly assign tally passing through eliminated candidates
            let (_excess, mut score) = self.running_total.assign_votes(&eliminated);

            // Scores lack elements for which there is not direct vote; we have to supply
            // for elimination to verbosely mention them.
            // Otherwise, some candidate may appear on one round after it should be eliminated.
            for c in candidates.iter() {
                if !eliminated.contains(c) {
                    score.entry(c.clone()).or_insert(C::zero());
                }
            }

            // If there are no more valid candidates, return early
            if score.is_empty() {
                return inverse_ranked;
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
                    inverse_ranked.push(RankedCandidate {
                        candidate: cand,
                        rank: inverse_rank,
                    });
                }
                break;
            }

            // Calculate the worst performing candidates and add them to the reverse ranking
            let min_score = score.values().min().unwrap_or(&max);
            let mut loosers = Vec::<T>::new();
            for (cand, count) in score.iter() {
                if count == min_score {
                    loosers.push(cand.clone());
                }
            }
            if loosers.is_empty() {
                // No one left
                break;
            }

            // Remove all loosers
            for looser in loosers.drain(..) {
                inverse_ranked.push(RankedCandidate {
                    candidate: looser.clone(),
                    rank: inverse_rank,
                });
                eliminated.insert(looser);
            }

            inverse_rank += 1;
        }

        let num_ranked = inverse_ranked.len();
        let mut ranked = Vec::<RankedCandidate<T>>::with_capacity(num_ranked);
        for inversed in inverse_ranked.drain(..).rev() {
            ranked.push(RankedCandidate {
                candidate: inversed.candidate,
                rank: num_ranked - inversed.rank - 1,
            });
        }
        ranked
    }

    pub fn tally_winners(&self) -> RankedWinners<T> {
        RankedWinners::from_ranked(self.tally_ranked(), 1)
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
        let winners = tally.tally_winners();
        assert!(winners.contains(&"Knoxville"));

        // Verify ranking
        let ranked = tally.tally_ranked();
        assert_eq!(ranked[0].candidate, "Knoxville");
        assert_eq!(ranked[0].rank, 0);
        assert_eq!(ranked[1].candidate, "Memphis");
        assert_eq!(ranked[1].rank, 1);
        assert_eq!(ranked[2].candidate, "Nashville");
        assert_eq!(ranked[2].rank, 2);
        assert_eq!(ranked[3].candidate, "Chattanooga");
        assert_eq!(ranked[3].rank, 3);

        Ok(())
    }
}
