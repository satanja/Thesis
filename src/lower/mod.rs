//! Module to compute lower bounds for the graph
use crate::graph::Graph;
mod grb_rilp;
// mod cycle_rilp;
// mod ecc_rilp;
// mod vc_rilp;
// mod vcsr_rilp;

pub fn lower_bound(gd: &Graph, gb: &Graph) -> (usize, Vec<u32>) {
    let (rlb, candidate) = raw_lower_bound(gd, gb);
    ((rlb - 1e-5).ceil() as usize, candidate)
}

pub fn raw_lower_bound(gd: &Graph, gb: &Graph) -> (f64, Vec<u32>) {
    grb_rilp::lower_bound(gd, gb)
}
