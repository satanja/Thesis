use std::path::PathBuf;

use super::{recover_solution, splitter::split_reduction};
use crate::{
    exact::vc_solver,
    graph::{EdgeCycleCover, Graph, ThreeCliques, Undirected},
    heur::hitting_set_upper_bound,
    io::Config,
    util::Constraint,
};
use grb::{expr::LinExpr, prelude::*};
use rustc_hash::FxHashSet;

pub fn solve(graph: Graph, config: &Config) -> Vec<u32> {
    let _out = shh::stdout();
    let vertices = graph.total_vertices();

    let mut try_vc_solver = false;
    if graph.is_undirected() {
        let mut dfvs = Vec::new();
        if vc_solver::solve(&graph, &mut dfvs, config) {
            return dfvs;
        } else {
            try_vc_solver = false;
        }
    }
    let data = split_reduction(graph, &config);
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
    
    let mut model = super::init_model();
    // model.set_obj_sense(Sense::Minimize);
    
    let mut vars = Vec::with_capacity(vertices);
    for i in 0..vertices {
        let n = format!("v{}", i);
        let var = add_binvar!(model, name: &n).unwrap();
        vars.push(var);
    }
    model
        .set_objective(vars.iter().sum::<Expr>(), Minimize)
        .unwrap();

    for constraint in &constraints {
        let mut expr = LinExpr::new();
        for v in constraint.variables() {
            let var = vars[*v as usize];
            expr.add_term(1., var);
        }

        model.add_constr("", c!(expr >= 1)).unwrap();
    }

    let cliques = undirected_graph.undirected_three_cliques();
    let delta = cliques.len();
    eprintln!("dc = {delta}");
    for (a, b, c) in cliques {
        let va = vars[a as usize];
        let vb = vars[b as usize];
        let vc = vars[c as usize];
        model.add_constr("", c!(va + vb + vc >= 2)).unwrap();
    }

    // let upper = hitting_set_upper_bound(&preprocess_constraints, graph.total_vertices());
    // model.remove_initial_solution();
    for variable in &upper_bound {
        model
            .set_obj_attr(attr::Start, &vars[*variable as usize], 1.)
            .unwrap();
    }

    model.optimize().unwrap();
    recover_solution(&model, &vars, &mut dfvs);

    // Our upper bound is an optimal solution for the ILP, which is
    // and it is also a DFVS, so it must be an optimal DFVS
    // We perform this check since the ILP may shift variables, breaking our
    // solution.
    if dfvs.len() == upper_bound.len() {
        upper_bound.append(&mut split_reduced);
        eprintln!("{}", constraints.len() + delta);
        eprintln!("{}", alive_variables(&constraints));
        eprintln!("iters = 1");
        return upper_bound;
    }

    // Even though our upper bound is not optimal, we have found an optimal
    // DFVS
    if graph.is_acyclic_with_fvs(&dfvs) {
        dfvs.append(&mut split_reduced);
        eprintln!("{}", constraints.len() + delta);
        eprintln!("{}", alive_variables(&constraints));
        eprintln!("iters = 1");
        return dfvs;
    }

    // We unfortunately do not have a solution, we need to iteratively
    // add edge cycle covers until we do
    let mut iters = 1;
    while !graph.is_acyclic_with_fvs(&dfvs) {
        iters += 1;
        let cycles = graph.disjoint_edge_cycle_cover(&dfvs);
        for cycle in cycles {
            let mut expr = LinExpr::new();
            for v in &cycle {
                let var = vars[*v as usize];
                expr.add_term(1., var);
            }
            model.add_constr("", c!(expr >= 1)).unwrap();
            constraints.push(Constraint::new(cycle, 1));
        }
        // this is an upper bound on the hitting set instance, not an upper
        // for the graph itself
        let upper_bound = hitting_set_upper_bound(&constraints, vertices);
        model.reset().unwrap();
        for variable in &upper_bound {
            model
                .set_obj_attr(attr::Start, &vars[*variable as usize], 1.)
                .unwrap();
        }

        let _out = shh::stdout();
        model.optimize().unwrap();
        recover_solution(&model, &vars, &mut dfvs);
    }
    dfvs.append(&mut split_reduced);
    eprintln!("{}", constraints.len() + delta);
    eprintln!("{}", alive_variables(&constraints));
    eprintln!("{iters}");
    dfvs
}

fn alive_variables(constraints: &[Constraint]) -> usize {
    let mut set = FxHashSet::default();
    for constraint in constraints {
        for var in constraint.variables() {
            set.insert(*var);
        }
    }
    set.len()
}