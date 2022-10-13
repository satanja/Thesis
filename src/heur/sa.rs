use super::Heuristic;
use crate::{
    graph::{Compressor, Graph, HeuristicReduce},
    util::RangeSet,
};
use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
    Rng, SeedableRng,
};

pub struct SimulatedAnnealing {
    graph: Graph,
    mapping: Vec<u32>,
    conf_vtoi: Vec<Option<usize>>,
    conf_itov: Vec<Option<u32>>,
    out_cache: Vec<Option<usize>>,
    in_cache: Vec<Option<usize>>,
    dfvs: RangeSet,
    rng: StdRng,
    reduced: Vec<u32>,
    iter: Option<usize>,
}

impl SimulatedAnnealing {
    fn with_max_iter(graph: &Graph, iter: usize) -> SimulatedAnnealing {
        let mut sa = Self::new(graph, true);
        sa.set_max_iter(iter);
        sa
    }

    fn set_max_iter(&mut self, iter: usize) {
        self.iter = Some(iter);
    }

    fn new(graph: &Graph, reduce_and_compress: bool) -> SimulatedAnnealing {
        let mut clone = graph.clone();
        let mut reduced = Vec::new();
        let (compressed, mapping) = if reduce_and_compress {
            reduced = clone.reduce();
            clone.compress()
        } else {
            let vertices = clone.total_vertices();
            (clone, (0..vertices as u32).collect())
        };

        let vertices = compressed.total_vertices();
        SimulatedAnnealing {
            graph: compressed,
            mapping,
            conf_vtoi: vec![None; vertices],
            conf_itov: Vec::with_capacity(vertices),
            out_cache: vec![None; vertices],
            in_cache: vec![None; vertices],
            dfvs: (0..vertices as u32).collect(),
            rng: StdRng::seed_from_u64(0),
            reduced,
            iter: None,
        }
    }

    fn out_index(&mut self, vertex: &u32) -> usize {
        // maybe we should cache this?
        // if let Some(index) = self.out_cache[*vertex as usize] {
        //     return index;
        // }

        let outgoing = self.graph.get_outgoing(vertex);
        let mut min = self.conf_vtoi.len();
        for vertex in outgoing {
            if let Some(index) = self.conf_vtoi[*vertex as usize] {
                min = std::cmp::min(min, index);
            }
        }
        // self.out_cache[*vertex as usize] = Some(min);
        min
    }

    fn conflicts_out_size(&self, vertex: &u32, index: usize) -> usize {
        let outgoing = self.graph.get_outgoing(vertex);
        let mut result = 0;
        for vertex in outgoing {
            if let Some(other_index) = self.conf_vtoi[*vertex as usize] {
                if other_index < index {
                    result += 1;
                }
            }
        }
        result
    }

    fn conflicts_out(&self, vertex: &u32, index: usize) -> Vec<u32> {
        let outgoing = self.graph.get_outgoing(vertex);
        let mut result = Vec::with_capacity(outgoing.len());
        for vertex in outgoing {
            if let Some(other_index) = self.conf_vtoi[*vertex as usize] {
                if other_index < index {
                    result.push(*vertex);
                }
            }
        }
        result
    }

    fn in_index(&mut self, vertex: &u32) -> usize {
        // if let Some(index) = self.in_cache[*vertex as usize] {
        //     return index;
        // }

        let incoming = self.graph.get_incoming(vertex);
        let mut max = 0;
        for vertex in incoming {
            if let Some(index) = self.conf_vtoi[*vertex as usize] {
                max = std::cmp::max(max, index);
            }
        }

        // self.in_cache[*vertex as usize] = Some(max + 1);
        max + 1
    }

    fn conflicts_in_size(&self, vertex: &u32, index: usize) -> usize {
        let incoming = self.graph.get_incoming(vertex);
        let mut result = 0;
        for vertex in incoming {
            if let Some(other_index) = self.conf_vtoi[*vertex as usize] {
                if other_index >= index {
                    result += 1;
                }
            }
        }
        result
    }

