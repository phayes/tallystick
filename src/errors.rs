use failure::Fail;

/// TallyError enum contains a list of all errors that may occur during a tally.
#[derive(Debug, Fail)]
pub enum TallyError {
  #[fail(display = "tallystick: vote contains duplicate candidates")]
  /// A vote contains duplicate candidates.
  VoteHasDuplicateCandidates,
}
