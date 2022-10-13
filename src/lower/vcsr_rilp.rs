use crate::{
    graph::{EdgeCycleCover, Graph, Reducable, ThreeCliques, WeakThreeCliques},
    util::Constraint,
};
use coin_cbc::Sense;
use rustc_hash::FxHashSet;

pub fn lower_bound(original: &Graph) -> f64 {
    let mut graph = original.clone();
    let reduction = graph.reduce(graph.total_vertices()).unwrap();
    if graph.is_empty() {
        return reduction.len() as f64;
    }
    let vertices = graph.total_vertices();
    let mut constraints = Vec::new();
    let mut constraint_map = vec![Vec::new(); vertices];
    let mut forced = Vec::new();

    let mut undirected_graph = Graph::new(vertices);

    // Start form the undirected part of the graph
    // Include the undirected edges as constraints, and remove the undirected
    // edges from the graph. Safely reduce the graph (endpoints cannot be
    // reduced). Repeat until no undirected edges exist.
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
        // break;
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
        undirected_graph.remove_vertex(*vertex);
    }

    let mut preprocess_constraints = Vec::new();
    for i in 0..constraints.len() {
        if forced_constraints.contains(&i) {
            continue;
        }
        preprocess_constraints.push(std::mem::take(&mut constraints[i]));
    }

    let mut model = crate::exact::init_model();
    model.set_obj_sense(Sense::Minimize);

    let mut vars = Vec::with_capacity(vertices);
    for _ in 0..vertices {
        let var = model.add_col();
        model.set_col_lower(var, 0.);
        model.set_col_upper(var, 1.);
        model.set_obj_coeff(var, 1.);
        vars.push(var);
    }

    for constraint in &preprocess_constraints {
        let cstr = model.add_row();
        model.set_row_lower(cstr, 1.);
        for vertex in constraint.variables() {
            model.set_weight(
                cstr,
                vars[*vertex as usize],
                constraint.lower_bound() as f64,
            );
        }
    }

    let _out = shh::stdout();
    for cycle in graph.edge_cycle_cover() {
        let cstr = model.add_row();
        model.set_row_lower(cstr, 1.);

        for vertex in &cycle {
            model.set_weight(cstr, vars[*vertex as usize], 1.);
        }
    }

    for set in graph.weak_three_cliques() {
        let cstr = model.add_row();
        model.set_row_lower(cstr, 2.);
        for vertex in set {
            model.set_weight(cstr, vars[vertex as usize], 1.);
        }
    }

    let cliques = undirected_graph.undirected_three_cliques();
    for (a, b, c) in cliques {
        let cstr = model.add_row();
        model.set_row_lower(cstr, 2.);
        model.set_weight(cstr, vars[a as usize], 1.);
        model.set_weight(cstr, vars[b as usize], 1.);
        model.set_weight(cstr, vars[c as usize], 1.);
    }

    let solution = model.solve();
    solution.raw().obj_value() + forced.len() as f64 + reduction.len() as f64
}