    fn conflicts_in(&self, vertex: &u32, index: usize) -> Vec<u32> {
        let incoming = self.graph.get_incoming(vertex);
        let mut result = Vec::with_capacity(incoming.len());
        for vertex in incoming {
            if let Some(other_index) = self.conf_vtoi[*vertex as usize] {
                if other_index >= index {
                    result.push(*vertex);
                }
            }
        }
        result
    }

    fn delta(&self, vertex: &u32, index: usize) -> i32 {
        self.conflicts_in_size(vertex, index) as i32 + self.conflicts_out_size(vertex, index) as i32
            - 1
    }

    fn random_move(&mut self) -> (u32, usize, bool) {
        let i = self.rng.gen_range(0..self.dfvs.len());
        let vertex = self.dfvs[i];
        let (m, is_in) = if self.rng.gen_bool(0.5) {
            (self.in_index(&vertex), true)
        } else {
            (self.out_index(&vertex), false)
        };

        (vertex, m, is_in)
    }

    fn apply_move(&mut self, vertex: u32, m: usize, is_in: bool) {
        self.dfvs.remove(&vertex);
        let to_remove = if is_in {
            self.conflicts_out(&vertex, m)
        } else {
            self.conflicts_in(&vertex, m)
        };

        let mut removed = false;
        for vertex in &to_remove {
            self.dfvs.insert(*vertex);
            if let Some(index) = self.conf_vtoi[*vertex as usize] {
                self.conf_itov[index] = None;
                removed = true;
            }
            self.conf_vtoi[*vertex as usize] = None;

            // for neighbor in self.graph.get_incoming(vertex) {
            //     self.in_cache[*neighbor as usize] = None;
            //     self.out_cache[*neighbor as usize] = None;
            // }

            // for neighbor in self.graph.get_outgoing(vertex) {
            //     self.in_cache[*neighbor as usize] = None;
            //     self.out_cache[*neighbor as usize] = None;
            // }
        }

        if m > self.conf_itov.len() {
            self.conf_itov.push(Some(vertex));
            self.conf_vtoi[vertex as usize] = Some(self.conf_itov.len() - 1);
        } else {
            self.conf_itov.insert(m, Some(vertex));
            self.conf_vtoi[vertex as usize] = Some(m);
        }

        if removed {
            let mut last = 0;
            for i in 0..self.conf_itov.len() {
                if self.conf_itov[i] != None {
                    if last != i {
                        let vertex = self.conf_itov[i].unwrap();
                        self.conf_itov[last] = self.conf_itov[i];
                        self.conf_itov[i] = None;
                        self.conf_vtoi[vertex as usize] = Some(last);
                    }
                    last += 1;
                }
            }

            loop {
                if self.conf_itov[self.conf_itov.len() - 1] == None {
                    self.conf_itov.pop();
                } else {
                    break;
                }
            }
        }

        for i in 0..self.conf_itov.len() {
            let vertex = self.conf_itov[i].unwrap();
            self.conf_vtoi[vertex as usize] = Some(i);
        }
    }

    fn recover_complete_solution(&mut self, mut solution: Vec<u32>) -> Vec<u32> {
        for i in 0..solution.len() {
            let vertex = solution[i];
            let original = self.mapping[vertex as usize];
            solution[i] = original;
        }
        solution.append(&mut self.reduced);
        solution
    }

