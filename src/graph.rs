use crate::util::{
    self,
    algorithms::{difference, intersection},
};
use core::fmt;
use rand::prelude::SliceRandom;
use rand::rngs::StdRng;
use rand::SeedableRng;
use rustc_hash::{FxHashMap, FxHashSet};
use std::{collections::VecDeque, fmt::Write, ops::Add};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Graph {
    /// The list of vertices which are deleted.
    deleted_vertices: Vec<bool>,

    // num_active_vertices: usize,
    coloring: Vec<Color>,

    /// The adjacency list representation of the graph.
    adj: Vec<Vec<u32>>,

    /// The adjacency list representation of the reversed graph.
    rev_adj: Vec<Vec<u32>>,

    /// Vertices which are forbidden to be reduced
    forbidden: Vec<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Color {
    Unvisited,
    Visited,
    Exhausted,
}

impl Graph {
    pub fn new(vertices: usize) -> Graph {
        let _num_active_vertices = vertices;
        let coloring = vec![Color::Unvisited; vertices];
        let adj = vec![Vec::new(); vertices];
        let rev_adj = vec![Vec::new(); vertices];
        Graph {
            deleted_vertices: vec![false; vertices],
            coloring,
            adj,
            rev_adj,
            // max_degree_heap: Heap::new(0),
            // current_out_degree: vec![0; vertices],
            // current_in_degree: vec![0; vertices],
            // sink_source_buffer: Vec::new(),
            // sinks_or_sources: RangeSet::new(vertices),
            forbidden: vec![false; vertices],
        }
    }

    pub fn add_arc(&mut self, source: u32, target: u32) {
        if let Err(index) = self.adj[source as usize].binary_search(&target) {
            self.adj[source as usize].insert(index, target);
        }
        if let Err(index) = self.rev_adj[target as usize].binary_search(&source) {
            self.rev_adj[target as usize].insert(index, source);
        }
    }

    pub fn remove_vertex(&mut self, vertex: u32) {
        self.deleted_vertices[vertex as usize] = true;
        let forward_list = std::mem::take(&mut self.adj[vertex as usize]);
        for next in forward_list {
            let index = self.rev_adj[next as usize].binary_search(&vertex).unwrap();
            self.rev_adj[next as usize].remove(index);
        }
        let backward_list = std::mem::take(&mut self.rev_adj[vertex as usize]);
        for source in backward_list {
            let index = self.adj[source as usize].binary_search(&vertex).unwrap();
            self.adj[source as usize].remove(index);
        }
    }

    pub fn remove_vertices(&mut self, vertices: &[u32]) {
        let mut affected_vertices_forward = FxHashSet::default();
        let mut affected_vertices_back = FxHashSet::default();
        for singleton in vertices {
            for target in &self.adj[*singleton as usize] {
                affected_vertices_forward.insert(*target);
            }
            for source in &self.rev_adj[*singleton as usize] {
                affected_vertices_back.insert(*source);
            }
        }
        for vertex in affected_vertices_forward {
            // u -> vertex, where u in vertices, so look at reverse adjacency list
            let list = std::mem::take(&mut self.rev_adj[vertex as usize]);
            let reduced = util::algorithms::difference(&list, vertices);
            self.rev_adj[vertex as usize] = reduced;
        }

        for vertex in affected_vertices_back {
            // vertex -> u, where u in vertices, so look at the forward adjacency list
            let list = std::mem::take(&mut self.adj[vertex as usize]);
            let reduced = util::algorithms::difference(&list, vertices);
            self.adj[vertex as usize] = reduced;
        }

        for vertex in vertices {
            self.adj[*vertex as usize].clear();
            self.rev_adj[*vertex as usize].clear();
            self.deleted_vertices[*vertex as usize] = true;
        }
    }

    // pub fn initialize_data_structures(&mut self) {
    //     // initialize the heaps
    //     let mut data = Vec::with_capacity(self.total_vertices());
    //     for vertex in 0..self.total_vertices() {
    //         self.current_in_degree[vertex] = self.rev_adj[vertex].len();
    //         let item = MaxItem::new(
    //             vertex,
    //             (self.adj[vertex].len() + self.rev_adj[vertex].len()) as i64,
    //         );
    //         data.push(item);
    //     }
    //     self.max_degree_heap.load(data);

    //     // already look for vertices that are sources or sinks
    //     for vertex in 0..self.total_vertices() {
    //         if self.current_in_degree[vertex] == 0 || self.current_out_degree[vertex] == 0 {
    //             self.sink_source_buffer.push(vertex as u32);
    //         }
    //     }
    // }

    pub fn set_adjacency(&mut self, source: u32, mut targets: Vec<u32>) {
        targets.sort_unstable();
        for vertex in &targets {
            self.rev_adj[*vertex as usize].push(source);
        }
        // self.current_out_degree[source as usize] = targets.len();
        self.adj[source as usize] = targets;
    }

    /// Returns the number of vertices in the original graph
    pub fn total_vertices(&self) -> usize {
        self.adj.len()
    }

    /// Returns the number of remaining vertices
    pub fn vertices(&self) -> usize {
        let mut n = 0;
        for i in 0..self.adj.len() {
            if !self.deleted_vertices[i] {
                n += 1;
            }
        }
        n
    }

    pub fn is_empty(&self) -> bool {
        for i in 0..self.adj.len() {
            if !self.deleted_vertices[i] {
                return false;
            }
        }
        true
    }

    // pub fn lower_bound(&self) -> usize {
    //     let stars = self.stars();
    //     debug_assert!(stars.len() % 2 == 0);
    //     stars.len() / 2
    // }

    // pub fn num_active_vertices(&self) -> usize {
    //     self.num_active_vertices
    // }

    // /// Requirement: only after disabling a vertex do data structures need to be
    // /// updated. Vertices disabled during kernelizations shall no longer be
    // /// enabled.
    // pub fn disable_vertex(&mut self, vertex: u32) {
    //     // remove from heaps
    //     self.max_degree_heap.decrease_key(MaxItem::new(
    //         vertex as usize,
    //         (self.total_vertices() + 1) as i64,
    //     ));

    //     let val = self.max_degree_heap.extract_min();
    //     debug_assert_eq!(val.unwrap().key() as u32, vertex);

    //     // update synposi
    //     self.active_vertices[vertex as usize] = false;
    //     self.coloring[vertex as usize] = Color::Exhausted;
    //     self.num_active_vertices -= 1;

    //     // update data structures for affected vertices
    //     for incoming in &self.rev_adj[vertex as usize] {
    //         if self.active_vertices[*incoming as usize] {
    //             self.current_out_degree[*incoming as usize] -= 1;

    //             if self.current_out_degree[*incoming as usize] == 0 {
    //                 self.sink_source_buffer.push(*incoming);
    //             }
    //         }
    //     }

    //     for outgoing in &self.adj[vertex as usize] {
    //         if self.active_vertices[*outgoing as usize] {
    //             self.current_in_degree[*outgoing as usize] -= 1;
    //             if self.current_in_degree[*outgoing as usize] == 0 {
    //                 self.sink_source_buffer.push(*outgoing);
    //             }
    //         }
    //     }

    //     for incoming in &self.rev_adj[vertex as usize] {
    //         if self.active_vertices[*incoming as usize] {
    //             let deg = self.current_out_degree[*incoming as usize]
    //                 + self.current_in_degree[*incoming as usize];
    //             self.max_degree_heap
    //                 .decrease_key(MaxItem::new(*incoming as usize, deg as i64));
    //         }
    //     }

    //     for outgoing in &self.adj[vertex as usize] {
    //         if self.active_vertices[*outgoing as usize] {
    //             let deg = self.current_out_degree[*outgoing as usize]
    //                 + self.current_in_degree[*outgoing as usize];
    //             self.max_degree_heap
    //                 .decrease_key(MaxItem::new(*outgoing as usize, deg as i64));
    //         }
    //     }
    // }

    // pub fn enable_vertex(&mut self, vertex: u32) {
    //     self.active_vertices[vertex as usize] = true;
    //     self.coloring[vertex as usize] = Color::Unvisited;
    //     self.num_active_vertices += 1;

    //     for incoming in &self.rev_adj[vertex as usize] {
    //         if self.active_vertices[*incoming as usize] {
    //             self.current_out_degree[*incoming as usize] += 1;
    //             let deg = self.current_out_degree[*incoming as usize];
    //             self.max_degree_heap
    //                 .decrease_key(MaxItem::new(*incoming as usize, deg as i64));
    //         }
    //     }
    // }

    pub fn disable_vertex_post(&mut self, vertex: u32) {
        self.deleted_vertices[vertex as usize] = true;
        self.coloring[vertex as usize] = Color::Exhausted;
    }

    pub fn enable_vertex_post(&mut self, vertex: u32) {
        self.deleted_vertices[vertex as usize] = false;
        self.coloring[vertex as usize] = Color::Unvisited;
    }

    pub fn get_active_vertices(&self) -> Vec<u32> {
        let mut result = Vec::new();
        for i in 0..self.total_vertices() {
            if !self.deleted_vertices[i] {
                result.push(i as u32);
            }
        }
        result
    }

    pub fn get_outgoing(&self, vertex: &u32) -> &[u32] {
        &self.adj[*vertex as usize]
    }

    pub fn get_incoming(&self, vertex: &u32) -> &[u32] {
        &self.rev_adj[*vertex as usize]
    }

    // pub fn get_disabled_vertices(&self) -> Vec<u32> {
    //     let mut result = Vec::new();
    //     for i in 0..self.total_vertices() {
    //         if !self.active_vertices[i] {
    //             result.push(i as u32)
    //         }
    //     }
    //     result
    // }

    /// Helper algorithm to find cycles in the directed graph.
    fn visit(&self, vertex: usize, coloring: &mut Vec<Color>) -> bool {
        if coloring[vertex] == Color::Exhausted {
            return false;
        }
        if coloring[vertex] == Color::Visited {
            return true;
        }

        coloring[vertex] = Color::Visited;

        for next in &self.adj[vertex] {
            if self.visit(*next as usize, coloring) {
                return true;
            }
        }

        coloring[vertex] = Color::Exhausted;
        false
    }

    /// Test whether the graph has a cycle. Simple DFS implementation based on
    /// computing a topological ordering. The graph may consist of several
    /// connected components.
    pub fn is_cyclic(&self) -> bool {
        let mut local_coloring = self.coloring.clone();
        for i in 0..self.total_vertices() {
            if self.deleted_vertices[i] {
                continue;
            }
            if local_coloring[i] == Color::Unvisited && self.visit(i, &mut local_coloring) {
                return true;
            }
        }
        false
    }

    /// Given a set `fvs` of vertices to delete, returns `false` if the
    /// remainder has a cycle somewhere, returns true otherwise.
    ///  Simple DFS implementation based on
    /// computing a topological ordering. The graph may consist of several
    /// connected components.
    pub fn is_acyclic_with_fvs(&self, fvs: &[u32]) -> bool {
        // keep track of which vertices have been exhaustively visited
        let mut coloring: Vec<_> = vec![Color::Unvisited; self.total_vertices()];
        for vertex in fvs {
            coloring[*vertex as usize] = Color::Exhausted;
        }

        for i in 0..self.total_vertices() {
            if self.deleted_vertices[i] {
                continue;
            }
            if coloring[i] == Color::Unvisited && self.visit(i, &mut coloring) {
                return false;
            }
        }
        true
    }

    fn recover_cycle(vertex: u32, dest: u32, pred: &Vec<Option<u32>>) -> Vec<u32> {
        let mut path = Vec::new();
        let mut current_vertex = vertex;
        while current_vertex != dest {
            path.push(current_vertex);
            if pred[current_vertex as usize] == None {
                println!("shit");
            }
            current_vertex = pred[current_vertex as usize].unwrap();
        }
        path.push(dest);
        path.reverse();
        path
    }

    fn dfs_find_cycle(
        &self,
        vertex: usize,
        coloring: &mut Vec<Color>,
        pred: &mut Vec<Option<u32>>,
    ) -> Option<Vec<u32>> {
        if coloring[vertex] == Color::Exhausted {
            return None;
        }

        coloring[vertex] = Color::Visited;

        for next in &self.adj[vertex] {
            if coloring[*next as usize] == Color::Visited {
                return Some(Graph::recover_cycle(vertex as u32, *next, pred));
            }

            pred[*next as usize] = Some(vertex as u32);
            if let Some(cycle) = self.dfs_find_cycle(*next as usize, coloring, pred) {
                return Some(cycle);
            }
        }

        coloring[vertex] = Color::Exhausted;
        None
    }

    pub fn find_cycle_with_fvs(&self, fvs: &[u32]) -> Option<Vec<u32>> {
        let mut coloring = vec![Color::Unvisited; self.total_vertices()];
        let mut pred: Vec<Option<u32>> = vec![None; self.total_vertices()];

        for vertex in fvs {
            coloring[*vertex as usize] = Color::Exhausted;
        }

        for i in 0..self.total_vertices() {
            if self.deleted_vertices[i] {
                continue;
            }
            if coloring[i] == Color::Unvisited {
                if let Some(cycle) = self.dfs_find_cycle(i, &mut coloring, &mut pred) {
                    return Some(cycle);
                }
            }
        }
        None
    }

    pub fn find_cycle_from_minimal(&self, minimal: &[u32]) -> Vec<Vec<u32>> {
        let mut coloring = vec![Color::Unvisited; self.total_vertices()];
        let mut pred: Vec<Option<u32>> = vec![None; self.total_vertices()];

        for vertex in minimal {
            coloring[*vertex as usize] = Color::Exhausted;
        }

        let minimal_set: FxHashSet<_> = minimal.iter().collect();

        let mut cycles = Vec::new();
        for vertex in minimal {
            coloring[*vertex as usize] = Color::Unvisited; // G now contains a cycle, and all its cycles contain `vertex`

            let cycle = self
                .dfs_find_cycle(*vertex as usize, &mut coloring, &mut pred)
                .unwrap();

            cycles.push(cycle);

            for v in 0..coloring.len() {
                if !minimal_set.contains(&(v as u32)) {
                    coloring[v] = Color::Unvisited;
                }
            }
            coloring[*vertex as usize] = Color::Exhausted;

            for entry in &mut pred {
                *entry = None;
            }
        }

        cycles
    }

    // Can be optimized using a heap. Each time a vertex is disabled, we can
    // update the number of in and outgoing edges.
    pub fn max_degree_vertex(&mut self) -> u32 {
        let mut max_deg = 0;
        let mut max_vertex = 0;
        for vertex in 0..self.total_vertices() {
            if !self.deleted_vertices[vertex] {
                let deg_out = self.adj[vertex]
                    .iter()
                    .filter(|u| !self.deleted_vertices[**u as usize])
                    .fold(0, |acc, _| acc + 1) as u32;

                let deg_in = self.rev_adj[vertex]
                    .iter()
                    .filter(|u| !self.deleted_vertices[**u as usize])
                    .fold(0, |acc, _| acc + 1) as u32;

                if deg_in + deg_out > max_deg {
                    max_deg = deg_in + deg_out;
                    max_vertex = vertex;
                }
            }
        }

        max_vertex as u32
    }

    fn strong_connect(
        &self,
        vertex: usize,
        stack: &mut Vec<usize>,
        index_vec: &mut Vec<i32>,
        index: &mut i32,
        low: &mut Vec<i32>,
        comp: &mut Vec<i32>,
        components: &mut i32,
    ) {
        let mut work_stack = vec![(vertex, 0)];
        while let Some((u, j)) = work_stack.pop() {
            if j == 0 {
                index_vec[u] = *index;
                *index += 1;
                low[u] = index_vec[u];
                stack.push(u);
            }
            let mut recurse = false;
            for i in j as usize..self.adj[u].len() {
                let next = self.adj[u][i];
                if index_vec[next as usize] == -1 {
                    work_stack.push((u, i + 1));
                    work_stack.push((next as usize, 0));
                    recurse = true;
                    break;
                } else if comp[next as usize] == -1 {
                    low[u] = std::cmp::min(low[u], index_vec[next as usize]);
                }
            }
            if !recurse {
                if low[u] == index_vec[u] {
                    while let Some(prev) = stack.pop() {
                        comp[prev] = *components;
                        if prev == u {
                            break;
                        }
                    }
                    *components += 1;
                }
                if !work_stack.is_empty() {
                    let (up, _) = work_stack.last().unwrap();
                    low[*up] = std::cmp::min(low[*up], low[u]);
                }
            }
        }
    }

    pub fn tarjan(&self, always_report: bool) -> Option<Vec<Vec<u32>>> {
        let mut index_vec = vec![-1; self.total_vertices()];
        let mut index = 0;
        let mut low = vec![0; self.total_vertices()];
        let mut comp = vec![-1; self.total_vertices()];
        let mut stack = Vec::new();
        let mut components = 0;

        let mut modified = always_report;
        for vertex in 0..self.total_vertices() {
            if index_vec[vertex] == -1 && !self.deleted_vertices[vertex] {
                modified = true;
                self.strong_connect(
                    vertex,
                    &mut stack,
                    &mut index_vec,
                    &mut index,
                    &mut low,
                    &mut comp,
                    &mut components,
                )
            }
        }

        if modified {
            let mut partition = vec![Vec::new(); components as usize];
            for vertex in 0..self.total_vertices() {
                if self.deleted_vertices[vertex] {
                    continue;
                }
                partition[comp[vertex] as usize].push(vertex as u32);
            }
            Some(partition)
        } else {
            None
        }
    }

    fn scc_reduction(&mut self) -> bool {
        let res = self.tarjan(false);
        if res == None {
            return false;
        }

        // only 1 SSC => no point in continuing the reduction
        let components = res.unwrap();
        if components.len() == 1 {
            return false;
        }

        let mut result = false;
        let mut singletons = Vec::new();
        for component in &components {
            if component.len() == 1 {
                let vertex = component[0];
                // may incorrectly pick up self loops
                if !self.adj[vertex as usize].contains(&vertex) {
                    result = true;
                    singletons.push(component[0]);
                }
            }
        }
        singletons.sort_unstable();
        self.remove_vertices(&singletons);

        // compute the induced graph by parts of the strongly connected components
        // SCCs may share edges, but they're irrelevant
        let mut removed_edges = false;
        for component in components {
            // we can skip all singletons, some may have self-loops, others are already removed
            if component.len() == 1 {
                continue;
            }

            for vertex in &component {
                let new_adj = intersection(&self.adj[*vertex as usize], &component);
                if new_adj.len() != self.adj[*vertex as usize].len() {
                    removed_edges = true;
                    result = true;
                    self.adj[*vertex as usize] = new_adj;
                }
            }
        }

        // only change the reverse adjacency list if actual progress has been
        // made
        if removed_edges {
            // rebuild reverse adj
            self.rev_adj = vec![Vec::new(); self.total_vertices()];
            for i in 0..self.adj.len() {
                for j in 0..self.adj[i].len() {
                    let target = self.adj[i][j];
                    self.rev_adj[target as usize].push(i as u32);
                }
            }

            for rev_list in &mut self.rev_adj {
                rev_list.sort_unstable();
            }
        }
        result
    }

    fn has_self_loop(&self) -> bool {
        for i in 0..self.adj.len() {
            if self.adj[i].contains(&(i as u32)) {
                return true;
            }
        }
        false
    }

    fn self_loop_reduction(&mut self) -> Vec<u32> {
        let mut forced = Vec::new();
        for i in 0..self.adj.len() {
            if self.adj[i].contains(&(i as u32)) {
                forced.push(i as u32);
                self.remove_vertex(i as u32);
            }
        }
        forced
    }

    fn has_single_outgoing(&self) -> bool {
        for i in 0..self.adj.len() {
            let list = &self.adj[i];
            let allowed = !self.deleted_vertices[i] && !self.forbidden[i];
            if list.len() == 1 && allowed && list[0] != i as u32 {
                return true;
            }
        }
        false
    }

    fn has_single_incoming(&self) -> bool {
        for i in 0..self.rev_adj.len() {
            let list = &self.rev_adj[i];
            let allowed = !self.deleted_vertices[i] && !self.forbidden[i];
            if list.len() == 1 && allowed && list[0] != i as u32 {
                return true;
            }
        }
        false
    }

    fn single_incoming_reduction(&mut self) {
        for i in 0..self.rev_adj.len() {
            let list = &self.rev_adj[i];
            let allowed = !self.deleted_vertices[i] && !self.forbidden[i];
            if list.len() == 1 && allowed {
                let source = *list.first().unwrap();
                if source == i as u32 {
                    // self-loop
                    continue;
                }

                // mark the vertex as deleted
                self.deleted_vertices[i] = true;

                // get the targets
                let nexts = self.adj[i].clone();

                // already erase the adjacency list
                self.adj[i].clear();

                // vertex i is located in the forward adjacency list of the
                // source
                let index = self.adj[source as usize]
                    .binary_search(&(i as u32))
                    .unwrap();
                self.adj[source as usize].remove(index);

                // redirect edges
                for next in nexts {
                    let v_index = self.rev_adj[next as usize]
                        .binary_search(&(i as u32))
                        .unwrap();
                    self.rev_adj[next as usize].remove(v_index);
                    self.add_arc(source, next);
                }
                self.rev_adj[i].clear();
            }
        }
    }

    fn single_outgoing_reduction(&mut self) {
        for i in 0..self.adj.len() {
            let list = &self.adj[i];
            let allowed = !self.deleted_vertices[i] && !self.forbidden[i];
            if list.len() == 1 && allowed {
                // get the single target
                let target = *list.first().unwrap();
                if target == i as u32 {
                    // self-loop
                    continue;
                }
                // mark the vertex as deleted
                self.deleted_vertices[i] = true;

                // get the sources & clear
                let sources = self.rev_adj[i].clone();
                self.rev_adj[i].clear();
                let index = self.rev_adj[target as usize]
                    .binary_search(&(i as u32))
                    .unwrap();
                self.rev_adj[target as usize].remove(index);

                for source in sources {
                    let v_index = self.adj[source as usize]
                        .binary_search(&(i as u32))
                        .unwrap();
                    self.adj[source as usize].remove(v_index);
                    self.add_arc(source, target);
                }
                self.adj[i].clear();
            }
        }
    }

    fn has_empty_vertex(&self) -> bool {
        for i in 0..self.adj.len() {
            if self.adj[i].is_empty() && self.rev_adj[i].is_empty() && !self.deleted_vertices[i] {
                return true;
            }
        }
        false
    }

    fn empty_vertices(&mut self) {
        for i in 0..self.adj.len() {
            if self.adj[i].is_empty() && self.rev_adj[i].is_empty() && !self.deleted_vertices[i] {
                self.deleted_vertices[i] = true;
            }
        }
    }

    /// Finds vertices contained in a 2-cycle and all its neighbors that are
    /// included in the 2-cycles
    pub fn stars(&self) -> Vec<(u32, Vec<u32>)> {
        let mut count = vec![Vec::new(); self.total_vertices()];
        for i in 0..self.total_vertices() {
            if self.deleted_vertices[i] {
                continue;
            }
            for j in 0..self.adj[i].len() {
                let t = self.adj[i][j];
                debug_assert!(!self.deleted_vertices[t as usize]);
                if self.adj[t as usize].contains(&(i as u32)) {
                    count[i].push(t);
                }
            }
        }
        let mut stars = Vec::new();
        for i in 0..count.len() {
            if !count[i].is_empty() {
                let neighborhood = std::mem::take(&mut count[i]);
                stars.push((i as u32, neighborhood));
            }
        }
        stars
    }

    /// Returns all vertices that have an undirected edge to another vertex
    fn single_stars(&self) -> Vec<u32> {
        let mut single_stars = Vec::new();
        for i in 0..self.total_vertices() {
            if self.deleted_vertices[i] {
                continue;
            }
            for j in 0..self.adj[i].len() {
                let t = self.adj[i][j];
                if self.adj[t as usize].contains(&(i as u32)) {
                    single_stars.push(i as u32);
                    break;
                }
            }
        }
        single_stars
    }

    fn undirected_components(&self) -> Vec<Vec<u32>> {
        let single_stars = self.single_stars();

        let mut map = FxHashMap::default();
        for i in 0..single_stars.len() {
            map.insert(single_stars[i], i);
        }

        let mut discovered = vec![false; single_stars.len()];
        let mut components = Vec::new();
        for i in 0..single_stars.len() {
            if discovered[i] {
                continue;
            }

            let mut queue = VecDeque::new();
            let mut component = Vec::new();
            queue.push_back(single_stars[i]);
            discovered[i] = true;

            while let Some(vertex) = queue.pop_front() {
                component.push(vertex);

                for u in &self.adj[vertex as usize] {
                    if let Some(index) = map.get(u) {
                        if self.adj[*u as usize].contains(&vertex) {
                            // vertex u has an undirected edge
                            if !discovered[*index] {
                                discovered[*index] = true;
                                queue.push_back(*u);
                            }
                        }
                    }
                }
            }
            component.sort_unstable();
            components.push(component);
        }
        components
    }

    fn contract_component_to_vertex(&mut self, component: &[u32]) {
        let mut leaving = FxHashSet::default();
        let mut entering = FxHashSet::default();
        let component_set: FxHashSet<_> = component.iter().copied().collect();
        for vertex in component {
            for target in &self.adj[*vertex as usize] {
                if !component_set.contains(target) {
                    leaving.insert(*target);
                }
            }
            for source in &self.rev_adj[*vertex as usize] {
                if !component_set.contains(source) {
                    entering.insert(*source);
                }
            }
        }

        let first = component[0];
        self.remove_vertices(component);

        for vertex in leaving {
            self.add_arc(first, vertex);
        }
        for vertex in entering {
            self.add_arc(vertex, first);
        }
        self.deleted_vertices[first as usize] = false;
    }

    pub fn max_degree_star(&self) -> Option<(u32, Vec<u32>)> {
        let mut stars = self.stars();
        if stars.is_empty() {
            return None;
        }

        let mut max = stars.pop().unwrap();
        for star in stars {
            if star.1.len() >= max.1.len() {
                max = star;
            }
        }

        Some(max)
    }

    fn star_reduction(&mut self, parameter: usize) -> Option<u32> {
        let stars = self.stars();
        if stars.is_empty() {
            return None;
        }
        let (v, neighbours) = stars.last().unwrap();
        if neighbours.len() > parameter {
            self.remove_vertex(*v);
            return Some(*v);
        }
        None
    }

    fn twin_reduction(&mut self) -> Vec<u32> {
        let mut classes: FxHashMap<Vec<u32>, Vec<u32>> = FxHashMap::default();
        let mut forced = Vec::new();

        let mut has_twins = false;
        for i in 0..self.adj.len() {
            if self.deleted_vertices[i] {
                continue;
            }

            let mut list = self.adj[i].clone();
            // closed neighborhood
            list.push(i as u32);
            list.sort_unstable();

            if let Some(class) = classes.get_mut(&list) {
                class.push(i as u32);
                has_twins = true;
            } else {
                classes.insert(list, vec![i as u32]);
            }
        }

        if has_twins {
            for (_, twins) in classes {
                if twins.len() == 1 {
                    continue;
                }
                let twin_set: FxHashSet<_> = twins.iter().copied().collect();
                let first = twins[0];

                // we use the fact that all vertices see the same neighbours outside twins
                let l = difference(&self.adj[first as usize], &twins).len();
                let mut undir_edges = 0;
                let mut dir_edges = twins.len() * l;

                // Reduction rule is only applicable if the cut (T, N(T)\T) has
                // only undirected or directed edges, but not both.
                if l != 0 {
                    for vertex in &twins {
                        for source in &self.rev_adj[*vertex as usize] {
                            if !twin_set.contains(source)
                                && self.adj[*vertex as usize].contains(source)
                            {
                                undir_edges += 1;
                                dir_edges -= 1;
                            }
                        }
                    }
                }

                if undir_edges == 0 || dir_edges == 0 {
                    let mut filtered = Vec::with_capacity(twins.len());
                    for vertex in &twins {
                        if !self.forbidden[*vertex as usize] {
                            filtered.push(*vertex);
                        }
                    }

                    if filtered.len() == twins.len() {
                        filtered.pop();
                    }

                    self.remove_vertices(&filtered);
                    forced.append(&mut filtered);
                }
            }
        }
        forced
    }

    pub fn induced_subgraph(&self, mut subset: Vec<u32>) -> Graph {
        subset.sort_unstable();
        let mut induced = self.clone();

        for i in 0..induced.total_vertices() {
            if !subset.contains(&(i as u32)) {
                induced.adj[i].clear();
                induced.deleted_vertices[i] = true;
                continue;
            }

            let intersect_adj = intersection(&induced.adj[i], &subset);
            let intersect_rev_adj = intersection(&induced.rev_adj[i], &subset);

            induced.adj[i] = intersect_adj;
            induced.rev_adj[i] = intersect_rev_adj;
        }

        induced
    }

    pub fn remove_undirected_edges(&mut self, stars: Vec<(u32, Vec<u32>)>) {
        for (source, neighbors) in stars {
            let red_source_adj = difference(&self.adj[source as usize], &neighbors);
            self.adj[source as usize] = red_source_adj;

            let red_source_rev_adj = difference(&self.rev_adj[source as usize], &neighbors);
            self.rev_adj[source as usize] = red_source_rev_adj;

            for neighbor in neighbors {
                for i in 0..self.adj[neighbor as usize].len() {
                    if self.adj[neighbor as usize][i] == source {
                        self.adj[neighbor as usize].remove(i);
                        break;
                    }
                }

                for i in 0..self.rev_adj[neighbor as usize].len() {
                    if self.rev_adj[neighbor as usize][i] == source {
                        self.rev_adj[neighbor as usize].remove(i);
                        break;
                    }
                }
            }
        }
    }

    pub fn mark_forbidden(&mut self, vertices: &[u32]) {
        for vertex in vertices {
            self.forbidden[*vertex as usize] = true;
        }
    }
}

pub trait Reducable {
    fn reduce(&mut self, upper_bound: usize) -> Option<Vec<u32>>;
}

impl Reducable for Graph {
    fn reduce(&mut self, mut upper_bound: usize) -> Option<Vec<u32>> {
        let mut reduced = true;
        let mut forced = Vec::new();
        while reduced {
            reduced = false;
            if self.scc_reduction() {
                reduced = true;
            }

            if self.has_empty_vertex() {
                self.empty_vertices();
                reduced = true;
            }

            if self.has_single_outgoing() {
                self.single_outgoing_reduction();
                reduced = true;
                continue;
            }
            if self.has_single_incoming() {
                self.single_incoming_reduction();
                reduced = true;
                continue;
            }

            upper_bound = std::cmp::min(upper_bound, self.vertices());

            if self.has_self_loop() {
                let mut self_loops = self.self_loop_reduction();
                if self_loops.len() > upper_bound {
                    return None;
                }
                reduced = true;
                upper_bound -= self_loops.len();
                forced.append(&mut self_loops);
                continue;
            }
        }
        Some(forced)
    }
}

pub trait HeuristicReduce {
    fn reduce(&mut self) -> Vec<u32>;
}

impl HeuristicReduce for Graph {
    fn reduce(&mut self) -> Vec<u32> {
        let mut reduced = true;
        let mut forced = Vec::new();
        while reduced {
            reduced = false;
            if self.scc_reduction() {
                reduced = true;
            }

            if self.has_empty_vertex() {
                self.empty_vertices();
                reduced = true;
            }

            if self.has_single_outgoing() {
                self.single_outgoing_reduction();
                reduced = true;
                continue;
            }
            if self.has_single_incoming() {
                self.single_incoming_reduction();
                reduced = true;
                continue;
            }

            if self.has_self_loop() {
                let mut self_loops = self.self_loop_reduction();
                reduced = true;
                forced.append(&mut self_loops);
                continue;
            }
        }
        forced
    }
}

impl fmt::Display for Graph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} 0 0", self.total_vertices())?;
        for list in &self.adj {
            let mut first = true;
            for i in 0..list.len() {
                if !self.deleted_vertices[list[i] as usize] {
                    if first {
                        write!(f, "{}", list[i] + 1)?;
                        first = false;
                    } else {
                        write!(f, " {}", list[i] + 1)?;
                    }
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
pub trait Undirected {
    fn is_undirected(&self) -> bool;
    fn as_string(&self) -> String;
}

impl Undirected for Graph {
    fn is_undirected(&self) -> bool {
        for i in 0..self.adj.len() {
            for j in 0..self.adj[i].len() {
                let u = self.adj[i][j];
                if !self.rev_adj[i].contains(&u) {
                    return false;
                }
            }
        }
        true
    }

    fn as_string(&self) -> String {
        let mut output = String::new();
        let directed_edges = self.adj.iter().fold(0, |x, list| x + list.len());
        let edges = directed_edges / 2;

        writeln!(output, "p td {} {}", self.total_vertices(), edges).unwrap();
        let edges = self.undir_edge_iter();
        for (u, v) in edges {
            writeln!(output, "{} {}", u + 1, v + 1).unwrap();
        }

        output
    }
}

pub trait EdgeIter {
    fn undir_edge_iter(&self) -> UndirEdgeIter;
}

impl EdgeIter for Graph {
    fn undir_edge_iter(&self) -> UndirEdgeIter {
        UndirEdgeIter {
            current_vertex: 0,
            current_neighbor: 0,
            graph: self,
        }
    }
}

pub struct UndirEdgeIter<'a> {
    current_vertex: usize,
    current_neighbor: usize,
    graph: &'a Graph,
}

impl<'a> Iterator for UndirEdgeIter<'a> {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        for i in self.current_vertex..self.graph.total_vertices() {
            for j in self.current_neighbor..self.graph.adj[i].len() {
                let u = self.graph.adj[i][j];
                if i < u as usize {
                    self.current_vertex = i;
                    self.current_neighbor = j + 1;
                    return Some((i as u32, u));
                }
            }
            self.current_neighbor = 0;
        }
        None
    }
}

pub trait Compressor {
    fn compress(&self) -> (Graph, Vec<u32>);
}

impl Compressor for Graph {
    fn compress(&self) -> (Graph, Vec<u32>) {
        let mut map = FxHashMap::default();
        let mut adj_map = FxHashMap::default();
        for i in 0..self.total_vertices() {
            if self.deleted_vertices[i] || self.adj[i].is_empty() {
                continue;
            }

            let index;
            if !map.contains_key(&(i as u32)) {
                let len = map.len() as u32;
                index = len;
                map.insert(i as u32, len);
            } else {
                index = *map.get(&(i as u32)).unwrap();
            }

            let mut neighbors = Vec::with_capacity(self.adj[i].len());
            for j in 0..self.adj[i].len() {
                let target = self.adj[i][j];
                if let Some(k) = map.get(&target) {
                    neighbors.push(*k);
                } else {
                    let len = map.len() as u32;
                    neighbors.push(len);
                    map.insert(target, len);
                }
            }
            adj_map.insert(index, neighbors);
        }

        let mut graph = Graph::new(map.len());
        for (source, neighbors) in adj_map {
            graph.set_adjacency(source, neighbors);
        }

        let mut vec_map = vec![0; map.len()];
        for (k, v) in map {
            vec_map[v as usize] = k;
        }
        (graph, vec_map)
    }
}

pub trait VertexSampler {
    fn vertex_sample(&self, vertices: usize) -> Graph;
}

impl VertexSampler for Graph {
    fn vertex_sample(&self, vertices: usize) -> Graph {
        let mut threshold = vertices;
        loop {
            let mut active = self.get_active_vertices();
            let mut rng = StdRng::seed_from_u64(0);
            active.shuffle(&mut rng);

            let subgraph = self.induced_subgraph(active[..threshold].to_vec());
            let compressed = subgraph.compress().0;
            if compressed.total_vertices() < vertices {
                threshold += 10;
            } else {
                return compressed;
            }
        }
    }
}

pub trait BFSSampler {
    fn bfs_sample(&self, vertices: usize) -> Graph;
}

impl BFSSampler for Graph {
    fn bfs_sample(&self, vertices: usize) -> Graph {
        let mut discovered = vec![false; self.total_vertices()];
        let mut subset = Vec::new();
        'main_loop: for i in 0..self.total_vertices() {
            if self.deleted_vertices[i] {
                continue;
            }

            let mut queue = VecDeque::with_capacity(vertices);
            queue.push_back(i as u32);
            discovered[i] = true;
            while let Some(vertex) = queue.pop_front() {
                if subset.len() < vertices {
                    subset.push(vertex);
                } else {
                    break 'main_loop;
                }
                for target in &self.adj[vertex as usize] {
                    if !discovered[*target as usize] {
                        discovered[*target as usize] = true;
                        if queue.len() + subset.len() < vertices {
                            queue.push_back(*target);
                        }
                    }
                }
            }
        }
        let subgraph = self.induced_subgraph(subset);
        subgraph.compress().0
    }
}

