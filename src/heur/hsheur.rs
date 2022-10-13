use rustc_hash::FxHashSet;

use crate::{
    graph::{EdgeCycleCover, Graph, Reducable},
    util::Constraint,
};

use super::{Heuristic, hitting_set_upper_bound};

pub struct HittingSetDFVS {}
impl Heuristic for HittingSetDFVS {
    fn upper_bound(graph: &Graph) -> Vec<u32> {
        let mut graph = graph.clone();
        let vertices = graph.total_vertices();
        // let initial = Vec::new();
        let mut initial = graph.reduce(vertices).unwrap();

        if graph.is_empty() {
            return initial;
        }

        let mut constraints = Vec::new();
        let mut constraint_map = vec![Vec::new(); vertices];
        let mut forced = Vec::new();

        loop {
            let stars = graph.stars();
            if stars.is_empty() {
                break;
            }

            let mut sources = Vec::with_capacity(stars.len());
            for (source, neighbors) in &stars {
                for neighbor in neighbors {
                    if *source < *neighbor {
                        constraint_map[*source as usize].push(constraints.len());
                        constraint_map[*neighbor as usize].push(constraints.len());
                        constraints.push(Constraint::new(vec![*source, *neighbor], 1));
                    }
                }
                sources.push(*source);
            }
            graph.mark_forbidden(&sources);
            graph.remove_undirected_edges(stars);

            let mut reduced = graph.reduce(vertices).unwrap();
            if reduced.is_empty() {
                break;
            }
            forced.append(&mut reduced);
        }

        // Some generated constraints may already be satisfied, filter those.
        let mut forced_constraints = FxHashSet::default();
        for vertex in &forced {
            for constraint_index in &constraint_map[*vertex as usize] {
                forced_constraints.insert(*constraint_index);
            }
        }
        let mut hitting_set = Vec::new();
        for i in 0..constraints.len() {
            if forced_constraints.contains(&i) {
                continue;
            }
            hitting_set.push(std::mem::take(&mut constraints[i]));
        }

        for cycle in graph.edge_cycle_cover() {
            hitting_set.push(Constraint::new(cycle, 1));
        }

        let mut upper_bound = hitting_set_upper_bound(&hitting_set, vertices);
        while !graph.is_acyclic_with_fvs(&upper_bound) {
            let cycles = graph.disjoint_edge_cycle_cover(&upper_bound);
            for cycle in cycles {
                hitting_set.push(Constraint::new(cycle, 1));
            }
            upper_bound = hitting_set_upper_bound(&hitting_set, vertices);
        }
        upper_bound.append(&mut forced);
        upper_bound.append(&mut initial);
        upper_bound
    }
}
