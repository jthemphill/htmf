use std::iter::FromIterator;

use NUM_CELLS;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CellSet {
    pub data: u64,
}

impl CellSet {
    pub fn new() -> Self {
        CellSet { data: 0 }
    }

    pub fn full() -> Self {
        CellSet { data: !0 >> 4 }
    }

    pub fn insert(&mut self, value: u8) {
        self.data |= 1 << value;
    }

    pub fn remove(&mut self, value: u8) {
        self.data &= !(1 << value);
    }

    pub fn contains(self, value: u8) -> bool {
        (self.data & (1 << value)) != 0
    }

    pub fn intersect(&mut self, other: CellSet) {
        self.data &= other.data;
    }

    pub fn exclude(&mut self, other: CellSet) {
        self.data &= !other.data;
    }

    pub fn union(&mut self, other: CellSet) {
        self.data |= other.data;
    }

    pub fn is_empty(self) -> bool {
        self.data == 0
    }

    pub fn len(self) -> usize {
        self.data.count_ones() as usize
    }

    pub fn iter(self) -> Iter {
        Iter {
            cell_set: self,
            value: 0,
        }
    }
}

pub struct Iter {
    cell_set: CellSet,
    value: u8,
}

impl Iterator for Iter {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        while self.value < NUM_CELLS as u8 {
            let v = self.value;

            self.value += 1;
            let remainder = self.cell_set.data >> self.value;
            self.value += remainder.trailing_zeros() as u8;
            self.value = std::cmp::min(self.value, NUM_CELLS as u8);

            if self.cell_set.contains(v) {
                return Some(v);
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(NUM_CELLS - self.value as usize))
    }
}

impl IntoIterator for CellSet {
    type Item = u8;
    type IntoIter = Iter;

    fn into_iter(self) -> Iter {
        self.iter()
    }
}

impl FromIterator<u8> for CellSet {
    fn from_iter<I: IntoIterator<Item = u8>>(iter: I) -> CellSet {
        let mut ret = CellSet::new();
        for i in iter {
            ret.insert(i);
        }
        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_then_contains() {
        let mut s = CellSet::new();
        assert!(!s.contains(0));
        s.insert(0);
        assert!(s.contains(0));
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn insert_then_remove() {
        let mut s = CellSet::new();
        assert!(!s.contains(0));
        s.insert(0);
        assert!(s.contains(0));
        s.remove(0);
        assert!(!s.contains(0));
    }

    #[test]
    fn insert_then_is_empty() {
        let mut s = CellSet::new();
        assert!(s.is_empty());
        s.insert(0);
        assert!(!s.is_empty());
    }

    #[test]
    fn insert_then_iter() {
        let mut s = CellSet::new();
        s.insert(0);
        assert_eq!(s.into_iter().next(), Some(0));
    }

    #[test]
    fn insert_multiple_values() {
        let mut s = CellSet::new();

        let vals = [0, 30, 46];

        for v in vals.iter() {
            s.insert(*v as u8);
        }

        assert_eq!(s.len(), vals.len());

        let mut iter = s.into_iter();
        for v in vals.iter() {
            assert_eq!(iter.next(), Some(*v as u8));
        }
    }
}
