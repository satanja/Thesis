use crate::{
    graph::{EdgeCycleCover, Graph, SplitReduce},
    heur::{Heuristic, SimulatedAnnealing},
    io::{Algorithm, Config},
    lower,
};

fn branch_and_reduce(
    graph: Graph,
    upper_bound: usize,
    freq: usize,
    mut depth: usize,
) -> Option<Vec<u32>> {
    if !graph.is_cyclic() {
        return Some(vec![]);
    }

    let mut k = upper_bound;
    let mut best_solution = None;

    // let (gd, gb, mut forced) = graph.split_reduce();
    let (gd, gb, mut forced) = if depth % freq == 0 {
        depth = 1;
        graph.split_reduce()
    } else {
        depth += 1;
        let (gd, gb) = graph.split();
        (gd, gb, vec![])
    };

    if forced.len() > k {
        return None;
    }

    k -= forced.len();

    let (lower_bound, mut candidate) = lower::lower_bound(&gd, &gb);
    if lower_bound > k {
        return None;
    }

    if candidate.len() <= k && gd.is_acyclic_with_fvs(&candidate) && gb.is_acyclic_with_fvs(&candidate) {
        if candidate.len() == lower_bound {
            candidate.append(&mut forced);
            return Some(candidate);
        }
        k = candidate.len() - 1;
        best_solution = Some(candidate);
    }

    let mut stars = gb.stars();
    if stars.is_empty() {
        let mut cycles = gd.edge_cycle_cover();
        cycles.sort_unstable_by(|a, b| a.len().cmp(&b.len()));
        let smallest_cycle = &mut cycles[0];

        smallest_cycle.sort_unstable_by(|a, b| {
            let a_key = gd.get_incoming(a).len() * gd.get_outgoing(a).len();
            let b_key = gd.get_incoming(b).len() * gd.get_outgoing(b).len();
            a_key.cmp(&b_key)
        });

        for vertex in smallest_cycle {
            let mut gc = gd.clone() + gb.clone();
            gc.remove_vertex(*vertex);
            let solution = branch_and_reduce(gc, k - 1, freq, depth);
            if let Some(mut dfvs) = solution {
                dfvs.push(*vertex);

                // current solution matches the lower bound, it must be an optimal solution
                if dfvs.len() == lower_bound {
                    dfvs.append(&mut forced);
                    return Some(dfvs);
                }
                k = dfvs.len() - 1; // look for a strictly better solution
                best_solution = Some(dfvs);
            }

            // we are not going to find a solution better than the lower bound
            if k <= lower_bound {
                break;
            }
        }
    } else {
        let mut max_star = std::mem::take(&mut stars[0]);
        for i in 1..stars.len() {
            if stars[i].1.len() > max_star.1.len() {
                max_star = std::mem::take(&mut stars[i]);
            }
        }
        let v = max_star.0;
        let mut nv = max_star.1;

        let mut gcv = gd.clone() + gb.clone();
        gcv.remove_vertex(v);
        let sv = branch_and_reduce(gcv, k - 1, freq, depth);
        if let Some(mut dfvs) = sv {
            dfvs.push(v);
            if dfvs.len() == lower_bound {
                dfvs.append(&mut forced);
                return Some(dfvs);
            }
            k = dfvs.len() - 1;
            best_solution = Some(dfvs);
        }

        if nv.len() <= k {
            let mut gcnv = gd.clone() + gb.clone();
            gcnv.remove_vertices(&nv);

            let snv = branch_and_reduce(gcnv, k - nv.len(), freq, depth);
            if let Some(mut dfvs) = snv {
                dfvs.append(&mut nv);
                if dfvs.len() == lower_bound {
                    dfvs.append(&mut forced);
                    return Some(dfvs);
                }
                best_solution = Some(dfvs);
            }
        }
    }

    if let Some(result) = best_solution.as_mut() {
        result.append(&mut forced);
    }
    best_solution
}

