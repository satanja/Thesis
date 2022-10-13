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

fn main() {
    let config = io::config();
    let graph = io::read().unwrap();
    let solution = exact::solve(graph, &config);
    io::write(solution);
}