    fn upper_bound(&mut self, _graph: &Graph) -> Vec<u32> {
        if !self.graph.is_empty() {
            const TEMPERATURE: f64 = 0.6;
            const ALPHA: f64 = 0.99;
            const FAILS: usize = 50;
            let max_mvt = self.graph.total_vertices() * 5;

            let mut temp = TEMPERATURE;
            let mut nb_fail = 0;
            let mut best_len = self.graph.total_vertices();
            let mut best_solution = Vec::with_capacity(self.graph.total_vertices());

            let ud = Uniform::new(0., 1.);

            if let Some(max_iter) = self.iter {
                let mut iter = 0;
                'main_loop: loop {
                    let mut nb_mvt = 0;
                    let mut failure = true;
                    loop {
                        if iter == max_iter {
                            break 'main_loop;
                        }

                        let (vertex, m, is_in) = self.random_move();
                        let delta = self.delta(&vertex, m);
                        if delta <= 0 || f64::exp(-delta as f64 / temp) >= ud.sample(&mut self.rng)
                        {
                            self.apply_move(vertex, m, is_in);
                            nb_mvt += 1;

                            if self.dfvs.len() < best_len {
                                best_solution.clear();
                                for vertex in self.dfvs.iter() {
                                    best_solution.push(*vertex);
                                }
                                best_len = best_solution.len();
                                failure = false;
                            }
                        }
                        if nb_mvt == max_mvt {
                            break;
                        }
                        iter += 1;
                    }
                    if failure {
                        nb_fail += 1;
                    } else {
                        nb_fail = 0;
                    }
                    temp *= ALPHA;

                    if nb_fail == FAILS {
                        break;
                    }
                }
                best_solution = self.recover_complete_solution(best_solution);
                best_solution
            } else {
                loop {
                    let mut nb_mvt = 0;
                    let mut failure = true;
                    loop {
                        let (vertex, m, is_in) = self.random_move();
                        let delta = self.delta(&vertex, m);
                        if delta <= 0 || f64::exp(-delta as f64 / temp) >= ud.sample(&mut self.rng)
                        {
                            self.apply_move(vertex, m, is_in);
                            nb_mvt += 1;

                            if self.dfvs.len() < best_len {
                                best_solution.clear();
                                for vertex in self.dfvs.iter() {
                                    best_solution.push(*vertex);
                                }
                                best_len = best_solution.len();
                                failure = false;
                            }
                        }
                        if nb_mvt == max_mvt {
                            break;
                        }
                    }
                    if failure {
                        nb_fail += 1;
                    } else {
                        nb_fail = 0;
                    }
                    temp *= ALPHA;

                    if nb_fail == FAILS {
                        break;
                    }
                }
                best_solution = self.recover_complete_solution(best_solution);
                // With the current parameters, we are usually finding a minimal
                // solution anyways, even though we have no guarantee that it is a
                // minimal solution.
                // best_solution = make_minimal(&mut graph.clone(), best_solution);
                best_solution
            }
        } else {
            self.reduced.clone()
        }
    }
}

impl Heuristic for SimulatedAnnealing {
    fn upper_bound(graph: &Graph) -> Vec<u32> {
        let mut sa = SimulatedAnnealing::new(graph, true);

        sa.upper_bound(graph)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn min_index_test_001() {
        let mut graph = Graph::new(7);
        graph.add_arc(0, 3);
        graph.add_arc(1, 3);
        graph.add_arc(2, 3);
        graph.add_arc(3, 4);
        graph.add_arc(3, 5);
        graph.add_arc(3, 6);
        let mut sa = SimulatedAnnealing::new(&graph, false);
        sa.conf_vtoi[0] = Some(4);
        sa.conf_vtoi[1] = Some(12);
        sa.conf_vtoi[2] = Some(7);
        sa.conf_vtoi[4] = Some(10);
        sa.conf_vtoi[5] = Some(8);
        sa.conf_vtoi[6] = Some(2);

        assert_eq!(sa.out_index(&3), 2);
        assert_eq!(sa.in_index(&3), 13);
    }

    #[test]
    fn conflicts_test_001() {
        let mut graph = Graph::new(2);
        graph.add_arc(0, 1);
        graph.add_arc(1, 0);
        let mut sa = SimulatedAnnealing::new(&graph, false);
        sa.upper_bound(&graph);
    }
}
