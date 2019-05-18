
use failure::Fail;

/// TallyError enum contains a list of all errors that may occur during a tally.
#[derive(Debug, Fail)]
pub enum TallyError {
  #[fail(display = "tallystick: vote contains duplicate candidates")]
  /// A vote contains duplicate candidates.
  VoteHasDuplicateCandidates,
}


/// ParseError enum contains a list of all errors that may occur during vote parsing.
#[derive(Debug, Fail)]
pub enum ParseError {
  #[fail(display = "tallystick: error parsing numeric value {}", 0)]
  /// Unable to parse this numeric value
  ParseError(String),

  #[fail(display = "tallystick: error reading vote data: {}", 0)]
  /// Unable to read cursor
  ReadError(std::io::Error)
}

impl From<std::io::Error> for ParseError {
    fn from(error: std::io::Error) -> Self {
        ParseError::ReadError(error)
    }
}