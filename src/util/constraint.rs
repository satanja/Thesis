#[derive(Clone, Default)]
pub struct Constraint {
    variables: Vec<u32>,
    lower_bound: u32,
}

impl Constraint {
    pub fn new(variables: Vec<u32>, lower_bound: u32) -> Constraint {
        Constraint {
            variables,
            lower_bound,
        }
    }

    pub fn variables(&self) -> &[u32] {
        &self.variables
    }

    /// Get the constraint's lower bound.
    pub fn lower_bound(&self) -> u32 {
        self.lower_bound
    }
}
