use crate::util::{Constraint, RangeSet};
use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
    Rng, SeedableRng,
};
use rustc_hash::FxHashSet;

struct SimulatedAnnealingHS {
    constraints: Vec<Constraint>,
    adj: Vec<Vec<usize>>,
    states: Vec<bool>,
    rng: StdRng,
    satisfied: RangeSet,
}

impl SimulatedAnnealingHS {
    fn new(constraints: &Vec<Constraint>, variables: usize) -> SimulatedAnnealingHS {
        let mut adj = vec![Vec::new(); variables];
        for i in 0..constraints.len() {
            let constraint = &constraints[i];
            for variable in constraint.variables() {
                adj[*variable as usize].push(i);
            }
        }

        let unsatisfied: Vec<_> = (0..constraints.len()).collect();
        let candidate_variables: Vec<_> = (0..variables as u32).collect();

        let initial_solution =
            SimulatedAnnealingHS::simple_greedy(&unsatisfied, &candidate_variables, &adj);

        let mut states = vec![false; variables];
        let mut satisfied = RangeSet::new(variables);
        for variable in &initial_solution {
            satisfied.insert(*variable);
            states[*variable as usize] = true;
        }

        SimulatedAnnealingHS {
            constraints: constraints.clone(),
            adj,
            states,
            rng: StdRng::seed_from_u64(0),
            satisfied,
        }
    }

    /// Randomly picks a satisfied variable to set to false
    fn random_move(&mut self) -> u32 {
        let i = self.rng.gen_range(0..self.satisfied.len());
        self.satisfied[i]
    }

    /// After flipping `variable` retrieve all constraints that have become
    /// unsatisfied
    fn get_unsatisfied(&self, variable: u32) -> Vec<usize> {
        let mut invalid = Vec::new();
        for i in &self.adj[variable as usize] {
            if !self.constraint_is_satisfied(*i) {
                invalid.push(*i);
            }
        }
        invalid
    }

    /// Determine whether a constraint is satisfied or not
    fn constraint_is_satisfied(&self, constraint_index: usize) -> bool {
        let constraint = &self.constraints[constraint_index];
        let mut sum = 0;
        for variable in constraint.variables() {
            if self.states[*variable as usize] {
                sum += 1;
            }
        }
        sum >= constraint.lower_bound()
    }

    /// Temporarily flips a variable, and computes a set of variables to also
    /// flip to satisfy the ILP again, or `Delta::Infeasible` if there does not
    /// exist such a set.  
    fn delta(&mut self, variable: u32) -> (i32, Option<Vec<u32>>) {
        self.flip_variable(variable);
        let unsatisfied = self.get_unsatisfied(variable);

        if unsatisfied.is_empty() {
            self.flip_variable(variable);
            return (-1, None);
        }

        // Simple greedy heuristic as a first implementation
        let candidate_variables = self.get_candidate_variables(&unsatisfied, variable);
        let mut covered_variables = FxHashSet::default();
        let mut counts: FxHashSet<_> = unsatisfied.into_iter().collect();

        while !counts.is_empty() {
            let mut max_variable = candidate_variables[0];
            let mut max_hit = 0;

            // Determine the variable that hits the most unsatisfied constraints
            for variable in &candidate_variables {
                // Skip variables that have already been included
                if covered_variables.contains(variable) {
                    continue;
                }

                let mut hit = 0;
                for j in &self.adj[*variable as usize] {
                    if counts.contains(j) {
                        hit += 1;
                    }
                }

                if hit > max_hit {
                    max_variable = *variable;
                    max_hit = hit;
                }
            }
            // Include the variable in the solution to fix and remove the set
            // of constraints it hits (if the constraint is then satisfied)
            covered_variables.insert(max_variable);
            for constraint in &self.adj[max_variable as usize] {
                counts.remove(&constraint);
            }
        }

        self.flip_variable(variable);

        let to_fix: Vec<_> = covered_variables.into_iter().collect();
        let delta = self.delta_to_repair(&to_fix, variable);
        (delta, Some(to_fix))
    }

