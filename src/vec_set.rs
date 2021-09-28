use std::{cmp::Ordering, slice};

#[derive(Clone, Debug)]
pub struct VecSet<T>(Vec<T>);

impl<T> VecSet<T> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub unsafe fn from_vec_unchecked(vec: Vec<T>) -> Self {
        Self(vec)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            iter: self.0.iter(),
        }
    }

    pub fn into_vec(self) -> Vec<T> {
        self.0
    }
}

impl<T: Ord> VecSet<T> {
    pub fn difference<'a>(&'a self, other: &'a Self) -> Difference<'a, T> {
        let mut iter = self.iter();
        let mut other_iter = other.iter();
        Difference {
            item: iter.next(),
            iter,
            other_item: other_iter.next(),
            other_iter,
        }
    }

    pub fn intersection<'a>(&'a self, other: &'a Self) -> Intersection<'a, T> {
        let mut iter = self.iter();
        let mut other_iter = other.iter();
        Intersection {
            item: iter.next(),
            iter,
            other_item: other_iter.next(),
            other_iter,
        }
    }
}

impl<'a, T> IntoIterator for &'a VecSet<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct Iter<'a, T> {
    iter: slice::Iter<'a, T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

pub struct Difference<'a, T> {
    item: Option<&'a T>,
    iter: Iter<'a, T>,
    other_item: Option<&'a T>,
    other_iter: Iter<'a, T>,
}

impl<'a, T: Clone + Ord> Difference<'a, T> {
    pub fn into_vec_set(self, vec_set: &mut VecSet<T>) {
        vec_set.0.clear();
        vec_set.0.extend(self.cloned());
    }
}

impl<'a, T: Ord> Iterator for Difference<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match (self.item, self.other_item) {
                (None, _) => break None,
                (Some(item), None) => break Some(item),
                (Some(item), Some(other_item)) => match item.cmp(&other_item) {
                    Ordering::Less => {
                        self.item = self.iter.next();
                        break Some(item);
                    }
                    Ordering::Equal => {
                        self.item = self.iter.next();
                        self.other_item = self.other_iter.next();
                        break None;
                    }
                    Ordering::Greater => self.other_item = self.other_iter.next(),
                },
            }
        }
    }
}

pub struct Intersection<'a, T> {
    item: Option<&'a T>,
    iter: Iter<'a, T>,
    other_item: Option<&'a T>,
    other_iter: Iter<'a, T>,
}

impl<'a, T: Clone + Ord> Intersection<'a, T> {
    pub fn into_vec_set(self, vec_set: &mut VecSet<T>) {
        vec_set.0.clear();
        vec_set.0.extend(self.cloned());
    }
}

impl<'a, T: Ord> Iterator for Intersection<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match (self.item, self.other_item) {
                (None, None) => break None,
                (Some(_), None) | (None, Some(_)) => break None,
                (Some(item), Some(other_item)) => match item.cmp(&other_item) {
                    Ordering::Less => self.item = self.iter.next(),
                    Ordering::Equal => {
                        self.item = self.iter.next();
                        self.other_item = self.other_iter.next();
                        break Some(item);
                    }
                    Ordering::Greater => self.other_item = self.other_iter.next(),
                },
            }
        }
    }
}
