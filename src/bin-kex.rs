#![allow(dead_code)]
mod exact;
mod graph;
mod heur;
mod io;
mod lower;
mod util;

use graph::SplitReduce;

fn main() {
    let graph = io::read().unwrap();
    let (gd, gb, _) = graph.split_reduce();
    println!("{}", gd + gb);
}
