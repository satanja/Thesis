//! Main entry point of the contest-deliverable. A Graph is read from stdin and
//! is then supplied to the algorithm, and after, the solution is written to
//! stdout.

#![allow(dead_code)]
mod exact;
mod graph;
mod heur;
mod io;
mod lower;
mod util;
use heur::Heuristic;

fn main() {
    let graph = io::read().unwrap();
    let solution = heur::HittingSetDFVS::upper_bound(&graph);
    io::write(solution);
}
