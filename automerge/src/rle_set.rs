use std::cmp::Ordering;
#[cfg(debug_assertions)]
use std::collections::HashSet;
use std::{collections::BTreeMap, fmt::Debug, hash::Hash};

use crate::types::OpId;

pub trait Runnable: Sized {
    fn next(&self) -> Self {
        self.at(1)
    }

    fn at(&self, index: u64) -> Self;

    fn sub(&self, other: &Self) -> u64;
}

impl Runnable for u64 {
    fn at(&self, index: u64) -> Self {
        self + index
    }

    fn sub(&self, other: &Self) -> u64 {
        self - other
    }
}

#[derive(PartialEq)]
pub struct RunnableOpId(OpId);

impl PartialOrd for RunnableOpId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.actor().partial_cmp(&other.0.actor()).and_then(|o| {
            if Ordering::Equal == o {
                self.0.counter().partial_cmp(&other.0.counter())
            } else {
                Some(o)
            }
        })
    }
}

impl Runnable for RunnableOpId {
    fn at(&self, index: u64) -> Self {
        RunnableOpId(OpId(self.0.counter() + index, self.0.actor()))
    }

    fn sub(&self, other: &Self) -> u64 {
        self.0.counter() - other.0.counter()
    }
}

#[derive(Debug, Clone, Default)]
pub struct RleSet<T> {
    /// mapping from start of range to end of the range
    map: BTreeMap<T, u64>,
    #[cfg(debug_assertions)]
    reference_set: HashSet<T>,
}

impl<T> PartialEq for RleSet<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.map == other.map
    }
}

impl<T> RleSet<T>
where
    T: Ord + Runnable + Hash + PartialEq + Debug + Clone,
{
    /// Insert the value into the set, returning true if it wasn't already in the set.
    pub fn insert(&mut self, value: T) -> bool {
        // get iterator at point of this value
        let right = self.map.remove(&(value.next()));
        let left = self.map.range_mut(..=&value).last();

        #[cfg(debug_assertions)]
        {
            self.reference_set.insert(value.clone());
        }

        let b = match (left, right) {
            (None, None) => {
                // nothing found so just insert ourselves
                self.map.insert(value, 1);
                true
            }
            (None, Some(v)) => {
                // so right that we can extend so merge that with our new value
                self.map.insert(value, v + 1);
                true
            }
            (Some((k, v)), None) => {
                match (k.at(*v)).cmp(&value) {
                    std::cmp::Ordering::Less => {
                        // can't extend the existing range so just add our own
                        self.map.insert(value, 1);
                        true
                    }
                    std::cmp::Ordering::Equal => {
                        // extend the existing range
                        *v += 1;
                        true
                    }
                    std::cmp::Ordering::Greater => {
                        // already included in the range
                        false
                    }
                }
            }
            (Some((lk, lv)), Some(rv)) => {
                if lk.at(*lv) == value {
                    *lv += 1 + rv;
                    true
                } else {
                    self.map.insert(value, 1 + rv);
                    true
                }
            }
        };
        #[cfg(debug_assertions)]
        {
            assert_eq!(self.reference_set, self.iter().collect());
            // println!("rleset space: {:?}", self.space_comparison());
        }
        b
    }

    pub fn len(&self) -> usize {
        self.map.values().map(|v| *v as usize).sum()
    }

    // TODO: test that this is the same as doing individual inserts
    /// Returns the number of insertions
    fn insert_run(&mut self, value: T, length: u64) -> usize {
        let right = self
            .map
            .range(&value..=&(value.at(length)))
            .map(|(a, b)| (a.clone(), *b))
            .collect::<Vec<_>>();
        let mut last_right = None;
        for (range, count) in right {
            self.map.remove(&range);
            last_right = Some((range, count));
        }

        let left = self
            .map
            .range(..=&value)
            .last()
            .map(|(a, b)| (a.clone(), *b));

        match (left, last_right) {
            (None, None) => {
                // nothing found so just insert ourselves
                self.map.insert(value, length);
                length as usize
            }
            (None, Some((v, c))) => {
                // so right that we can extend so merge that with our new value
                self.map
                    .insert(value.clone(), c + length - (v.sub(&value.at(length))));
                length as usize // FIXME
            }
            (Some((k, v)), None) => {
                match (k.at(v)).cmp(&value) {
                    std::cmp::Ordering::Less => {
                        // can't extend the existing range so just add our own
                        self.map.insert(value, length);
                        length as usize // FIXME
                    }
                    std::cmp::Ordering::Equal => {
                        // extend the existing range
                        self.map.insert(k, v + length);
                        length as usize // FIXME
                    }
                    std::cmp::Ordering::Greater => {
                        // may already included in the range
                        self.map
                            .insert(value.clone(), v + length - (k.sub(&value.at(length))));
                        length as usize // FIXME
                    }
                }
            }
            (Some((lk, lv)), Some((rk, rv))) => {
                if lk.at(lv) >= value {
                    self.map.insert(
                        lk.clone(),
                        lv + length + rv
                            - (lk.sub(&value.at(length)))
                            - (rk.sub(&value.at(length))),
                    );
                    length as usize // FIXME
                } else {
                    self.map.insert(value, length + rv);
                    length as usize // FIXME
                }
            }
        }
    }

    #[cfg(debug_assertions)]
    pub fn space_comparison(&self) -> (usize, usize, f64) {
        use std::mem::size_of;
        let map_size = size_of::<BTreeMap<u64, u64>>() + (size_of::<u64>() * self.map.len() * 2);
        let set_size = size_of::<HashSet<u64>>() + (size_of::<u64>() * self.reference_set.len());
        (map_size, set_size, map_size as f64 / set_size as f64)
    }

    pub fn remove(&mut self, value: &T) {
        let left = self.map.range_mut(..=value).last();
        if let Some((k, v)) = left {
            if k == value {
                // start of the range
                let k = k.clone();
                let v = *v;
                self.map.insert(value.next(), v - 1);
                self.map.remove(&k);
            } else if k.at(*v - 1) == *value {
                // end of the range
                *v -= 1;
            } else if k.at(*v) >= *value {
                // middle of the range
                let left = value.sub(&k);
                let right = k.at(*v - 1).sub(value);
                *v = left;
                self.map.insert(value.next(), right);
            }
        }
        #[cfg(debug_assertions)]
        {
            self.reference_set.remove(value);
            assert_eq!(self.reference_set, self.iter().collect());
        }
    }

    pub fn contains(&self, value: T) -> bool {
        self.map
            .range(..=&value)
            .last()
            .map_or(false, |(k, v)| k.at(*v) > value)
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = T> + '_ {
        self.map
            .iter()
            .map(|(start, length)| (0..*length).map(|t| start.at(t)))
            .flatten()
    }

    /// Merges the other set into this one, returning the number of new items inserted.
    pub fn merge(&mut self, other: &Self) -> usize {
        if self.is_empty() {
            *self = other.clone();
            other.len()
        } else {
            let mut count = 0;
            for (value, length) in &other.map {
                count += self.insert_run(value.clone(), *length);
            }
            count
        }
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

        s.remove(&6);
        map.insert(5, 1);
        map.insert(7, 2);
        assert_eq!(s.map, map);

        s.remove(&3);
        map.insert(1, 2);
        assert_eq!(s.map, map);

        s.remove(&1);
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
