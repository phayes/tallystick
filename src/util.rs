pub use crate::errors::ParseError;
use crate::errors::TallyError;
use num_traits::Num;
use std::convert::TryInto;

use std::io::BufRead;
use std::io::BufReader;

/// A parsed vote, either ranked or unranked.
#[derive(Debug)]
pub enum ParsedVote {
    /// An unranked vote. Candidates are returned in preferential order, with the most significant selection first.
    Unranked(Vec<String>),

    /// A ranked vote as (candidate, rank) pairs. Ranks are ordered ascending, so that the most significant rank is rank 0.
    Ranked(Vec<(String, u32)>),
}

impl ParsedVote {
    /// Convert unranked ParsedVote into a ranked parsed vote.
    /// This is a no-op if the vote is already ranked
    pub fn into_ranked(self) -> Vec<(String, u32)> {
        match self {
            ParsedVote::Ranked(ranked) => ranked,
            ParsedVote::Unranked(mut unranked) => {
                let mut ranked = Vec::<(String, u32)>::with_capacity(unranked.len());
                for (rank, vote) in unranked.drain(..).enumerate() {
                    // Safe to unwrap here since we can't have more than u32::MAX candidates anyways.
                    ranked.push((vote, rank.try_into().unwrap()));
                }
                ranked
            }
        }
    }
}

/// Read votes from a reader, parsing them and returning a vector of parsed votes and their weights.
///
/// TODO: Add Example
pub fn read_votes<T: std::io::Read, C: Num>(votes: T) -> Result<Vec<(ParsedVote, C)>, ParseError> {
    let reader = BufReader::new(votes);

    let mut res = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if !line.trim().is_empty() {
            res.push(parse_line_into_vote(&line)?)
        }
    }

    Ok(res)
}

fn parse_line_into_vote<C: Num>(line: &str) -> Result<(ParsedVote, C), ParseError> {
    let parts: Vec<&str> = line.trim().split('*').collect();

    let weight;
    if parts.len() == 1 {
        weight = C::one();
    } else {
        weight = match C::from_str_radix(parts[1].trim(), 10) {
            Ok(num) => num,
            Err(_) => {
                return Err(ParseError::ParseError(parts[1].trim().to_string()));
            }
        }
    }

    let mut vote = Vec::<(String, u32)>::new();
    let mut candidate_buf = String::new();
    let mut rank = 0;
    let mut is_ranked = false;
    for c in parts[0].trim().chars() {
        if c == '>' {
            vote.push((candidate_buf.trim().to_string(), rank));
            candidate_buf.clear();
            rank += 1;
        } else if c == '=' {
            vote.push((candidate_buf.trim().to_string(), rank));
            candidate_buf.clear();
            is_ranked = true;
        } else {
            candidate_buf.push(c);
        }
    }
    if !candidate_buf.trim().is_empty() {
        vote.push((candidate_buf.trim().to_string(), rank));
    }

    if is_ranked {
        Ok((ParsedVote::Ranked(vote), weight))
    } else {
        let mut unranked_vote = Vec::<String>::with_capacity(vote.len());
        for (vote, _rank) in vote.drain(..) {
            unranked_vote.push(vote);
        }
        Ok((ParsedVote::Unranked(unranked_vote), weight))
    }
}

/// Check for duplicates in a transitive vote.
pub fn check_duplicates_transitive_vote<T: Eq>(vote: &[T]) -> Result<(), TallyError> {
    for (i, candidate) in vote.iter().enumerate() {
        for j in i + 1..vote.len() {
            if &vote[j] == candidate {
                return Err(TallyError::VoteHasDuplicateCandidates);
            }
        }
    }

    Ok(())
}

/// Check for duplicates in a ranked vote.
pub fn check_duplicates_ranked_vote<T: Eq>(vote: &[(T, u32)]) -> Result<(), TallyError> {
    for (i, (candidate, _rank)) in vote.iter().enumerate() {
        for j in i + 1..vote.len() {
            if &vote[j].0 == candidate {
                return Err(TallyError::VoteHasDuplicateCandidates);
            }
        }
    }

    Ok(())
}
