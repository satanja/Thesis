use crate::{
    graph::{EdgeCycleCover, Graph, Reducable},
    util::{reduce_hitting_set, RangeSet},
};
use coin_cbc::{Col, Model, Sense, Solution};
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

    // Time to create a model and solve it.
    let mut dfvs = Vec::new();

    let mut model = Model::default();
    model.set_obj_sense(Sense::Minimize);
    model.set_parameter("log", "0");

    let mut vars = Vec::with_capacity(vertices);
    for _ in 0..vertices {
        let var = model.add_binary();
        model.set_obj_coeff(var, 1.);
        vars.push(var);
    }

    for constraint in preprocess_constraints {
        let cstr = model.add_row();
        model.set_row_lower(cstr, 1.);
        for vertex in constraint {
            model.set_weight(cstr, vars[vertex as usize], 1.);
        }
    }

    let mut solution = model.solve();
    recover_solution(&solution, &vars, &mut dfvs, graph.total_vertices());

    if graph.is_acyclic_with_fvs(&dfvs) {
        // We were lucky with our constraints
        // Recall that we already forced some vertices when partially reducing
        // our graph.
        dfvs.append(&mut forced);
        return Some(dfvs);
    }

    // Unfortunately, we are not lucky with our constraints, so we incrementally
    // include cycles from an edge cycle cover of the graph

    let ecc = graph.edge_cycle_cover();
    let mut ecc_mapping = vec![Vec::new(); vertices];
    for i in 0..ecc.len() {
        let cycle = &ecc[i];
        for vertex in cycle {
            ecc_mapping[*vertex as usize].push(i as u32);
        }
    }

    if true {
        model.set_initial_solution(&solution);

        for cycle in ecc {
            let cstr = model.add_row();
            model.set_row_lower(cstr, 1.);
            for vertex in &cycle {
                model.set_weight(cstr, vars[*vertex as usize], 1.);
            }
            model.set_col_initial_solution(vars[cycle[0] as usize], 1.)
        }

        let _out = shh::stdout();
        let solution = model.solve();
        recover_solution(&solution, &vars, &mut dfvs, graph.total_vertices());
        if graph.is_acyclic_with_fvs(&dfvs) {
            // We were lucky with our constraints
            // Recall that we already forced some vertices when partially reducing
            // our graph.
            dfvs.append(&mut forced);
            return Some(dfvs);
        }
    } else {
        loop {
            let mut cstr_set: RangeSet = (0..ecc.len() as u32).collect();
            for vertex in &dfvs {
                for hit in &ecc_mapping[*vertex as usize] {
                    cstr_set.remove(hit);
                }
            }
            let uncovered_constraints = cstr_set.get_set();
            if uncovered_constraints.is_empty() {
                break;
            }

            // println!("{} of {}", uncovered_constraints.len(), reduced_ecc.len());
            model.set_initial_solution(&solution);
            for index in uncovered_constraints {
                let cycle = &ecc[index as usize];
                let cstr = model.add_row();
                model.set_row_lower(cstr, 1.);
                for vertex in cycle {
                    model.set_weight(cstr, vars[*vertex as usize], 1.);
                }
                model.set_col_initial_solution(vars[cycle[0] as usize], 1.);
            }

            let _out = shh::stdout();
            solution = model.solve();
            recover_solution(&solution, &vars, &mut dfvs, graph.total_vertices());
            if graph.is_acyclic_with_fvs(&dfvs) {
                // We were lucky with our constraints
                // Recall that we already forced some vertices when partially reducing
                // our graph.
                dfvs.append(&mut forced);
                return Some(dfvs);
            }
        }
    }

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

fn recover_solution(solution: &Solution, vars: &Vec<Col>, dfvs: &mut Vec<u32>, vertices: usize) {
    dfvs.clear();
    for i in 0..vertices {
        if solution.col(vars[i]) >= 0.9995 {
            dfvs.push(i as u32);
        }
    }
}