pub trait Statistics {
    /// Returns the total number of edges in the graph.
    fn edges(&self) -> usize;

    /// Returns the total number of edges that do not have their symmetric edge.
    fn directed_edges(&self) -> usize;

    /// Returns the number of edges that are in undirected form (1 undirected
    /// edge corresponds to 2 directed edges).
    fn undirected_edges(&self) -> usize;

    /// Returns the average degree of each vertex, including deleted vertices.
    fn avg_degree(&self) -> f64;

    /// Returns the average degree of each vertex after unreachable vertices
    /// have been removed.
    fn compressed_avg_degree(&self) -> f64;

    /// Returns the *diameter* of a graph, i.e., the maximimum distance between
    /// any two vertices, which may be `usize::MAX` if starting from vertex
    /// *v*, if there is an unreachable vertex *u* in the compressed graph.
    /// If the graph is empty, 0 is returned.
    fn compressed_diameter(&self) -> usize;

    /// Returns the number of ordered pairs *(v, u)* such that *v* has no path
    /// *u* in the compressed graph *G*.
    fn compressed_unreachable_vertices(&self) -> usize;

    /// Returns the number of vertices with at least one undirected edge.
    fn number_of_stars(&self) -> usize;

    /// Returns the average number of vertices in the neighborhood of all the
    /// stars.
    fn avg_star_neighborhood(&self) -> f64;

