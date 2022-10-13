use super::recover_solution;
use crate::graph::Graph;
use coin_cbc::Sense;
use std::time::Instant;

pub fn solve(graph: &Graph) -> Option<Vec<u32>> {
    let start = Instant::now();
    let mut model = super::init_model();

    let mut vars = Vec::with_capacity(graph.total_vertices());
    for _ in 0..graph.total_vertices() {
        let var = model.add_binary();
        model.set_obj_coeff(var, 1.);
        vars.push(var);
    }
    model.set_obj_sense(Sense::Minimize);

    let mut dfvs = Vec::new();
    let mut ilp_duration = 0;
    loop {
        let elapsed = start.elapsed().as_secs();
        if elapsed + ilp_duration > 450 {
            println!("{}s", elapsed);
            return None;
        }

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

        let ilp_start = Instant::now();
        let solution = model.solve();
        ilp_duration = ilp_start.elapsed().as_secs();

        recover_solution(&solution, &vars, &mut dfvs, graph.total_vertices());
    }
    Some(dfvs)
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
        let solution = solve(&graph).unwrap();
        assert_eq!(solution.len(), 1);
        assert!(graph.is_acyclic_with_fvs(&solution));
    }

    #[test]
    fn cycle_test_002() {
        let mut graph = Graph::new(5);
        graph.add_arc(0, 1);
        graph.add_arc(1, 0);
        graph.add_arc(2, 3);
        graph.add_arc(3, 2);
        graph.add_arc(4, 0);
        let solution = solve(&graph).unwrap();
        assert_eq!(solution.len(), 2);
        assert!(graph.is_acyclic_with_fvs(&solution));
    }
}
