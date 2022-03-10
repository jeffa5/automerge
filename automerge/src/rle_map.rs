use std::collections::BTreeMap;
#[cfg(debug_assertions)]
use std::collections::HashMap;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RleMap {
    /// mapping from start of range to end of the range
    map: BTreeMap<u64, Vec<usize>>,
    #[cfg(debug_assertions)]
    reference_map: HashMap<u64, usize>,
}

impl RleMap {
    pub fn get(&self, counter: u64) -> Option<&usize> {
        let left = self.map.range(..=counter).last();
        match left {
            Some((k, v)) => {
                if k + v.len() as u64 > counter {
                    v.get((counter - k) as usize)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    pub fn insert(&mut self, counter: u64, value: usize) {
        // get iterator at point of this value
        let right = self.map.remove(&(counter + 1));
        let left = self
            .map
            .range(..=counter)
            .last()
            .map(|(a, b)| (*a, b.len()));

        match (left, right) {
            (None, None) => {
                // nothing found so just insert ourselves
                self.map.insert(counter, vec![value]);
            }
            (None, Some(mut v)) => {
                // so right that we can extend so merge that with our new value
                v.insert(0, value);
                self.map.insert(counter, v);
            }
            (Some((k, v_len)), None) => {
                match (k + v_len as u64).cmp(&counter) {
                    std::cmp::Ordering::Less => {
                        // can't extend the existing range so just add our own
                        self.map.insert(counter, vec![value]);
                    }
                    std::cmp::Ordering::Equal => {
                        // extend the existing range
                        let v = self.map.get_mut(&k).unwrap();
                        v.insert((counter - k) as usize, value);
                    }
                    std::cmp::Ordering::Greater => {
                        // already in the range
                    }
                }
            }
            (Some((lk, lv_len)), Some(mut rv)) => {
                if lk + lv_len as u64 == counter {
                    let lv = self.map.get_mut(&lk).unwrap();
                    lv.push(value);
                    lv.append(&mut rv);
                } else {
                    rv.insert(0, value);
                    self.map.insert(counter, rv);
                }
            }
        }
        #[cfg(debug_assertions)]
        {
            self.reference_map.insert(counter, value);
            assert_eq!(self.reference_map, self.iter().collect());
        }
    }

    pub fn num_keys(&self) -> usize {
        self.map.len()
    }

    pub fn remove(&mut self, counter: u64) {
        let left = self
            .map
            .range(..=counter)
            .last()
            .map(|(a, b)| (*a, b.len()));
        if let Some((k, v_len)) = left {
            if k == counter {
                // start of the range
                let mut v = self.map.remove(&k).unwrap();
                v.remove(0);
                self.map.insert(counter + 1, v);
            } else if k + v_len as u64 - 1 == counter {
                // end of the range
                let v = self.map.get_mut(&k).unwrap();
                v.remove(v.len() - 1);
            } else if k + v_len as u64 >= counter {
                // middle of the range
                let v = self.map.get_mut(&k).unwrap();
                let right = v.split_off((counter - k + 1).try_into().unwrap());
                v.remove(v.len() - 1);
                self.map.insert(counter + 1, right);
            }
        }
        #[cfg(debug_assertions)]
        {
            self.reference_map.remove(&counter);
            assert_eq!(self.reference_map, self.iter().collect());
        }
    }

    pub fn contains_key(&self, counter: u64) -> bool {
        let left = self.map.range(..=counter).last();
        match left {
            Some((k, v)) => k + v.len() as u64 > counter,
            None => false,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (u64, usize)> + '_ {
        self.map
            .iter()
            .map(|(start, values)| {
                values
                    .iter()
                    .enumerate()
                    .map(move |(i, v)| (start + i as u64, *v))
            })
            .flatten()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn basic_insert() {
        let mut s = RleMap::default();
        s.insert(1, 1);
        s.insert(2, 2);
        s.insert(5, 5);
        s.insert(7, 7);
        s.insert(8, 8);
        let mut map = BTreeMap::new();
        map.insert(1, vec![1, 2]);
        map.insert(5, vec![5]);
        map.insert(7, vec![7, 8]);
        assert_eq!(s.map, map);

        s.insert(3, 3);
        map.insert(1, vec![1, 2, 3]);
        assert_eq!(s.map, map);

        s.insert(6, 6);
        map.insert(5, vec![5, 6, 7, 8]);
        map.remove(&7);
        assert_eq!(s.map, map);

        s.insert(15, 15);
        map.insert(15, vec![15]);
        assert_eq!(s.map, map);

        s.insert(14, 14);
        map.insert(14, vec![14, 15]);
        map.remove(&15);
        assert_eq!(s.map, map);

        let mut s = RleMap::default();
        s.insert(5, 5);
        let mut map = BTreeMap::new();
        map.insert(5, vec![5]);
        assert_eq!(s.map, map);

        s.insert(4, 4);
        map.insert(4, vec![4, 5]);
        map.remove(&5);
        assert_eq!(s.map, map);
    }

    #[test]
    fn duplicate_insert() {
        let mut s = RleMap::default();
        s.insert(1, 1);
        s.insert(2, 2);
        s.insert(5, 5);
        s.insert(7, 7);
        s.insert(8, 8);
        let mut map = BTreeMap::new();
        map.insert(1, vec![1, 2]);
        map.insert(5, vec![5]);
        map.insert(7, vec![7, 8]);
        assert_eq!(s.map, map);

        s.insert(1, 1);
        assert_eq!(s.map, map);
        s.insert(2, 2);
        assert_eq!(s.map, map);
        s.insert(5, 5);
        assert_eq!(s.map, map);
        s.insert(7, 7);
        assert_eq!(s.map, map);
        s.insert(8, 8);
        assert_eq!(s.map, map);
    }

    #[test]
    fn basic_remove() {
        let mut s = RleMap::default();
        s.insert(1, 1);
        s.insert(2, 2);
        s.insert(5, 5);
        s.insert(7, 7);
        s.insert(8, 8);
        s.insert(3, 3);
        s.insert(6, 6);

        let mut map = BTreeMap::new();
        map.insert(1, vec![1, 2, 3]);
        map.insert(5, vec![5, 6, 7, 8]);
        assert_eq!(s.map, map);

        s.remove(6);
        map.insert(5, vec![5]);
        map.insert(7, vec![7, 8]);
        assert_eq!(s.map, map);

        s.remove(3);
        map.insert(1, vec![1, 2]);
        assert_eq!(s.map, map);

        s.remove(1);
        map.insert(2, vec![2]);
        map.remove(&1);
        assert_eq!(s.map, map);
    }

    #[test]
    fn basic_iter() {
        let mut s = RleMap::default();
        s.insert(1, 1);
        s.insert(2, 2);
        s.insert(5, 5);
        s.insert(7, 7);
        s.insert(8, 8);
        s.insert(3, 3);
        s.insert(6, 6);
        s.insert(15, 15);
        s.insert(14, 14);

        assert_eq!(
            s.iter().collect::<Vec<_>>(),
            vec![
                (1, 1),
                (2, 2),
                (3, 3),
                (5, 5),
                (6, 6),
                (7, 7),
                (8, 8),
                (14, 14),
                (15, 15)
            ]
        );
    }
}
