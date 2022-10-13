use crate::{
    exact,
    graph::{EdgeCycleCover, EdgeIter, Graph, ThreeCliques, FourCliques, TwinCliques},
    heur::hitting_set_upper_bound_custom,
    util::Constraint,
};
use grb::{expr::LinExpr, prelude::*};

pub fn lower_bound(graph: &Graph, undirected_graph: &Graph) -> (f64, Vec<u32>) {
    let _out = shh::stdout();
    let vertices = graph.total_vertices();

    let mut constraints = Vec::new();

    for (u, v) in undirected_graph.undir_edge_iter() {
        constraints.push(Constraint::new(vec![u, v], 1));
    }

    let mut iter = 5;
    let mut upper_bound = if constraints.is_empty() {
        Vec::new()
    } else {
        iter -= 1;
        hitting_set_upper_bound_custom(&constraints, vertices, 20_000)
    };

    while !graph.is_acyclic_with_fvs(&upper_bound) && iter > 0 {
        let cycles = graph.disjoint_edge_cycle_cover(&upper_bound);
        for cycle in cycles {
            constraints.push(Constraint::new(cycle, 1));
        }
        upper_bound = hitting_set_upper_bound_custom(&constraints, vertices, 20_000);
        iter -= 1;
    }

    let mut model = exact::init_model();

    let mut vars = Vec::with_capacity(vertices);
    for i in 0..vertices {
        let n = format!("v{}", i);
        let var = add_ctsvar!(model, name: &n, bounds: 0..1).unwrap();
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
    for (a, b, c) in cliques {
        let va = vars[a as usize];
        let vb = vars[b as usize];
        let vc = vars[c as usize];
        model.add_constr("", c!(va + vb + vc >= 2)).unwrap();
    }

    let four_cliques = undirected_graph.four_cliques();
    for [a, b, c, d] in four_cliques {
        let va = vars[a as usize];
        let vb = vars[b as usize];
        let vc = vars[c as usize];
        let vd = vars[d as usize];
        model.add_constr("", c!(va + vb + vc + vd >= 3)).unwrap();
    }

    let twins = undirected_graph.twin_cliques();
    for twin in twins {
        let lb = twin.len() - 1;
        let mut expr = LinExpr::new();
        for v in twin {
            let var = vars[v as usize];
            expr.add_term(1., var);
        }
        model.add_constr("", c!(expr >= lb)).unwrap();
    }

    model.optimize().unwrap();
    (model.get_attr(attr::ObjVal).unwrap(), upper_bound)
}
