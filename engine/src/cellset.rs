use std::iter::FromIterator;

use crate::NUM_CELLS;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[must_use]
pub struct CellSet {
    pub data: u64,
}

impl CellSet {
    pub const fn new() -> Self {
        CellSet { data: 0 }
    }

    pub const fn full() -> Self {
        CellSet { data: !0 >> 4 }
    }

    pub const fn insert(self, value: u8) -> Self {
        CellSet {
            data: self.data | 1 << value,
        }
    }

    pub const fn remove(self, value: u8) -> Self {
        CellSet {
            data: self.data & !(1 << value),
        }
    }

    pub const fn contains(self, value: u8) -> bool {
        (self.data & (1 << value)) != 0
    }

    pub const fn intersect(self, other: CellSet) -> Self {
        CellSet {
            data: self.data & other.data,
        }
    }

    pub const fn exclude(self, other: CellSet) -> Self {
        CellSet {
            data: self.data & !other.data,
        }
    }

    pub const fn union(self, other: CellSet) -> Self {
        CellSet {
            data: self.data | other.data,
        }
    }

    pub const fn is_empty(self) -> bool {
        self.data == 0
    }

    pub const fn len(self) -> usize {
        self.data.count_ones() as usize
    }

    pub const fn iter(self) -> Iter {
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
        let size = (self.cell_set.data >> self.value).count_ones() as usize;
        (size, Some(size))
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
            ret = ret.insert(i);
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
        s = s.insert(0);
        assert!(s.contains(0));
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn insert_then_remove() {
        let mut s = CellSet::new();
        assert!(!s.contains(0));
        s = s.insert(0);
        assert!(s.contains(0));
        s = s.remove(0);
        assert!(!s.contains(0));
    }

    #[test]
    fn insert_then_is_empty() {
        let mut s = CellSet::new();
        assert!(s.is_empty());
        s = s.insert(0);
        assert!(!s.is_empty());
    }

    #[test]
    fn insert_then_iter() {
        let mut s = CellSet::new();
        s = s.insert(0);
        assert_eq!(s.into_iter().next(), Some(0));
    }

    #[test]
    fn insert_multiple_values() {
        let mut s = CellSet::new();

        let vals = [0, 30, 46];

        for v in vals.iter() {
            s = s.insert(*v as u8);
        }

        assert_eq!(s.len(), vals.len());

        let mut iter = s.into_iter();
        for (seen, v) in vals.iter().enumerate() {
            assert_eq!(iter.size_hint().0, vals.len() - seen);
            assert_eq!(iter.next(), Some(*v as u8));
        }
    }
}
