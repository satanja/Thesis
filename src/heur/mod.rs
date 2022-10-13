use crate::graph::{Graph, HeuristicReduce};

mod hs_sa;
mod hsheur;
mod sa;
pub use hs_sa::hitting_set_upper_bound;
pub use hs_sa::hitting_set_upper_bound_custom;
pub use hsheur::HittingSetDFVS;
use rustc_hash::FxHashSet;
pub use sa::SimulatedAnnealing;
pub trait Heuristic {
    fn upper_bound(graph: &Graph) -> Vec<u32>;
}

pub struct Greedy {}
impl Heuristic for Greedy {
    fn upper_bound(graph: &Graph) -> Vec<u32> {
        let mut copy = graph.clone();
        let mut solution = Vec::new();
        while copy.is_cyclic() {
            let v = copy.max_degree_vertex();
            copy.remove_vertex(v);
            solution.push(v);
        }

        make_minimal(&mut graph.clone(), solution)
    }
}

pub struct GRMaxDegree {}
impl Heuristic for GRMaxDegree {
    fn upper_bound(graph: &Graph) -> Vec<u32> {
        let mut copy = graph.clone();
        let mut solution = Vec::new();
        // use a different set of reduction rules because they perform better
        // with the heuristic
        solution.append(&mut copy.reduce());

        while copy.is_cyclic() {
            let v = copy.max_degree_vertex();
            copy.remove_vertex(v);
            solution.push(v);
            solution.append(&mut copy.reduce());
        }

        // debug_assert!(!graph.has_cycle_with_fvs(&solution));
        let p = make_minimal(&mut graph.clone(), solution);
        debug_assert!(graph.is_acyclic_with_fvs(&p));
        p
    }
}

pub struct GRCycle {}
impl Heuristic for GRCycle {
    fn upper_bound(graph: &Graph) -> Vec<u32> {
        let mut copy = graph.clone();
        let mut solution = Vec::new();

        solution.append(&mut copy.reduce());
        while let Some(cycle) = copy.find_cycle_with_fvs(&solution) {
            solution.push(cycle[0]);
            solution.append(&mut copy.reduce());
        }

        solution
    }
}

/// Reduces the solution to a minimal solution. Tries to reintroduce a vertex to
/// the graph, and if the graph is still acyclic, we can continue. Otherwise,
/// that vertex must be removed from the graph.
pub fn make_minimal(graph: &mut Graph, solution: Vec<u32>) -> Vec<u32> {
    let mut set: FxHashSet<_> = solution.iter().copied().collect();
    for vertex in &solution {
        graph.disable_vertex_post(*vertex);
    }

    for vertex in solution {
        graph.enable_vertex_post(vertex);
        if graph.is_cyclic() {
            graph.disable_vertex_post(vertex);
        } else {
            set.remove(&vertex);
            println!("optimized");
        }
    }
    set.into_iter().collect()
}