    /// Returns the number of undirected components, i.e., if *G'* is the graph
    /// only *undirected* edges, we return the number of connected components in
    /// *G'*.
    fn number_of_undirected_components(&self) -> usize;

    /// Returns the number of strongly connected components.
    fn strongly_connected_components(&self) -> usize;

    fn non_empty_vertices(&self) -> usize;
}

impl Statistics for Graph {
    fn edges(&self) -> usize {
        self.adj.iter().fold(0, |acc, l| acc + l.len())
    }

    fn directed_edges(&self) -> usize {
        let mut directed = 0;
        for i in 0..self.adj.len() {
            if self.deleted_vertices[i] || self.adj[i].is_empty() {
                continue;
            }
            for target in &self.adj[i] {
                if !self.adj[*target as usize].contains(&(i as u32)) {
                    directed += 1;
                }
            }
        }
        directed
    }

    fn undirected_edges(&self) -> usize {
        (self.edges() - self.directed_edges()) / 2
    }

    fn avg_degree(&self) -> f64 {
        let edges = self.edges();
        edges as f64 / self.total_vertices() as f64
    }

    fn compressed_avg_degree(&self) -> f64 {
        let (compressed, _) = self.compress();
        let edges = compressed.edges();
        let vertices = compressed.total_vertices();
        edges as f64 / vertices as f64
    }

