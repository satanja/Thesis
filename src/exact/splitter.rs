use rustc_hash::FxHashSet;

use crate::{
    graph::{EdgeCycleCover, Graph, Reducable},
    util::Constraint, heur::hitting_set_upper_bound, io::Config,
};

pub struct ILPData {
    /// The forced vertices found during the split reduction
    pub split_reduced: Vec<u32>,

    /// A hopefully good set of constraints to obtain a solution
    pub constraints: Vec<Constraint>,

    /// An upper bound for `constraints`
    pub upper_bound: Vec<u32>,

    /// The completely directed graph generated in the split reduction
    pub directed_graph: Graph,

    /// The completely undirected graph generated in the split reduction
    pub undirected_graph: Graph,
}

pub fn split_reduction(mut graph: Graph, config: &Config) -> ILPData {
    let vertices = graph.total_vertices();

    let mut constraints = Vec::new();
    let mut constraint_map = vec![Vec::new(); vertices];
    let mut forced = Vec::new();
    let mut undirected_graph = Graph::new(vertices);

    loop {
        let stars = graph.stars();
        if stars.is_empty() {
            break;
        }

        let mut sources = Vec::with_capacity(stars.len());
        for (source, neighbors) in &stars {
            for neighbor in neighbors {
                undirected_graph.add_arc(*source, *neighbor);
                undirected_graph.add_arc(*neighbor, *source);

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

        if !config.reduce() && true {
            break;
        }

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
        undirected_graph.remove_vertex(*vertex);
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

    ILPData {
        split_reduced: forced,
        constraints: hitting_set,
        upper_bound,
        directed_graph: graph,
        undirected_graph,
    }
}

