use crate::graph::Graph;
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
    model.set_obj_sense(Sense::Minimize);

    let mut dfvs = Vec::new();
    while let Some(cycle) = graph.find_cycle_with_fvs(&dfvs) {
        dfvs.push(cycle[0]);
        let row = model.add_row();
        model.set_row_lower(row, 1.);
        for vertex in cycle {
            model.set_weight(row, vars[vertex as usize], 1.);
        }
    }

    let solution = model.solve();
    solution.raw().obj_value().floor() as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycle_test_001() {
        let mut graph = Graph::new(5);
        graph.add_arc(0, 1);
        graph.add_arc(1, 2);
        graph.add_arc(2, 3);
        graph.add_arc(3, 4);
        graph.add_arc(4, 0);
        assert_eq!(lower_bound(&graph), 1);
    }

    #[test]
    fn cycle_test_002() {
        let mut graph = Graph::new(5);
        graph.add_arc(0, 1);
        graph.add_arc(1, 2);
        graph.add_arc(2, 3);
        graph.add_arc(3, 4);
        graph.add_arc(4, 0);
        assert_eq!(lower_bound(&graph), 1);
    }

    #[test]
    fn cycle_test_003() {
        let mut graph = Graph::new(5);
        graph.add_arc(0, 1);
        graph.add_arc(1, 0);
        graph.add_arc(2, 3);
        graph.add_arc(3, 2);
        graph.add_arc(4, 0);
        assert_eq!(lower_bound(&graph), 1);
    }
}
