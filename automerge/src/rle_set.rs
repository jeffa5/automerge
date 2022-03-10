use std::collections::BTreeMap;
#[cfg(debug_assertions)]
use std::collections::HashSet;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RleSet {
    /// mapping from start of range to end of the range
    map: BTreeMap<u64, u64>,
    #[cfg(debug_assertions)]
    set: HashSet<u64>,
}

impl RleSet {
    pub fn insert(&mut self, value: u64) {
        // get iterator at point of this value
        let right = self.map.remove(&(value + 1));
        let left = self.map.range(..=value).last().map(|(a, b)| (*a, *b));

        match (left, right) {
            (None, None) => {
                // nothing found so just insert ourselves
                self.map.insert(value, 1);
            }
            (None, Some(v)) => {
                // so right that we can extend so merge that with our new value
                self.map.insert(value, v + 1);
            }
            (Some((k, v)), None) => {
                if k + v == value {
                    // extend the existing range
                    self.map.insert(k, v + 1);
                } else if k + v < value {
                    // can't extend the existing range so just add our own
                    self.map.insert(value, 1);
                }
            }
            (Some((lk, lv)), Some(rv)) => {
                if lk + lv == value {
                    self.map.insert(lk, lv + 1 + rv);
                } else {
                    self.map.insert(value, 1 + rv);
                }
            }
        }
        #[cfg(debug_assertions)]
        {
            self.set.insert(value);
            assert_eq!(self.set, self.iter().collect());
        }
    }

    pub fn remove(&mut self, value: u64) {
        let left = self.map.range(..=value).last().map(|(a, b)| (*a, *b));
        match left {
            Some((k, v)) => {
                if k == value {
                    // start of the range
                    self.map.remove(&k);
                    self.map.insert(value + 1, v - 1);
                } else if k + v - 1 == value {
                    // end of the range
                    self.map.insert(k, v - 1);
                } else if k + v >= value {
                    // middle of the range
                    let left = value - k;
                    let right = k + v - 1 - value;
                    self.map.insert(k, left);
                    self.map.insert(value + 1, right);
                }
            }
            None => {
                // nothing to delete
            }
        }
        #[cfg(debug_assertions)]
        {
            self.set.remove(&value);
            assert_eq!(self.set, self.iter().collect());
        }
    }

    pub fn contains(&self, value: u64) -> bool {
        let left = self.map.range(..=value).last().map(|(a, b)| (*a, *b));
        match left {
            Some((k, v)) => k + v > value,
            None => false,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = u64> + '_ {
        self.map
            .iter()
            .map(|(start, length)| (*start..(start + length)))
            .flatten()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn basic_insert() {
        let mut s = RleSet::default();
        s.insert(1);
        s.insert(2);
        s.insert(5);
        s.insert(7);
        s.insert(8);
        let mut map = BTreeMap::new();
        map.insert(1, 2);
        map.insert(5, 1);
        map.insert(7, 2);
        assert_eq!(s.map, map);

        s.insert(3);
        map.insert(1, 3);
        assert_eq!(s.map, map);

        s.insert(6);
        map.insert(5, 4);
        map.remove(&7);
        assert_eq!(s.map, map);

        s.insert(15);
        map.insert(15, 1);
        assert_eq!(s.map, map);

        s.insert(14);
        map.insert(14, 2);
        map.remove(&15);
        assert_eq!(s.map, map);

        let mut s = RleSet::default();
        s.insert(5);
        let mut map = BTreeMap::new();
        map.insert(5, 1);
        assert_eq!(s.map, map);

        s.insert(4);
        map.insert(4, 2);
        map.remove(&5);
        assert_eq!(s.map, map);
    }

    #[test]
    fn duplicate_insert() {
        let mut s = RleSet::default();
        s.insert(1);
        s.insert(2);
        s.insert(5);
        s.insert(7);
        s.insert(8);
        let mut map = BTreeMap::new();
        map.insert(1, 2);
        map.insert(5, 1);
        map.insert(7, 2);
        assert_eq!(s.map, map);

        s.insert(1);
        assert_eq!(s.map, map);
        s.insert(2);
        assert_eq!(s.map, map);
        s.insert(5);
        assert_eq!(s.map, map);
        s.insert(7);
        assert_eq!(s.map, map);
        s.insert(8);
        assert_eq!(s.map, map);
    }

    #[test]
    fn basic_remove() {
        let mut s = RleSet::default();
        s.insert(1);
        s.insert(2);
        s.insert(5);
        s.insert(7);
        s.insert(8);
        s.insert(3);
        s.insert(6);

        let mut map = BTreeMap::new();
        map.insert(1, 3);
        map.insert(5, 4);
        assert_eq!(s.map, map);

        s.remove(6);
        map.insert(5, 1);
        map.insert(7, 2);
        assert_eq!(s.map, map);

        s.remove(3);
        map.insert(1, 2);
        assert_eq!(s.map, map);

        s.remove(1);
        map.insert(2, 1);
        map.remove(&1);
        assert_eq!(s.map, map);
    }

    #[test]
    fn basic_iter() {
        let mut s = RleSet::default();
        s.insert(1);
        s.insert(2);
        s.insert(5);
        s.insert(7);
        s.insert(8);
        s.insert(3);
        s.insert(6);
        s.insert(15);
        s.insert(14);

        assert_eq!(
            s.iter().collect::<Vec<_>>(),
            vec![1, 2, 3, 5, 6, 7, 8, 14, 15]
        );
    }
}
