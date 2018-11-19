#![warn(rust_2018_idioms)]
#![warn(missing_docs)]
#![feature(crate_visibility_modifier)]
#![feature(nll)]


#[allow(unused_imports)]
#[macro_use] extern crate indexmap;
#[macro_use] extern crate derive_more;

extern crate hashbrown;
extern crate petgraph;
extern crate num_traits;

/// Plurality voting is an electoral system in which each voter is allowed to vote for only one candidate, 
/// and the candidate who polls the most among their counterparts (a plurality) is elected. It it may be called
/// first-past-the-post (FPTP), single-choice voting, simple plurality or relative/simple majority. 
/// 
/// # Example
/// ```
///    use tallyman::plurality::DefaultTally;
///
///    // Election between Alice, Bob, and Cir with two winners.
///    let mut tally = DefaultTally::new(2);
///    tally.add("Alice");
///    tally.add("Cir");
///    tally.add("Bob");
///    tally.add("Alice");
///    tally.add("Alice");
///    tally.add("Bob");
/// 
///    let winners = tally.winners().into_unranked();
///    println!("The winners are {:?}", winners);
/// ```
pub mod plurality;

/// Approval voting is a single-winner electoral system where each voter may select ("approve") any number of
/// candidates. The winner is the most-approved candidate.
pub mod approval;

/// Score voting or "range voting" is an electoral system in which voters give each candidate a score,
/// the scores are summed, and the candidate with the highest total is elected. It has been described
/// by various other names including "evaluative voting", "utilitarian voting", and "the point system".
pub mod score;


pub mod stv;
pub mod condorcet;
pub mod result;