    fn compressed_diameter(&self) -> usize {
        let (compressed, _) = self.compress();
        let components = compressed.tarjan(true).unwrap();

        if compressed.is_empty() {
            return 0;
        }

        let mut max = 0;
        for component in components {
            let subgraph = compressed.induced_subgraph(component);
            let (csubgraph, _) = subgraph.compress();

            for i in 0..csubgraph.vertices() {
                let mut discovered = vec![false; csubgraph.vertices()];
                let mut queue = VecDeque::with_capacity(csubgraph.vertices());
                discovered[i] = true;
                queue.push_back((i, 0));
                while let Some((vertex, distance)) = queue.pop_front() {
                    max = std::cmp::max(max, distance);
                    for target in &csubgraph.adj[vertex] {
                        if !discovered[*target as usize] {
                            discovered[*target as usize] = true;
                            queue.push_back((*target as usize, distance + 1));
                        }
                    }
                }
            }
        }

        max
    }

    fn compressed_unreachable_vertices(&self) -> usize {
        let mut unreachable = 0;
        let (compressed, _) = self.compress();
        for i in 0..compressed.vertices() {
            let mut discovered = vec![false; compressed.vertices()];
            let mut queue = VecDeque::with_capacity(compressed.vertices());
            let mut discoveries = 1;
            discovered[i] = true;
            queue.push_back(i);
            while let Some(vertex) = queue.pop_front() {
                for target in &compressed.adj[vertex] {
                    if !discovered[*target as usize] {
                        discovered[*target as usize] = true;
                        discoveries += 1;
                        queue.push_back(*target as usize);
                    }
                }
            }

            unreachable += compressed.vertices() - discoveries;
        }

        unreachable
    }

