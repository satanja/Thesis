use std::{cmp::Ordering, ops::Index};

fn left_child(index: usize) -> usize {
    2 * index + 1
}

fn right_child(index: usize) -> usize {
    2 * index + 2
}

fn parent(index: usize) -> usize {
    (index - 1) / 2
}

pub trait KeyValue {
    fn key(&self) -> usize;
    fn value(&self) -> i64;
}

#[derive(Clone, Debug)]
pub struct MinItem {
    key: usize,
    count: i64,
}

impl MinItem {
    pub fn new(key: usize, count: i64) -> MinItem {
        MinItem { key, count }
    }
}

impl KeyValue for MinItem {
    fn key(&self) -> usize {
        self.key
    }
    fn value(&self) -> i64 {
        self.count
    }
}

impl PartialEq for MinItem {
    fn eq(&self, other: &Self) -> bool {
        self.count == other.count
    }
}

impl Eq for MinItem {}

impl PartialOrd for MinItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MinItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.count.cmp(&other.count)
    }
}

#[derive(Clone, Debug)]
pub struct MaxItem {
    key: usize,
    count: i64,
}

impl MaxItem {
    pub fn new(key: usize, count: i64) -> MaxItem {
        MaxItem { key, count }
    }
}

impl KeyValue for MaxItem {
    fn key(&self) -> usize {
        self.key
    }
    fn value(&self) -> i64 {
        self.count
    }
}

impl PartialEq for MaxItem {
    fn eq(&self, other: &Self) -> bool {
        self.count == other.count
    }
}

impl Eq for MaxItem {}

impl PartialOrd for MaxItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MaxItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.count.cmp(&self.count)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]

pub struct Heap<T: Ord + KeyValue> {
    heap: Vec<T>,
    table: Vec<usize>,
}

impl<T: Ord + KeyValue> Heap<T> {
    pub fn new(_range: usize) -> Heap<T> {
        Heap {
            heap: Vec::new(),
            table: Vec::new(),
        }
    }

    pub fn load(&mut self, data: Vec<T>) {
        self.table = (0..data.len()).collect();
        self.heap = data;
        self.build_min();
    }

    pub fn build_min(&mut self) {
        for i in (0..self.len() / 2).rev() {
            self.min_heapify(i);
        }
    }

    fn min_heapify(&mut self, index: usize) {
        let left = left_child(index);
        let right = right_child(index);

        let mut smallest = if left < self.len()
            && self.heap[left].partial_cmp(&self.heap[index]).unwrap() == Ordering::Less
        {
            left
        } else {
            index
        };

        if right < self.len()
            && self.heap[right].partial_cmp(&self.heap[smallest]).unwrap() == Ordering::Less
        {
            smallest = right;
        }

        if smallest != index {
            let i = self.heap[smallest].key();
            let j = self.heap[index].key();
            self.table[i] = index;
            self.table[j] = smallest;

            self.heap.swap(index, smallest);
            self.min_heapify(smallest);
        }
    }

    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn peek_min(&self) -> Option<&T> {
        if self.len() != 0 {
            Some(&self.heap[0])
        } else {
            None
        }
    }

    pub fn extract_min(&mut self) -> Option<T> {
        let last = self.len() - 1;

        let i = self.heap[0].key();
        let j = self.heap[last].key();
        self.table[i] = last;
        self.table[j] = 0;

        self.heap.swap(0, last);

        let min = self.heap.pop();
        self.min_heapify(0);

        min
    }

    #[allow(dead_code)]
    pub fn insert(&mut self, element: T) {
        let i = element.key();
        self.heap.push(element);
        let mut index = self.len() - 1;
        self.table[i] = index;

        while index > 0
            && self.heap[parent(index)]
                .partial_cmp(&self.heap[index])
                .unwrap()
                == Ordering::Greater
        {
            let i = self.heap[parent(index)].key();
            let j = self.heap[index].key();
            self.table[i] = index;
            self.table[j] = parent(index);

            self.heap.swap(parent(index), index);
            index = parent(index);
        }
    }

    pub fn unchecked_insert(&mut self, element: T, index: usize) {
        self.table[element.key()] = index;
        self.heap.push(element);
    }

    pub fn decrease_key(&mut self, element: T) {
        let idx = self.get_index(element.key());
        let original = &self.heap[idx];

        if element.partial_cmp(original).unwrap() == Ordering::Greater {
            return;
        }

        let mut index = idx;
        self.heap[index] = element;
        while index > 0
            && self.heap[parent(index)]
                .partial_cmp(&self.heap[index])
                .unwrap()
                == Ordering::Greater
        {
            let i = self.heap[parent(index)].key();
            let j = self.heap[index].key();
            self.table[i] = index;
            self.table[j] = parent(index);

            self.heap.swap(parent(index), index);
            index = parent(index);
        }
    }
    pub fn get_index(&self, key: usize) -> usize {
        self.table[key]
    }
}

impl<T: Ord + KeyValue> Index<usize> for Heap<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.heap[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn singleton() {
        let mut heap = Heap::new(5);
        let item = MinItem::new(0, 2);
        heap.insert(item);

        assert_eq!(heap.get_index(0), 0);
    }

    #[test]
    fn two_elements() {
        let mut heap = Heap::new(5);
        let a = MinItem::new(0, 2);
        let b = MinItem::new(1, 0);
        heap.insert(a);
        heap.insert(b);
        assert_eq!(heap.get_index(0), 1);
        assert_eq!(heap.get_index(1), 0);
    }

    #[test]
    fn decreasing_key() {
        let mut heap = Heap::new(5);
        let a = MinItem::new(0, 2);
        let b = MinItem::new(1, 1);
        heap.insert(a);
        heap.insert(b);
        assert_eq!(heap.get_index(0), 1);

        heap.decrease_key(MinItem::new(0, 0));
        assert_eq!(heap.get_index(0), 0);
        assert_eq!(heap.get_index(1), 1);
    }
}
