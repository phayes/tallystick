//! VoteTree data structure for storing transferable preferential votes
//! This data structure is taken from https://gitlab.com/mbq/wybr,
//! with a special thanks to mbq.

use hashbrown::HashMap;
use hashbrown::HashSet;
use num_traits::cast::NumCast;
use num_traits::Num;
use std::cmp::Ord;
use std::hash::Hash;
use std::ops::AddAssign;

#[derive(Copy, Clone)]
///Vote transferring type
pub enum Transfer {
    ///Meek transfer; each rank gets given candidate's weight times the remaining vote part
    Meek,
    ///Warren transfer; each ranks gets given candidate's weight or the whole remaining vote,
    ///whatever is smaller
    Warren,
}

pub(crate) struct VoteTree<T, C = u64>
where
    T: Eq + Clone + Hash,                                   // Candidate type
    C: Copy + PartialOrd + AddAssign + Num + NumCast + Ord, // Count type
{
    pub(crate) count: C,
    children: HashMap<T, VoteTree<T, C>>,
}

impl<T, C> VoteTree<T, C>
where
    T: Eq + Clone + Hash,                                   // Candidate type
    C: Copy + PartialOrd + AddAssign + Num + NumCast + Ord, // Count type
{
    pub(crate) fn new() -> VoteTree<T, C> {
        VoteTree {
            count: C::zero(),
            children: HashMap::new(),
        }
    }

    pub(crate) fn add(&mut self, vote: &[T], count: C) -> C {
        self.count += count;
        if vote.is_empty() {
            self.count
        } else {
            self.children
                // TODO: remove this clone - must use raw_entry
                .entry(vote[0].clone())
                .or_insert(VoteTree {
                    count: C::zero(),
                    children: HashMap::new(),
                })
                .add(&vote[1..], count)
        }
    }

    pub(crate) fn distribute_votes(&self, scores: &mut HashMap<T, C>, eliminated: &HashSet<T>) -> C {
        let mut assigned = C::zero();
        for (cand, deeper) in &self.children {
            if !eliminated.contains(&cand) {
                // TODO: Remove this clone
                *scores.entry(cand.clone()).or_insert(C::zero()) += deeper.count;
                assigned += deeper.count;
            } else {
                assigned += deeper.distribute_votes(scores, eliminated);
            }
        }
        assigned
    }

    pub(crate) fn transfer_votes(&self, scores: &mut HashMap<T, C>, weights: &HashMap<T, C>, vote: &C, base: &C, transfer: Transfer) -> C {
        use std::cmp::min;

        let zero = C::zero();

        let mut assigned = C::zero();
        for (c, deeper) in &self.children {
            let given = match transfer {
                //c gets its weight * remaining part of vote
                Transfer::Meek => (*vote * *weights.get(c).unwrap_or(&zero)) / *base,
                //c gets its weight or the remaining vote, whatever is smaller
                Transfer::Warren => min(*vote, *weights.get(&c).unwrap_or(&C::zero())),
            };
            if given > C::zero() {
                // TODO: remove this clone
                *scores.entry(c.clone()).or_insert(C::zero()) += deeper.count * given;
                assigned += deeper.count * given;
            }
            if given < *vote {
                let remaining = *vote - given;
                assigned += deeper.transfer_votes(scores, weights, &remaining, base, transfer);
            }
        }
        assigned
    }

    pub(crate) fn count_ranks(&self, points: &mut HashMap<(T, usize), C>, skipped: &HashSet<T>, depth: usize) {
        for (c, deeper) in &self.children {
            if !skipped.contains(&c) {
                // TODO: remove this clone using lifetimes
                *points.entry((c.clone(), depth)).or_insert(C::zero()) += deeper.count;
                deeper.count_ranks(points, skipped, depth + 1);
            } else {
                //Skip, hence go deeper without increasing depth
                deeper.count_ranks(points, skipped, depth);
            }
        }
    }

    pub(crate) fn transfer_votes_fp(&self, weights: &HashMap<T, C>, base: &C, transfer: Transfer) -> (C, HashMap<T, C>) {
        let mut scores = HashMap::new();
        let total = self.count * *base;
        let excess = total - self.transfer_votes(&mut scores, weights, base, base, transfer);
        (excess, scores)
    }

    #[cfg(test)]
    fn first_vote_count(&self, candidate: &T) -> C {
        if let Some(b) = self.children.get(candidate) {
            b.count
        } else {
            C::zero()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl From<Vec<(u64, Vec<u32>)>> for VoteTree<u32, u64> {
        fn from(vec: Vec<(u64, Vec<u32>)>) -> Self {
            let mut tree = VoteTree::new();
            for v in vec {
                tree.add(&v.1, v.0);
            }
            tree
        }
    }

    #[test]
    fn stv_branch_test() {
        let mut x = VoteTree::new();
        x.add(&[3, 5, 1, 7, 2], 4);
        let n = x.add(&[3, 5, 1, 7, 2], 3);
        assert_eq!(n, 7);
        assert_eq!(x.add(&[], 0), 7);
        assert_eq!(x.add(&[3], 0), 7);
        assert_eq!(x.add(&[4], 0), 0);
    }
    #[test]
    fn transfer_votes() {
        let x = VoteTree::from(vec![
            (1, vec![0, 1]),
            (2, vec![0, 2]),
            (3, vec![1, 0]),
            (4, vec![1, 2]),
            (5, vec![2, 0]),
            (6, vec![2, 1]),
        ]);

        let base = 1_000_000;

        for &transfer in [Transfer::Meek, Transfer::Warren].iter() {
            let weights_one: HashMap<u32, u64> = (0..3).map(|i| (i as u32, base)).collect();
            let (excess, score) = x.transfer_votes_fp(&weights_one, &base, transfer);
            assert_eq!(excess, 0);
            for i in 0..3 {
                assert_eq!(score[&i], x.first_vote_count(&i) * base);
            }
        }

        let weights_half: HashMap<u32, u64> = [(0, base / 2), (1, base / 2), (2, base / 2)]
            .iter()
            .map(|(a, b)| (*a as u32, *b as u64))
            .collect();
        //Meek
        let (excess, score) = x.transfer_votes_fp(&weights_half, &base, Transfer::Meek);
        //Will be perfectly accurate provided that base is divisible by 4
        assert_eq!(score[&0], 3 * base / 2 + (3 + 5) * base / 4);
        assert_eq!(score[&1], 7 * base / 2 + (1 + 6) * base / 4);
        assert_eq!(score[&2], 11 * base / 2 + (4 + 2) * base / 4);
        assert_eq!(excess, (base * (1 + 2 + 3 + 4 + 5 + 6)) / 4);
        assert_eq!(score.iter().map(|(_, v)| *v).sum::<u64>() + excess, x.count * base);

        //Warren
        let (excess, score) = x.transfer_votes_fp(&weights_half, &base, Transfer::Warren);
        //Will be perfectly accurate provided that base is divisible by 2
        assert_eq!(score[&0], 3 * base / 2 + (3 + 5) * base / 2);
        assert_eq!(score[&1], 7 * base / 2 + (1 + 6) * base / 2);
        assert_eq!(score[&2], 11 * base / 2 + (4 + 2) * base / 2);
        assert_eq!(excess, 0);
        assert_eq!(score.iter().map(|(_, v)| *v).sum::<u64>() + excess, x.count * base);
    }

    #[test]
    fn transfer_zero_weight() {
        let x = VoteTree::from(vec![
            (3, vec![0, 2, 3]),
            (4, vec![0, 2, 1]),
            (2, vec![3, 0, 2]),
            (1, vec![1]),
            (2, vec![1, 3, 2, 0]),
            (1, vec![2, 3, 1]),
        ]);
        let base = 1_000_000;

        let vsum = x.count;

        let mut w: HashMap<u32, u64> = (0..4).map(|i| (i as u32, base)).collect();
        w.remove(&1);
        for &transfer in [Transfer::Meek, Transfer::Warren].iter() {
            let (excess, score) = x.transfer_votes_fp(&w, &base, transfer);
            assert_eq!(excess + score.values().sum::<u64>(), vsum * base);
        }
    }

    #[test]
    fn count_ranks() {
        let x = VoteTree::from(vec![
            (1, vec![0]),
            (2, vec![0, 1]),
            (3, vec![0, 1, 2]),
            (4, vec![2]),
            (5, vec![2, 1]),
            (6, vec![2, 1, 0]),
        ]);

        let mut points = HashMap::new();
        x.count_ranks(&mut points, &HashSet::new(), 0);
        assert_eq!(points[&(0, 0)], 6);
        assert_eq!(points[&(0, 2)], 6);
        assert_eq!(points[&(1, 1)], 16);
        assert_eq!(points[&(2, 0)], 15);
        assert_eq!(points[&(2, 2)], 3);
    }

    #[test]
    fn count_ranks_skip() {
        let x = VoteTree::from(vec![
            (1, vec![0]),
            (2, vec![0, 1]),
            (3, vec![0, 1, 2]),
            (4, vec![2]),
            (5, vec![2, 1]),
            (6, vec![2, 1, 0]),
        ]);

        let mut points = HashMap::new();
        x.count_ranks(&mut points, &([0].iter().cloned().collect()), 0);
        assert_eq!(points[&(1, 1)], 11);
        assert_eq!(points[&(2, 1)], 3);
        assert_eq!(points[&(2, 0)], 15);
        assert_eq!(points[&(1, 0)], 5);

        let mut points2 = HashMap::new();
        x.count_ranks(&mut points2, &([0, 2].iter().cloned().collect()), 0);
        assert_eq!(points2[&(1, 0)], 16);
    }
}
