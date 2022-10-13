//! Vertex Cover based lower bound computation
use crate::graph::{EdgeCycleCover, Graph, ThreeCliques};
use coin_cbc::{Model, Sense};

pub fn lower_bound(graph: &Graph) -> usize {
    let _out = shh::stdout();
    let mut model = Model::default();

    let mut vars = Vec::with_capacity(graph.total_vertices());
    for _ in 0..graph.total_vertices() {
        let var = model.add_col();
        model.set_col_lower(var, 0.);
        model.set_col_upper(var, 1.);
        model.set_obj_coeff(var, 1.);
        vars.push(var);
    }

    let edge_cyles = graph.edge_cycle_cover();

    for cycle in edge_cyles {
        let cstr = model.add_row();
        model.set_row_lower(cstr, 1.);

        for v in cycle {
            model.set_weight(cstr, vars[v as usize], 1.);
        }
    }

    let three_cliques = graph.three_cliques();
    for (a, b, c) in three_cliques {
        let cstr = model.add_row();
        model.set_row_lower(cstr, 2.);

        model.set_weight(cstr, vars[a as usize], 1.);
        model.set_weight(cstr, vars[b as usize], 1.);
        model.set_weight(cstr, vars[c as usize], 1.);
    }

    model.set_obj_sense(Sense::Minimize);
    let solution = model.solve();

    let value = solution.raw().obj_value();
    (value - 1e-5).ceil() as usize
}