fn branch_and_bound(graph: Graph, upper_bound: usize) -> Option<Vec<u32>> {
    if !graph.is_cyclic() {
        return Some(vec![]);
    }
    let mut k = upper_bound;
    let mut best_solution = None;
    let (gd, gb) = graph.split();

    let (lower_bound, candidate) = lower::lower_bound(&gd, &gb);
    if lower_bound > k {
        return None;
    }

    if candidate.len() <= k && gd.is_acyclic_with_fvs(&candidate) {
        if candidate.len() == lower_bound {
            return Some(candidate);
        }
        k = candidate.len() - 1;
        best_solution = Some(candidate);
    }

    let mut stars = gb.stars();
    if stars.is_empty() {
        let mut cycles = gd.edge_cycle_cover();
        cycles.sort_unstable_by(|a, b| a.len().cmp(&b.len()));
        let smallest_cycle = &mut cycles[0];

        smallest_cycle.sort_unstable_by(|a, b| {
            let a_key = gd.get_incoming(a).len() * gd.get_outgoing(a).len();
            let b_key = gd.get_incoming(b).len() * gd.get_outgoing(b).len();
            a_key.cmp(&b_key)
        });

        for vertex in smallest_cycle {
            let mut gc = gd.clone() + gb.clone();
            gc.remove_vertex(*vertex);
            let solution = branch_and_bound(gc, k - 1);
            if let Some(mut dfvs) = solution {
                dfvs.push(*vertex);

                // current solution matches the lower bound, it must be an optimal solution
                if dfvs.len() == lower_bound {
                    return Some(dfvs);
                }
                k = dfvs.len() - 1; // look for a strictly better solution
                best_solution = Some(dfvs);
            }

            // we are not going to find a solution better than the lower bound
            if k <= lower_bound {
                break;
            }
        }
    } else {
        let mut max_star = std::mem::take(&mut stars[0]);
        for i in 1..stars.len() {
            if stars[i].1.len() > max_star.1.len() {
                max_star = std::mem::take(&mut stars[i]);
            }
        }
        let v = max_star.0;
        let mut nv = max_star.1;

        let mut gcv = gd.clone() + gb.clone();
        gcv.remove_vertex(v);
        let sv = branch_and_bound(gcv, k - 1);
        if let Some(mut dfvs) = sv {
            dfvs.push(v);
            if dfvs.len() == lower_bound {
                return Some(dfvs);
            }
            k = dfvs.len() - 1;
            best_solution = Some(dfvs);
        }

        if nv.len() <= k {
            let mut gcnv = gd.clone() + gb.clone();
            gcnv.remove_vertices(&nv);

            let snv = branch_and_bound(gcnv, k - nv.len());
            if let Some(mut dfvs) = snv {
                dfvs.append(&mut nv);
                if dfvs.len() == lower_bound {
                    return Some(dfvs);
                }
                best_solution = Some(dfvs);
            }
        }
    }

    best_solution
}

pub fn solve(graph: &mut Graph, config: &Config) -> Vec<u32> {
    let mut solution = Vec::new();
    let components = graph.tarjan(true).unwrap();
    for component in components {
        let subgraph = graph.induced_subgraph(component);

        let mut ub = SimulatedAnnealing::upper_bound(&subgraph);
        match config.algorithm() {
            Algorithm::BNR => {
                if let Some(mut sub_solution) =
                    branch_and_reduce(subgraph, ub.len() - 1, config.frequency(), 0)
                {
                    solution.append(&mut sub_solution);
                } else {
                    solution.append(&mut ub);
                }
            }
            Algorithm::BNB => {
                if let Some(mut sub_solution) = branch_and_bound(subgraph, ub.len() - 1) {
                    solution.append(&mut sub_solution);
                } else {
                    solution.append(&mut ub);
                }
            }
            _ => panic!("should not happen"),
        }
    }

    solution
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
    fn branch_and_reduce_test_001() {
        let n = 3;
        let graph = generate_clique(n);
        let solution = branch_and_reduce(graph.clone(), n, 0, 0).unwrap();
        assert_eq!(solution.len(), n - 1);
        assert!(graph.is_acyclic_with_fvs(&solution));
    }

    #[test]
    fn branch_and_reduce_test_002() {
        let n = 4;
        let graph = generate_clique(n);
        let solution = branch_and_reduce(graph.clone(), n, 0, 0).unwrap();
        assert_eq!(solution.len(), n - 1);
        assert!(graph.is_acyclic_with_fvs(&solution));
    }

    #[test]
    fn branch_and_reduce_test_003() {
        let n = 5;
        let graph = generate_clique(n);
        let solution = branch_and_reduce(graph.clone(), n, 0, 0).unwrap();
        assert_eq!(solution.len(), n - 1);
        assert!(graph.is_acyclic_with_fvs(&solution));
    }
}
