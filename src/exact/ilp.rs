use super::{recover_solution, splitter::split_reduction};
use crate::{
    exact::vc_solver,
    graph::{EdgeCycleCover, Graph, ThreeCliques, Undirected},
    heur::hitting_set_upper_bound,
    io::Config,
    util::Constraint,
};
use coin_cbc::Sense;

pub fn solve(graph: Graph, config: &Config) -> Vec<u32> {
    let vertices = graph.total_vertices();

    let mut try_vc_solver = true;
    if graph.is_undirected() {
        let mut dfvs = Vec::new();
        if vc_solver::solve(&graph, &mut dfvs, config) {
            return dfvs;
        } else {
            try_vc_solver = false;
        }
    }

    let data = split_reduction(graph);
    let graph = data.directed_graph;
    let undirected_graph = data.undirected_graph;
    let mut constraints = data.constraints;
    let mut upper_bound = data.upper_bound;
    let mut split_reduced = data.split_reduced;

    if graph.is_empty() && try_vc_solver {
        let mut dfvs = Vec::new();
        if vc_solver::solve(&undirected_graph, &mut dfvs, config) {
            return dfvs;
        } else {
            try_vc_solver = false;
        }
    }

    let mut dfvs = Vec::new();
    if !undirected_graph.is_empty()
        && try_vc_solver
        && vc_solver::solve(&undirected_graph, &mut dfvs, config)
        && graph.is_acyclic_with_fvs(&dfvs)
    {
        dfvs.append(&mut split_reduced);
        return dfvs;
    }

    let _out = shh::stdout();
    let mut model = super::init_model();
    model.set_obj_sense(Sense::Minimize);

    let mut vars = Vec::with_capacity(vertices);
    for _ in 0..vertices {
        let var = model.add_binary();
        model.set_obj_coeff(var, 1.);
        vars.push(var);
    }

    for constraint in &constraints {
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

    let cliques = undirected_graph.undirected_three_cliques();
    for (a, b, c) in cliques {
        let cstr = model.add_row();
        model.set_row_lower(cstr, 2.);
        model.set_weight(cstr, vars[a as usize], 1.);
        model.set_weight(cstr, vars[b as usize], 1.);
        model.set_weight(cstr, vars[c as usize], 1.);
    }

    // let upper = hitting_set_upper_bound(&preprocess_constraints, graph.total_vertices());
    // model.remove_initial_solution();
    for variable in &upper_bound {
        model.set_col_initial_solution(vars[*variable as usize], 1.);
    }

    let solution = model.solve();
    recover_solution(&solution, &vars, &mut dfvs, vertices);

    // Our upper bound is an optimal solution for the ILP, which is
    // and it is also a DFVS, so it must be an optimal DFVS
    // We perform this check since the ILP may shift variables, breaking our
    // solution.
    if dfvs.len() == upper_bound.len() {
        upper_bound.append(&mut split_reduced);
        return upper_bound;
    }

    // Even though our upper bound is not optimal, we have found an optimal
    // DFVS
    if graph.is_acyclic_with_fvs(&dfvs) {
        dfvs.append(&mut split_reduced);
        return dfvs;
    }

    // We unfortunately do not have a solution, we need to iteratively
    // add edge cycle covers until we do
    while !graph.is_acyclic_with_fvs(&dfvs) {
        let cycles = graph.disjoint_edge_cycle_cover(&dfvs);
        for cycle in cycles {
            let cstr = model.add_row();
            model.set_row_lower(cstr, 1.);
            for vertex in &cycle {
                model.set_weight(cstr, vars[*vertex as usize], 1.);
            }
            constraints.push(Constraint::new(cycle, 1));
        }

        // this is an upper bound on the hitting set instance, not an upper
        // for the graph itself
        let upper_bound = hitting_set_upper_bound(&constraints, vertices);
        model.remove_initial_solution();
        for variable in &upper_bound {
            model.set_col_initial_solution(vars[*variable as usize], 1.);
        }

        let _out = shh::stdout();
        let solution = model.solve();
        recover_solution(&solution, &vars, &mut dfvs, vertices);
    }
    dfvs.append(&mut split_reduced);
    dfvs
}
