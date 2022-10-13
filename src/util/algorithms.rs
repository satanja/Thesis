use std::cmp::Ordering;

/// Computes the set difference `lhs` \ `rhs`,  in time *O(n + m)*
/// where *n =* `lhs.len()` and *m =* `rhs.len()`. Assumes each element
/// in `lhs` and `rhs` is distinct and `lhs` and `rhs` are in ascending order.
pub fn difference(lhs: &[u32], rhs: &[u32]) -> Vec<u32> {
    let size = std::cmp::max(lhs.len(), rhs.len());
    let mut result = Vec::with_capacity(size);

    let mut ptr_lhs = 0;
    let mut ptr_rhs = 0;

    while ptr_lhs < lhs.len() && ptr_rhs < rhs.len() {
        match lhs[ptr_lhs].cmp(&rhs[ptr_rhs]) {
            Ordering::Equal => {
                ptr_lhs += 1;
                ptr_rhs += 1;
            }
            Ordering::Less => {
                result.push(lhs[ptr_lhs]);
                ptr_lhs += 1;
            }
            _ => ptr_rhs += 1,
        }
    }

    for i in ptr_lhs..lhs.len() {
        result.push(lhs[i]);
    }
    result.shrink_to_fit();
    result
}

/// Computes the set intersection between `lhs` and `rhs` in time *O(n + m)*
/// where *n =* `lhs.len()` and *m =* `rhs.len()`. Assumes each element
/// in `lhs` and `rhs` is distinct and `lhs` and `rhs` are in ascending order.
pub fn intersection(lhs: &[u32], rhs: &[u32]) -> Vec<u32> {
    let size = std::cmp::max(lhs.len(), rhs.len());
    let mut result = Vec::with_capacity(size);

    let mut ptr_lhs = 0;
    let mut ptr_rhs = 0;

    while ptr_lhs < lhs.len() && ptr_rhs < rhs.len() {
        match lhs[ptr_lhs].cmp(&rhs[ptr_rhs]) {
            Ordering::Equal => {
                result.push(lhs[ptr_lhs]);
                ptr_lhs += 1;
                ptr_rhs += 1;
            }
            Ordering::Less => ptr_lhs += 1,
            _ => ptr_rhs += 1,
        }
    }
    result.shrink_to_fit();
    result
}