    fn number_of_stars(&self) -> usize {
        self.single_stars().len()
    }

    fn avg_star_neighborhood(&self) -> f64 {
        let stars = self.stars();
        let neighborhood = stars
            .iter()
            .fold(0, |acc, (_, neighborhood)| acc + neighborhood.len());
        neighborhood as f64 / stars.len() as f64
    }

    fn number_of_undirected_components(&self) -> usize {
        self.undirected_components().len()
    }

    fn strongly_connected_components(&self) -> usize {
        self.tarjan(true).unwrap().len()
    }

    fn non_empty_vertices(&self) -> usize {
        let mut n = 0;
        for i in 0..self.adj.len() {
            if !self.adj[i].is_empty() || !self.rev_adj[i].is_empty() {
                n += 1;
            }
        }
        n
    }
}

pub trait EdgeCycleCover {
    /// Returns a distinct set of cycles that covers as many edges as possible.
    fn edge_cycle_cover(&self) -> Vec<Vec<u32>>;
    fn disjoint_edge_cycle_cover(&self, forbidden: &[u32]) -> Vec<Vec<u32>>;
}

impl EdgeCycleCover for Graph {
    fn edge_cycle_cover(&self) -> Vec<Vec<u32>> {
        let mut cycles = FxHashSet::default();
        for i in 0..self.total_vertices() {
            if self.deleted_vertices[i] {
                continue;
            }
            for source in &self.adj[i] {
                let mut queue = VecDeque::new();
                let mut discovered = vec![None; self.total_vertices()];
                queue.push_back(*source);

                let target = i as u32;
                'main_loop: while let Some(vertex) = queue.pop_front() {
                    for next in &self.adj[vertex as usize] {
                        if *next == target {
                            let mut cycle = vec![target];
                            let mut current_vertex = vertex;
                            while current_vertex != *source {
                                cycle.push(current_vertex);
                                current_vertex = discovered[current_vertex as usize].unwrap();
                            }
                            cycle.push(*source);
                            cycle.reverse();

                            if let Some(shortcut) = self.find_shortuct(&cycle) {
                                cycle = shortcut;
                            }

                            cycle.sort_unstable();
                            cycles.insert(cycle);
                            break 'main_loop;
                        }
                        if discovered[*next as usize] == None {
                            discovered[*next as usize] = Some(vertex);
                            queue.push_back(*next);
                        }
                    }
                }
            }
        }
        cycles.into_iter().collect()
    }

    fn disjoint_edge_cycle_cover(&self, forbidden: &[u32]) -> Vec<Vec<u32>> {
        let mut cycles = FxHashSet::default();
        let forbidden_set: FxHashSet<_> = forbidden.iter().copied().collect();
        for i in 0..self.total_vertices() {
            if self.deleted_vertices[i] || forbidden_set.contains(&(i as u32)) {
                continue;
            }

            for source in &self.adj[i] {
                if forbidden_set.contains(source) {
                    continue;
                }

                let mut queue = VecDeque::new();
                let mut discovered = vec![None; self.total_vertices()];
                queue.push_back(*source);

                let target = i as u32;
                'main_loop: while let Some(vertex) = queue.pop_front() {
                    for next in &self.adj[vertex as usize] {
                        if forbidden_set.contains(next) {
                            continue;
                        }

                        if *next == target {
                            let mut cycle = vec![target];
                            let mut current_vertex = vertex;
                            while current_vertex != *source {
                                cycle.push(current_vertex);
                                current_vertex = discovered[current_vertex as usize].unwrap();
                            }
                            cycle.push(*source);
                            cycle.reverse();

                            if let Some(shortcut) = self.find_shortuct(&cycle) {
                                cycle = shortcut;
                            }

                            cycle.sort_unstable();
                            cycles.insert(cycle);
                            break 'main_loop;
                        }
                        if discovered[*next as usize] == None {
                            discovered[*next as usize] = Some(vertex);
                            queue.push_back(*next);
                        }
                    }
                }
            }
        }
        cycles.into_iter().collect()
    }
}

