use super::recover_solution;
use crate::{
    exact::vc_solver,
    graph::{EdgeCycleCover, Graph, Reducable, ThreeCliques, Undirected, WeakThreeCliques},
    heur::hitting_set_upper_bound,
    io::Config,
    util::Constraint,
};
use coin_cbc::Sense;
use rustc_hash::FxHashSet;

pub fn solve(graph: &mut Graph, config: &Config) -> Vec<u32> {
    let vertices = graph.total_vertices();
    let mut constraints = Vec::new();
    let mut constraint_map = vec![Vec::new(); vertices];
    let mut forced = Vec::new();

    if graph.is_undirected() {
        let mut dfvs = Vec::new();
        if vc_solver::solve(graph, &mut dfvs, config) {
            return dfvs;
        }
    }

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
    drop(constraints);

    let mut dfvs = Vec::new();
    if !preprocess_constraints.is_empty()
        && vc_solver::solve(&undirected_graph, &mut dfvs, config)
        && graph.is_acyclic_with_fvs(&dfvs)
    {
        dfvs.append(&mut forced);
        return dfvs;
    }

    let mut model = super::init_model();
    model.set_obj_sense(Sense::Minimize);

    let mut vars = Vec::with_capacity(vertices);
    for _ in 0..vertices {
        let var = model.add_binary();
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
    dfvs.clear();
    for cycle in graph.edge_cycle_cover() {
        let cstr = model.add_row();
        model.set_row_lower(cstr, 1.);

        for vertex in &cycle {
            model.set_weight(cstr, vars[*vertex as usize], 1.);
        }
        preprocess_constraints.push(Constraint::new(cycle, 1));
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

    let upper = hitting_set_upper_bound(&preprocess_constraints, graph.total_vertices());
    model.remove_initial_solution();
    for variable in upper {
        model.set_col_initial_solution(vars[variable as usize], 1.);
    }

    let solution = model.solve();
    recover_solution(&solution, &vars, &mut dfvs, vertices);
    if graph.is_acyclic_with_fvs(&dfvs) {
        dfvs.append(&mut forced);
        return dfvs;
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
    dfvs
}
