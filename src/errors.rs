use failure::Fail;

#[derive(Debug, Fail)]
pub enum TallyError {
  #[fail(display = "tallyman: vote contains duplicate candidates")]
  /// A vote contains duplicate candidates.
  VoteHasDuplicateCandidates,
}
