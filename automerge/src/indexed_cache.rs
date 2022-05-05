use itertools::Itertools;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Index;

#[derive(Debug, Clone)]
pub(crate) struct IndexedCache<T> {
    pub(crate) cache: Vec<T>,
    lookup: HashMap<T, usize>,
}

impl<T> PartialEq for IndexedCache<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.cache == other.cache
    }
}

impl<T> IndexedCache<T>
where
    T: Clone + Eq + Hash + Ord,
{
    pub(crate) fn new() -> Self {
        IndexedCache {
            cache: Default::default(),
            lookup: Default::default(),
        }
    }

    pub(crate) fn cache(&mut self, item: T) -> usize {
        match self.lookup.entry(item.clone()) {
            std::collections::hash_map::Entry::Occupied(e) => *e.get(),
            std::collections::hash_map::Entry::Vacant(e) => {
                let n = self.cache.len();
                self.cache.push(item);
                e.insert(n);
                n
            }
        }
    }

    pub(crate) fn lookup(&self, item: &T) -> Option<usize> {
        self.lookup.get(item).cloned()
    }

    pub(crate) fn len(&self) -> usize {
        self.cache.len()
    }

    pub(crate) fn get(&self, index: usize) -> &T {
        &self.cache[index]
    }

    /// Remove the last inserted entry into this cache.
    /// This is safe to do as it does not require reshuffling other entries.
    ///
    /// # Panics
    ///
    /// Panics on an empty cache.
    pub(crate) fn remove_last(&mut self) -> T {
        let last = self.cache.len() - 1;
        let t = self.cache.remove(last);
        self.lookup.remove(&t);
        t
    }

    pub(crate) fn sorted(&self) -> IndexedCache<T> {
        let mut sorted = Self::new();
        self.cache.iter().sorted().cloned().for_each(|item| {
            let n = sorted.cache.len();
            sorted.cache.push(item.clone());
            sorted.lookup.insert(item, n);
        });
        sorted
    }

    pub(crate) fn encode_index(&self) -> Vec<usize> {
        let sorted: Vec<_> = self.cache.iter().sorted().cloned().collect();
        self.cache
            .iter()
            .map(|a| sorted.iter().position(|r| r == a).unwrap())
            .collect()
    }
}

impl<T> IntoIterator for IndexedCache<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.cache.into_iter()
    }
}

impl<T> Index<usize> for IndexedCache<T> {
    type Output = T;
    fn index(&self, i: usize) -> &T {
        &self.cache[i]
    }
}