trait ShortCut {
    fn find_shortuct(&self, cycle: &[u32]) -> Option<Vec<u32>>;
}

impl ShortCut for Graph {
    fn find_shortuct(&self, cycle: &[u32]) -> Option<Vec<u32>> {
        if cycle.len() < 4 {
            return None;
        }

        let cycle_set: FxHashSet<_> = cycle.iter().copied().collect();
        for i in 1..cycle.len() {
            let source = cycle[i];
            let mut queue = VecDeque::new();
            let mut discovered = vec![None; self.total_vertices()];
            queue.push_back((source, 1));

            while let Some((vertex, length)) = queue.pop_front() {
                if vertex == source && length > 1 {
                    // we found a smaller cycle in the original cycle
                    let mut new_cycle = vec![];
                    let mut current_vertex = discovered[vertex as usize].unwrap();
                    while current_vertex != source {
                        new_cycle.push(current_vertex);
                        current_vertex = discovered[current_vertex as usize].unwrap();
                    }
                    new_cycle.push(source);
                    new_cycle.reverse();
                    return Some(new_cycle);
                }
                if length >= cycle.len() - 1 {
                    // we are not going to find a smaller cycle
                    continue;
                }
                // we can still search for a smaller cycle
                for target in &self.adj[vertex as usize] {
                    if cycle_set.contains(target) && discovered[*target as usize].is_none() {
                        discovered[*target as usize] = Some(vertex);
                        queue.push_back((*target, length + 1));
                    }
                }
            }
        }
        None
    }
}

pub trait TwinCliques {
    fn twin_cliques(&self) -> Vec<Vec<u32>>;
}

impl TwinCliques for Graph {
    fn twin_cliques(&self) -> Vec<Vec<u32>> {
        let mut classes: FxHashMap<Vec<u32>, Vec<u32>> = FxHashMap::default();
        let mut cliques = Vec::new();

        let mut has_twins = false;
        for i in 0..self.adj.len() {
            if self.deleted_vertices[i] {
                continue;
            }

            let mut list = self.adj[i].clone();
            // closed neighborhood
            list.push(i as u32);
            list.sort_unstable();

            if let Some(class) = classes.get_mut(&list) {
                class.push(i as u32);
                has_twins = true;
            } else {
                classes.insert(list, vec![i as u32]);
            }
        }

        if has_twins {
            for (_, twins) in classes {
                if twins.len() == 1 {
                    continue;
                }
                cliques.push(twins);
            }
        }
        cliques
    }
}

pub trait ThreeCliques {
    fn three_cliques(&self) -> Vec<(u32, u32, u32)>;
    fn undirected_three_cliques(&self) -> Vec<(u32, u32, u32)>;
}

