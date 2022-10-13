use super::recover_solution;
use crate::heur::{make_minimal, GRMaxDegree};
use crate::{graph::Graph, heur::Heuristic};
use coin_cbc::Sense;

pub fn solve(graph: &Graph) -> Option<Vec<u32>> {
    let mut model = super::init_model();
    model.set_parameter("log", "0");

    let mut vars = Vec::with_capacity(graph.total_vertices());
    for _ in 0..graph.total_vertices() {
        let var = model.add_binary();
        model.set_obj_coeff(var, 1.);
        vars.push(var);
    }
    model.set_obj_sense(Sense::Minimize);

    let _ilp_duration = 0;

    let mut dfvs = GRMaxDegree::upper_bound(graph);
    let cycles = graph.find_cycle_from_minimal(&dfvs);
    for cycle in cycles {
        let row = model.add_row();
        model.set_row_lower(row, 1.);
        for vertex in cycle {
            model.set_weight(row, vars[vertex as usize], 1.);
        }
    }

    let solution = model.solve();

    recover_solution(&solution, &vars, &mut dfvs, graph.total_vertices());

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

        if changed {
            dfvs = make_minimal(&mut graph.clone(), dfvs);
            let cycles = graph.find_cycle_from_minimal(&dfvs);
            for cycle in cycles {
                let row = model.add_row();
                model.set_row_lower(row, 1.);
                for vertex in cycle {
                    model.set_weight(row, vars[vertex as usize], 1.);
                }
            }
        } else {
            break;
        }

        let _out = shh::stdout();

        let solution = model.solve();
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
