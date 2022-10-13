//! Vertex Cover ILP solver
use super::recover_solution;
use crate::graph::{EdgeIter, Graph};
use coin_cbc::Sense;

pub fn solve(graph: &Graph) -> Option<Vec<u32>> {
    let _out = shh::stdout();
    let mut model = super::init_model();
    // TODO possible optimization flags

    let mut vars = Vec::with_capacity(graph.total_vertices());
    for _ in 0..graph.total_vertices() {
        let var = model.add_binary();
        model.set_obj_coeff(var, 1.);
        vars.push(var);
    }

    let edges = graph.undir_edge_iter();

    for (u, v) in edges {
        let cstr = model.add_row();
        model.set_row_lower(cstr, 1.);

        model.set_weight(cstr, vars[u as usize], 1.);
        model.set_weight(cstr, vars[v as usize], 1.);
    }

    model.set_obj_sense(Sense::Minimize);
    let solution = model.solve();

    let mut dfvs = Vec::new();
    recover_solution(&solution, &vars, &mut dfvs, graph.total_vertices());
    Some(dfvs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_clique(vertices: usize) -> Graph {
        let mut graph = Graph::new(vertices);
        for i in 0..vertices {
            for j in i + 1..vertices {
                graph.add_arc(i as u32, j as u32);
                graph.add_arc(j as u32, i as u32);
            }
        }
        graph
    }

    #[test]
    fn clique_test_001() {
        let n = 3;
        let graph = generate_clique(n);
        let solution = solve(&graph).unwrap();
        assert_eq!(solution.len(), n - 1);
        assert!(graph.is_acyclic_with_fvs(&solution));
    }

    #[test]
    fn clique_test_002() {
        let n = 4;
        let graph = generate_clique(n);
        let solution = solve(&graph).unwrap();
        assert_eq!(solution.len(), n - 1);
        assert!(graph.is_acyclic_with_fvs(&solution));
    }

    #[test]
    fn clique_test_003() {
        let n = 5;
        let graph = generate_clique(n);
        let solution = solve(&graph).unwrap();
        assert_eq!(solution.len(), n - 1);
        assert!(graph.is_acyclic_with_fvs(&solution));
    }
}