impl ThreeCliques for Graph {
    fn three_cliques(&self) -> Vec<(u32, u32, u32)> {
        let mut cliques = Vec::new();
        let stars = self.stars();

        for (center, neighbors) in stars {
            for i in 0..neighbors.len() {
                let a = neighbors[i];
                if a < center {
                    continue;
                }
                for j in i + 1..neighbors.len() {
                    let b = neighbors[j];
                    if b > a
                        && self.adj[a as usize].contains(&b)
                        && self.adj[b as usize].contains(&a)
                    {
                        cliques.push((center, a, b));
                    }
                }
            }
        }
        cliques
    }

    fn undirected_three_cliques(&self) -> Vec<(u32, u32, u32)> {
        let mut cliques = Vec::new();
        for center in 0..self.total_vertices() {
            for i in 0..self.adj[center].len() {
                let a = self.adj[center][i];
                if a < center as u32 {
                    continue;
                }
                for j in i + 1..self.adj[center].len() {
                    let b = self.adj[center][j];
                    if b > a
                        && self.adj[a as usize].contains(&b)
                        && self.adj[b as usize].contains(&a)
                    {
                        cliques.push((center as u32, a, b));
                    }
                }
            }
        }
        cliques
    }
}

pub trait FourCliques {
    fn four_cliques(&self) -> Vec<[u32; 4]>;
}

pub trait ThreeCycles {
    fn three_cycles(&self) -> Vec<[u32; 3]>;
}

impl ThreeCycles for Graph {
    fn three_cycles(&self) -> Vec<[u32; 3]> {
        let mut cycles = Vec::new();
        let mut found = FxHashSet::default();
        for a in 0..self.total_vertices() {
            let reverse = &self.rev_adj[a];
            for b in &self.adj[a] {
                let intersection = intersection(&self.adj[*b as usize], reverse);
                for c in intersection {
                    let mut sorted = [a as u32, *b, c];
                    sorted.sort_unstable();
                    if !found.contains(&sorted) {
                        cycles.push([a as u32, *b, c]);
                        found.insert(sorted);
                    }
                }
            }
        }
        cycles
    }
}

pub trait WeakThreeCliques {
    fn weak_three_cliques(&self) -> Vec<Vec<u32>>;
}

impl WeakThreeCliques for Graph {
    fn weak_three_cliques(&self) -> Vec<Vec<u32>> {
        let three_cycles = self.three_cycles();
        let mut weak_cliques = Vec::new();
        'main_loop: for cycle in three_cycles {
            let mut traversed = FxHashSet::default();
            for i in 0..cycle.len() {
                let a = cycle[i];
                let b = cycle[(i + 1) % cycle.len()];
                let c = cycle[(i + 2) % cycle.len()];
                let mut discovered = vec![None; self.total_vertices()];
                let mut queue = VecDeque::new();
                queue.push_back(b);

                let mut found_cycle = false;
                while let Some(vertex) = queue.pop_front() {
                    if vertex == a {
                        found_cycle = true;
                        // recover cycle
                        let mut current_vertex = vertex;
                        while current_vertex != b {
                            traversed.insert(current_vertex);
                            current_vertex = discovered[current_vertex as usize].unwrap();
                        }
                        traversed.insert(b);
                    }
                    for next in &self.adj[vertex as usize] {
                        if *next == c || *next == b || discovered[*next as usize].is_some() {
                            continue;
                        }
                        discovered[*next as usize] = Some(vertex);
                        queue.push_back(*next);
                    }
                }

                if !found_cycle {
                    // continue with the next cycle
                    continue 'main_loop;
                }
            }
            // only reachable if we found three cycles
            weak_cliques.push(traversed.into_iter().collect());
        }
        weak_cliques
    }
}

impl FourCliques for Graph {
    fn four_cliques(&self) -> Vec<[u32; 4]> {
        let mut cliques = Vec::new();
        let mut edge_set = FxHashSet::default();
        for source in 0..self.total_vertices() {
            for target in &self.adj[source] {
                edge_set.insert((source as u32, *target));
            }
        }

        for mid in 0..self.total_vertices() {
            if self.adj[mid].len() < 3 {
                continue;
            }
            for i in 0..self.adj[mid].len() {
                let a = &self.adj[mid][i];
                if *a < mid as u32 {
                    continue;
                }
                for j in i + 1..self.adj[mid].len() {
                    let b = &self.adj[mid][j];
                    if *b < mid as u32 {
                        continue;
                    }
                    for k in j + 1..self.adj[mid].len() {
                        let c = &self.adj[mid][k];
                        if *c < mid as u32 {
                            continue;
                        }
                        if edge_set.contains(&(*a, *b))
                            && edge_set.contains(&(*a, *c))
                            && edge_set.contains(&(*b, *c))
                        {
                            cliques.push([mid as u32, *a, *b, *c]);
                        }
                    }
                }
            }
        }
        cliques
    }
}

pub trait SplitReduce {
    fn split_reduce(self) -> (Graph, Graph, Vec<u32>);
    fn split(self) -> (Graph, Graph);
}

impl SplitReduce for Graph {
    fn split_reduce(mut self) -> (Graph, Graph, Vec<u32>) {
        let vertices = self.total_vertices();
        let mut undirected_graph = Graph::new(vertices);
        let mut forced = Reducable::reduce(&mut self, vertices).unwrap();

        let mut id = 0;
        let mut constraint_map = vec![Vec::new(); vertices];

        loop {
            let stars = self.stars();
            if stars.is_empty() {
                break;
            }

            let mut sources = Vec::with_capacity(stars.len());
            for (source, neighbors) in &stars {
                for neighbor in neighbors {
                    undirected_graph.add_arc(*source, *neighbor);
                    undirected_graph.add_arc(*neighbor, *source);

                    if *source < *neighbor {
                        constraint_map[*source as usize].push(id);
                        constraint_map[*neighbor as usize].push(id);
                        id += 1;
                    }
                }
                sources.push(*source);
            }
            self.mark_forbidden(&sources);
            self.remove_undirected_edges(stars);

            let mut reduced = Reducable::reduce(&mut self, vertices).unwrap();
            if reduced.is_empty() {
                break;
            }
            forced.append(&mut reduced);
        }

        // Some generated constraints may already be satisfied, filter those.
        let mut forced_constraints = FxHashSet::default();
        for vertex in &forced {
            for constraint_index in &constraint_map[*vertex as usize] {
                forced_constraints.insert(*constraint_index);
            }
            undirected_graph.remove_vertex(*vertex);
        }

        (self, undirected_graph, forced)
    }

    fn split(mut self) -> (Graph, Graph) {
        let vertices = self.total_vertices();
        let mut undirected_graph = Graph::new(vertices);

        let stars = self.stars();
        let mut sources = Vec::with_capacity(stars.len());
        for (source, neighbors) in &stars {
            for neighbor in neighbors {
                undirected_graph.add_arc(*source, *neighbor);
                undirected_graph.add_arc(*neighbor, *source);

            }
            sources.push(*source);
        }
        self.remove_undirected_edges(stars);
        (self, undirected_graph)
    }
}


impl Add for Graph {
    type Output = Graph;

    fn add(mut self, rhs: Self) -> Self::Output {
        assert_eq!(self.total_vertices(), rhs.total_vertices());
        for i in 0..rhs.total_vertices() {
            for j in 0..rhs.adj[i].len() {
                let from = i as u32;
                let to = rhs.adj[i][j];
                self.add_arc(from, to);
            }
        }
        
        for i in 0..self.forbidden.len() {
            self.forbidden[i] = false;
        }

        for i in 0..self.deleted_vertices.len() {
            self.deleted_vertices[i] = false;
        }

        self
    }
}

#[cfg(test)]
mod tests {
    use crate::graph::Statistics;

    use super::EdgeCycleCover;
    use super::EdgeIter;
    use super::Graph;
    use super::Reducable;
    use super::WeakThreeCliques;

    fn pace_example_graph() -> Graph {
        let mut graph = Graph::new(4);
        graph.add_arc(0, 1);
        graph.add_arc(0, 2);
        graph.add_arc(1, 2);
        graph.add_arc(2, 3);
        graph.add_arc(3, 0);
        graph
    }

    #[test]
    fn is_cyclic_test_001() {
        let graph = pace_example_graph();
        assert!(graph.is_cyclic());
    }

