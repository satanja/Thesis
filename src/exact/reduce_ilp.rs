use super::recover_solution;
use crate::{
    graph::{EdgeCycleCover, Graph, Reducable},
    util::reduce_hitting_set,
};
use coin_cbc::{Col, Model, Sense};
use rustc_hash::FxHashSet;

pub fn solve(graph: &mut Graph) -> Option<Vec<u32>> {
    let vertices = graph.total_vertices();
    let mut constraints = Vec::new();
    let mut constraint_map = vec![Vec::new(); vertices];
    let mut forced = Vec::new();

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
                if *source < *neighbor {
                    constraint_map[*source as usize].push(constraints.len());
                    constraint_map[*neighbor as usize].push(constraints.len());
                    constraints.push(vec![*source, *neighbor]);
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
        for constraint in &constraint_map[*vertex as usize] {
            forced_constraints.insert(*constraint);
        }
    }
    let mut preprocess_constraints = Vec::new();
    for i in 0..constraints.len() {
        if forced_constraints.contains(&i) {
            continue;
        }
        preprocess_constraints.push(std::mem::take(&mut constraints[i]));
    }
    drop(constraints);

    // Done filtering already satisified constraints. Reduce the set of constraints
    // using a Hitting Set reduction.
    let mut reduction = reduce_hitting_set(&mut preprocess_constraints, vertices as u32);
    let reduced_constraints = reduction.reduced;

    // Time to create a model and solve it.
    let mut dfvs = Vec::new();

    let (model, vars) = create_model(vertices, reduced_constraints);
    let solution = model.solve();
    recover_solution(&solution, &vars, &mut dfvs, graph.total_vertices());
    let mut candidate_dfvs = dfvs.clone();
    candidate_dfvs.append(&mut reduction.forced);

    if graph.is_acyclic_with_fvs(&candidate_dfvs) {
        // We were lucky with our constraints
        // Recall that we already forced some vertices when partially reducing
        // our graph.
        candidate_dfvs.append(&mut forced);
        return Some(candidate_dfvs);
    }

    // Unfortunately, we are not lucky with our constraints, so we have to drop
    // old model, and find new constraints.
    drop(model);

    for cycle in graph.edge_cycle_cover() {
        preprocess_constraints.push(cycle);
    }

    let mut reduction = reduce_hitting_set(&mut preprocess_constraints, vars.len() as u32);
    let reduced_constraints = reduction.reduced;
    let (model, vars) = create_model(vertices, reduced_constraints);

    let solution = model.solve();
    recover_solution(&solution, &vars, &mut dfvs, graph.total_vertices());
    let mut candidate_dfvs = dfvs.clone();
    candidate_dfvs.append(&mut reduction.forced);

    if graph.is_acyclic_with_fvs(&candidate_dfvs) {
        candidate_dfvs.append(&mut forced);
        return Some(candidate_dfvs);
    }

    // Again not lucky, let's just add constraints until we do have a solution.
    drop(model);

    let (mut model, vars) = create_model(vertices, preprocess_constraints);
    loop {
        let mut changed = false;
        while let Some(cycle) = graph.find_cycle_with_fvs(&dfvs) {
            changed = true;
            dfvs.push(cycle[0]);
            let row = model.add_row();
            model.set_row_lower(row, 1.);
            for vertex in cycle {
                model.set_weight(row, vars[vertex as usize], 1.);
            }
        }

        if !changed {
            break;
        }

        let _out = shh::stdout();

        let solution = model.solve();

        recover_solution(&solution, &vars, &mut dfvs, graph.total_vertices());
    }
    dfvs.append(&mut forced);
    Some(dfvs)
}

fn create_model(vertices: usize, constraints: Vec<Vec<u32>>) -> (Model, Vec<Col>) {
    let mut model = super::init_model();
    model.set_obj_sense(Sense::Minimize);

    let mut vars = Vec::with_capacity(vertices);
    for _ in 0..vertices {
        let var = model.add_binary();
        model.set_obj_coeff(var, 1.);
        vars.push(var);
    }

    for constraint in constraints {
        let cstr = model.add_row();
        model.set_row_lower(cstr, 1.);
        for vertex in constraint {
            model.set_weight(cstr, vars[vertex as usize], 1.);
        }
    }

    (model, vars)
}