    fn simple_greedy(
        unsatisfied: &[usize],
        candidate_variables: &[u32],
        adj: &Vec<Vec<usize>>,
    ) -> Vec<u32> {
        let mut covered_variables = vec![false; candidate_variables.len()];
        let mut unsatisfied_vec = vec![false; unsatisfied.len()];
        let mut num_unsatisfied = unsatisfied.len();
        let mut solution_size = 0;

        let mut last_max = 0;
        while num_unsatisfied != 0 {
            let mut max_variable = candidate_variables[0];
            let mut max_hit = 0;

            // Determine the variable that hits the most unsatisfied constraints
            for variable in candidate_variables {
                // Skip variables that have already been included
                if covered_variables[*variable as usize] {
                    continue;
                }

                let mut hit = 0;
                for j in &adj[*variable as usize] {
                    if !unsatisfied_vec[*j] {
                        hit += 1;
                    }
                }

                if hit > max_hit {
                    max_variable = *variable;
                    max_hit = hit;
                    if hit == last_max {
                        break;
                    }
                }
            }

            last_max = max_hit;

            // Include the variable in the solution to fix and remove the set
            // of constraints it hits (if the constraint is then satisfied)
            covered_variables[max_variable as usize] = true;
            solution_size += 1;
            for j in &adj[max_variable as usize] {
                if !unsatisfied_vec[*j] {
                    unsatisfied_vec[*j] = true;
                    num_unsatisfied -= 1;
                }
            }
        }
        let mut result = Vec::with_capacity(solution_size);
        for i in 0..covered_variables.len() {
            if covered_variables[i] {
                result.push(i as u32);
            }
        }
        result
    }

    /// Determines the cost of flipping a set of variables. Variables set to
    /// true decrease the cost by 1, variables set to false increase the cost
    /// by 1.
    fn delta_to_repair(&self, to_fix: &[u32], moved_variable: u32) -> i32 {
        let mut cost = 0;
        for variable in to_fix {
            if self.states[*variable as usize] {
                cost -= 1;
            } else {
                cost += 1;
            }
        }

        if self.states[moved_variable as usize] {
            cost -= 1;
        } else {
            cost += 1;
        }
        cost
    }

    /// Given a set of unsatisfied constraints, compute the set of variables
    /// that are set to false in the constraints.
    fn get_candidate_variables(&self, unsatisfied: &[usize], variable: u32) -> Vec<u32> {
        let mut set = FxHashSet::default();
        for constraint in unsatisfied {
            for v in self.constraints[*constraint].variables() {
                if variable != *v {
                    set.insert(*v);
                }
            }
        }
        set.into_iter().collect()
    }

    /// Flips a variable from 0 to 1, or from 1 to 0.
    fn flip_variable(&mut self, variable: u32) {
        self.states[variable as usize] ^= true;
    }

    /// Flips all variables in `to_fix`.
    fn flip_variables(&mut self, to_fix: &[u32]) {
        for variable in to_fix {
            self.flip_variable(*variable);
        }
    }

    /// Flips `variable` and all variables in `to_fix`, and fixes the set of
    /// satisfied variables
    fn apply_move(&mut self, variable: u32, to_fix: &[u32]) {
        self.flip_variable(variable);
        self.flip_variables(to_fix);

        self.satisfied.remove(&variable);
        for var in to_fix {
            self.satisfied.insert(*var);
        }
    }

    fn get_solution_len(&self) -> usize {
        self.satisfied.len()
    }

    /// Retrieves the current solution.
    fn get_solution(&self) -> Vec<u32> {
        let mut solution = Vec::new();
        for i in 0..self.states.len() {
            if self.states[i] {
                solution.push(i as u32);
            }
        }
        solution
    }
}

pub fn hitting_set_upper_bound(constraints: &Vec<Constraint>, variables: usize) -> Vec<u32> {
    hitting_set_upper_bound_custom(constraints, variables, 1_000_000)
}

pub fn hitting_set_upper_bound_custom(constraints: &Vec<Constraint>, variables: usize, iter: i32) -> Vec<u32> {
    let mut ilp = SimulatedAnnealingHS::new(constraints, variables);
    let mut best_solution: Vec<_> = (0..variables as u32).collect();
    let ud = Uniform::new(0., 1.);
    let mut temp = 5.;
    let end_temp = -1. * 1. / (1e-9f64.ln());
    let alpha = (end_temp / temp).powf(1. / iter as f64);
    
    for _ in 0..iter {
        let variable = ilp.random_move();
        let (delta, opt_to_fix) = ilp.delta(variable);
        if delta <= 0 || f64::exp(-delta as f64 / temp) >= ud.sample(&mut ilp.rng) {
            if let Some(to_fix) = opt_to_fix {
                ilp.apply_move(variable, &to_fix);
            } else {
                ilp.apply_move(variable, &[]);
            }

            if ilp.get_solution_len() < best_solution.len() {
                let new_solution = ilp.get_solution();
                best_solution = new_solution;
            }
        }
        temp *= alpha;
    }
    best_solution
}
