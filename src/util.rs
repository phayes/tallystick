use std::io::BufRead;
use std::io::BufReader;

#[derive(Debug)]
pub enum ParsedVote {
  Unranked(Vec<String>),
  Ranked(Vec<(String, u32)>),
}

pub fn read_votes<T: std::io::Read>(votes: T) -> Vec<(ParsedVote, u64)> {
  let mut reader = BufReader::new(votes);

  let mut res = Vec::new();
  for line in reader.lines() {
    let line = line.unwrap(); // TODO: Handle this
    if line.trim().len() > 0 {
      res.push(parse_line_into_vote(&line))
    }
  }

  return res;
}

fn parse_line_into_vote(line: &str) -> (ParsedVote, u64) {
  let parts: Vec<&str> = line.trim().split("*").collect();

  let weight;
  if parts.len() == 1 {
    weight = 1;
  } else {
    weight = parts[1].trim().parse::<u64>().unwrap();
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
    return (ParsedVote::Ranked(vote), weight);
  } else {
    let mut unranked_vote = Vec::<String>::with_capacity(vote.len());
    for (vote, _rank) in vote.drain(..) {
      unranked_vote.push(vote);
    }
    return (ParsedVote::Unranked(unranked_vote), weight);
  }
}
