pub use crate::errors::ParseError;
use num_traits::Num;
use std::fmt::Debug;
use std::io::BufReader;
use std::io::BufRead;
use std::iter::Iterator;
use std::convert::TryInto;

#[derive(Debug)]
pub enum ParsedVote {
  Unranked(Vec<String>),
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

pub fn read_votes<T: std::io::Read, C: Num + Debug>(votes: T) -> Result<Vec<(ParsedVote, C)>, ParseError> {
  let reader = BufReader::new(votes);

  let mut res = Vec::new();
  for line in reader.lines() {
    let line = line?;
    if line.trim().len() > 0 {
      res.push(parse_line_into_vote(&line)?)
    }
  }

  return Ok(res);
}

fn parse_line_into_vote<C: Num + Debug>(line: &str) -> Result<(ParsedVote, C), ParseError> {
  let parts: Vec<&str> = line.trim().split("*").collect();

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
  if candidate_buf.trim().len() > 0 {
    vote.push((candidate_buf.trim().to_string(), rank));
  }

  if is_ranked {
    return Ok((ParsedVote::Ranked(vote), weight));
  } else {
    let mut unranked_vote = Vec::<String>::with_capacity(vote.len());
    for (vote, _rank) in vote.drain(..) {
      unranked_vote.push(vote);
    }
    return Ok((ParsedVote::Unranked(unranked_vote), weight));
  }
}
