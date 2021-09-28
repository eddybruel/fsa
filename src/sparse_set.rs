use std::slice;

#[derive(Clone, Debug)]
pub struct SparseSet {
    dense: Vec<usize>,
    sparse: Vec<usize>,
}

impl SparseSet {
    pub fn new(capacity: usize) -> Self {
        Self {
            dense: Vec::with_capacity(capacity),
            sparse: vec![0; capacity],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.dense.is_empty()
    }

    pub fn len(&self) -> usize {
        self.dense.len()
    }

    pub fn contains(&self, value: usize) -> bool {
        self.dense.get(self.sparse[value]) == Some(&value)
    }

    pub fn iter(&self) -> Iter<'_> {
        Iter {
            iter: self.dense.iter(),
        }
    }

    pub fn insert(&mut self, value: usize) -> bool {
        assert!(value < self.dense.capacity());
        if self.contains(value) {
            return false;
        }
        let index = self.dense.len();
        self.dense.push(value);
        self.sparse[value] = index;
        true
    }

    pub fn clear(&mut self) {
        self.dense.clear()
    }
}

impl<'a> IntoIterator for &'a SparseSet {
    type IntoIter = Iter<'a>;
    type Item = usize;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct Iter<'a> {
    iter: slice::Iter<'a, usize>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().cloned()
    }
}