    #[test]
    fn is_cyclic_test_002() {
        let mut graph = pace_example_graph();
        graph.remove_vertex(1);
        assert!(graph.is_cyclic());
    }

    #[test]
    fn is_cyclic_test_003() {
        let mut graph = pace_example_graph();
        graph.remove_vertex(0);
        assert!(!graph.is_cyclic());
    }

    #[test]
    fn has_fvs_cycle_test_001() {
        let graph = pace_example_graph();
        let fvs = vec![];
        assert!(!graph.is_acyclic_with_fvs(&fvs));
    }

    #[test]
    fn has_fvs_cycle_test_002() {
        let graph = pace_example_graph();
        let fvs = vec![1];
        assert!(!graph.is_acyclic_with_fvs(&fvs));
    }

    #[test]
    fn has_fvs_cycle_test_004() {
        let graph = pace_example_graph();
        let fvs = vec![0];
        assert!(graph.is_acyclic_with_fvs(&fvs));
    }

    #[test]
    fn has_fvs_cycle_test_005() {
        let graph = pace_example_graph();
        let fvs = vec![2];
        assert!(graph.is_acyclic_with_fvs(&fvs));
    }

    #[test]
    fn has_fvs_cycle_test_006() {
        let graph = pace_example_graph();
        let fvs = vec![3];
        assert!(graph.is_acyclic_with_fvs(&fvs));
    }

    #[test]
    fn self_loop_test() {
        let mut graph = Graph::new(2);
        graph.add_arc(0, 1);
        graph.add_arc(1, 0);
        graph.reduce(2);
    }

    #[test]
    fn scc_test_001() {
        let mut graph = Graph::new(3);
        graph.add_arc(0, 1);
        graph.add_arc(1, 0);
        graph.add_arc(0, 2);
        let components = graph.tarjan(false).unwrap();
        assert_eq!(components.len(), 2);
    }

    #[test]
    fn scc_test_002() {
        let mut graph = Graph::new(3);
        graph.add_arc(0, 1);
        graph.add_arc(1, 0);
        graph.add_arc(0, 2);
        graph.add_arc(2, 0);
        let components = graph.tarjan(false).unwrap();
        assert_eq!(components.len(), 1);
    }

    #[test]
    fn ssc_reduction_test_001() {
        let mut graph = Graph::new(4);
        graph.add_arc(0, 1);
        graph.add_arc(1, 0);
        graph.add_arc(1, 2);
        graph.add_arc(2, 3);
        graph.add_arc(3, 2);
        graph.scc_reduction();

        let mut expected = Graph::new(4);
        expected.add_arc(0, 1);
        expected.add_arc(1, 0);
        expected.add_arc(2, 3);
        expected.add_arc(3, 2);

        // assert_eq!(graph, expected);
    }

    #[test]
    fn ssc_reduction_test002() {
        let mut graph = Graph::new(2);
        graph.add_arc(0, 1);
        graph.add_arc(1, 0);
        graph.single_incoming_reduction();
        graph.scc_reduction();
        assert_eq!(graph.vertices(), 1);
    }

    #[test]
    fn undir_edge_iter_test_001() {
        let mut graph = Graph::new(3);
        graph.add_arc(0, 1);
        graph.add_arc(1, 0);
        graph.add_arc(0, 2);
        graph.add_arc(2, 0);
        graph.add_arc(1, 2);
        graph.add_arc(2, 1);
        let iter = graph.undir_edge_iter();
        assert_eq!(iter.count(), 3);
    }

    #[test]
    fn find_cycle_test_001() {
        let mut graph = Graph::new(2);
        graph.add_arc(0, 1);
        assert_eq!(graph.find_cycle_with_fvs(&[]), None);
    }

    #[test]
    fn find_cycle_test_002() {
        let mut graph = Graph::new(2);
        graph.add_arc(0, 1);
        graph.add_arc(1, 0);
        assert_eq!(graph.find_cycle_with_fvs(&[]), Some(vec![0, 1]));
    }

    #[test]
    fn find_cycle_test_003() {
        let mut graph = Graph::new(5);
        graph.add_arc(0, 1);
        graph.add_arc(1, 2);
        graph.add_arc(2, 3);
        graph.add_arc(3, 4);
        graph.add_arc(4, 2);

        assert_eq!(graph.find_cycle_with_fvs(&[]), Some(vec![2, 3, 4]));
    }

    #[test]
    fn find_cycle_test_004() {
        let mut graph = Graph::new(5);
        graph.add_arc(0, 1);
        graph.add_arc(1, 2);
        graph.add_arc(2, 3);
        graph.add_arc(3, 4);
        graph.add_arc(4, 2);

        assert_eq!(graph.find_cycle_with_fvs(&[2]), None);
    }

    #[test]
    fn find_cycle_test_005() {
        let mut graph = Graph::new(3);
        graph.add_arc(0, 1);
        graph.add_arc(0, 2);
        graph.add_arc(1, 2);
        assert_eq!(graph.find_cycle_with_fvs(&[]), None);
    }

    #[test]
    fn undirected_component_test_001() {
        let mut graph = Graph::new(4);
        graph.add_arc(0, 1);
        graph.add_arc(1, 0);
        graph.add_arc(2, 3);
        graph.add_arc(3, 2);
        graph.add_arc(2, 1);
        let components = graph.undirected_components();
        assert_eq!(components.len(), 2);
    }

    #[test]
    fn induced_subgraph_test_001() {
        let mut graph = Graph::new(3);
        graph.add_arc(0, 1);
        graph.add_arc(1, 2);
        graph.add_arc(2, 1);
        let set = vec![1, 2];
        let subgraph = graph.induced_subgraph(set);
        assert_eq!(subgraph.vertices(), 2);
    }

    #[test]
    fn induced_subgraph_test_002() {
        let mut graph = Graph::new(4);
        graph.add_arc(0, 1);
        graph.add_arc(1, 2);
        graph.add_arc(2, 1);
        let set = vec![1, 2, 3];
        let subgraph = graph.induced_subgraph(set);
        assert_eq!(subgraph.vertices(), 3);
    }

    #[test]
    fn edges_test_001() {
        let mut graph = Graph::new(4);
        graph.add_arc(0, 1);
        graph.add_arc(1, 2);
        graph.add_arc(2, 1);
        assert_eq!(graph.edges(), 3);
    }

    #[test]
    fn directed_edges_test_001() {
        let mut graph = Graph::new(4);
        graph.add_arc(0, 1);
        graph.add_arc(1, 2);
        graph.add_arc(2, 1);
        assert_eq!(graph.directed_edges(), 1);
    }

    #[test]
    fn directed_edges_test_002() {
        let mut graph = Graph::new(4);
        graph.add_arc(0, 1);
        graph.add_arc(1, 2);
        assert_eq!(graph.directed_edges(), 2);
    }

    #[test]
    fn undirected_edges_test_001() {
        let mut graph = Graph::new(4);
        graph.add_arc(0, 1);
        graph.add_arc(1, 2);
        graph.add_arc(2, 1);
        assert_eq!(graph.undirected_edges(), 1);
    }

    #[test]
    fn compressed_unreachable_vertices_test_001() {
        let mut graph = Graph::new(4);
        graph.add_arc(1, 2);
        graph.add_arc(2, 1);
        assert_eq!(graph.compressed_unreachable_vertices(), 0);
    }

    #[test]
    fn edge_cycle_cover_test_001() {
        let mut graph = Graph::new(3);
        graph.add_arc(0, 1);
        graph.add_arc(1, 2);
        graph.add_arc(2, 0);
        let cycle_cover = graph.edge_cycle_cover();
        assert_eq!(cycle_cover, vec![vec![0, 1, 2]]);
    }

    #[test]
    fn weak_three_clique_test_001() {
        let mut graph = Graph::new(6);
        for i in 0..3 {
            graph.add_arc(i, (i + 1) % 3);
            graph.add_arc((i + 1) % 3, i + 3);
            graph.add_arc(i + 3, i);
        }
        let weak_three_cliques = graph.weak_three_cliques();
        assert_eq!(weak_three_cliques.len(), 1);
    }
}
