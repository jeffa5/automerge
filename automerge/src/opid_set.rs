use std::collections::HashMap;
#[cfg(debug_assertions)]
use std::collections::HashSet;

use fxhash::FxBuildHasher;

use crate::{rle_set::RleSet, types::OpId};

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct OpIdSet {
    map: HashMap<usize, RleSet<u64>, FxBuildHasher>,
    #[cfg(debug_assertions)]
    reference_set: HashSet<OpId>,
}

impl OpIdSet {
    pub fn insert(&mut self, opid: OpId) -> bool {
        let b = self
            .map
            .entry(opid.actor())
            .or_default()
            .insert(opid.counter());
        #[cfg(debug_assertions)]
        {
            self.reference_set.insert(opid);
            assert_eq!(self.reference_set, self.iter().collect());
            // println!("opidset space: {:?}", self.space_comparison());
        }
        b
    }

    #[cfg(debug_assertions)]
    pub fn space_comparison(&self) -> (usize, usize, f64) {
        use std::mem::size_of;
        let map_size = size_of::<HashMap<usize, RleSet<u64>, FxBuildHasher>>()
            + (self
                .map
                .iter()
                .map(|(_, set)| (size_of::<usize>() + set.space_comparison().0))
                .sum::<usize>());
        let set_size = size_of::<HashSet<OpId>>() + (size_of::<OpId>() * self.reference_set.len());
        (map_size, set_size, map_size as f64 / set_size as f64)
    }

    /// Remove an opid from this set, returns whether it was present.
    pub fn remove(&mut self, opid: &OpId) -> bool {
        let mut present = false;
        if let Some(set) = self.map.get_mut(&opid.actor()) {
            present = true;
            set.remove(&opid.counter());
            if set.is_empty() {
                self.map.remove(&opid.actor());
            }
        }
        #[cfg(debug_assertions)]
        {
            self.reference_set.remove(opid);
            assert_eq!(self.reference_set, self.iter().collect());
        }
        present
    }

    pub fn contains(&self, opid: &OpId) -> bool {
        let b = self
            .map
            .get(&opid.actor())
            .map_or(false, |set| set.contains(opid.counter()));
        #[cfg(debug_assertions)]
        {
            assert_eq!(b, self.reference_set.contains(opid), "{:?}", self);
        }
        b
    }

    pub fn iter(&self) -> impl Iterator<Item = OpId> + '_ {
        self.map
            .iter()
            .map(|(actor, set)| set.iter().map(|counter| OpId(counter, *actor)))
            .flatten()
    }

    pub fn merge(&mut self, other: &Self) {
        for (actor, other_rleset) in other.map.iter() {
            if let Some(our_rleset) = self.map.get_mut(actor) {
                our_rleset.merge(other_rleset);
            } else {
                self.map.insert(*actor, other_rleset.clone());
            }
        }
    }
}
