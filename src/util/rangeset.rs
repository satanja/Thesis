use std::ops::Index;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct RangeSet {
    set: Vec<u32>,
    table: Vec<Option<usize>>,
}

impl RangeSet {
    pub fn new(range: usize) -> RangeSet {
        RangeSet {
            set: Vec::with_capacity(range),
            table: vec![None; range],
        }
    }

    pub fn insert(&mut self, vertex: u32) -> bool {
        if self.table[vertex as usize].is_none() {
            self.table[vertex as usize] = Some(self.set.len());
            self.set.push(vertex);
            return true;
        }
        false
    }

    pub fn contains(&self, vertex: &u32) -> bool {
        self.table[*vertex as usize].is_some()
    }

    pub fn remove(&mut self, vertex: &u32) -> bool {
        if let Some(index) = self.table[*vertex as usize] {
            // exists since vertex has an index
            let last = self.set.last().unwrap();
            let last_index = self.table[*last as usize].unwrap();

            self.table.swap(*last as usize, *vertex as usize);

            self.set.swap(index, last_index);

            self.set.pop();
            self.table[*vertex as usize] = None;

            return true;
        }
        false
    }

    pub fn pop(&mut self) -> Option<u32> {
        if self.len() == 0 {
            None
        } else {
            let last = self.set.pop().unwrap();
            self.table[last as usize] = None;
            Some(last)
        }
    }

    pub fn len(&self) -> usize {
        self.set.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &u32> {
        self.set.iter()
    }
}

impl Index<usize> for RangeSet {
    type Output = u32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.set[index]
    }
}

impl FromIterator<u32> for RangeSet {
    fn from_iter<T: IntoIterator<Item = u32>>(iter: T) -> Self {
        let mut set = Vec::new();
        for vertex in iter {
            set.push(vertex);
        }
        let table: Vec<_> = (0..set.len()).map(Some).collect();
        RangeSet { set, table }
    }
}
