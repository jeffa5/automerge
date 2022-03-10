use std::collections::HashMap;
#[cfg(debug_assertions)]
use std::collections::HashSet;

use fxhash::FxBuildHasher;

use crate::{rle_set::RleSet, types::OpId};

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct OpIdSet {
    map: HashMap<usize, RleSet, FxBuildHasher>,
    #[cfg(debug_assertions)]
    set: HashSet<OpId>,
}

impl OpIdSet {
    pub fn insert(&mut self, opid: OpId) {
        self.map
            .entry(opid.actor())
            .or_default()
            .insert(opid.counter());
        #[cfg(debug_assertions)]
        {
            self.set.insert(opid);
            assert_eq!(self.set, self.iter().collect());
        }
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
            self.set.remove(opid);
            assert_eq!(self.set, self.iter().collect());
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
            assert_eq!(b, self.set.contains(opid), "{:?}", self);
        }
        b
    }

    pub fn iter(&self) -> impl Iterator<Item = OpId> + '_ {
        self.map
            .iter()
            .map(|(actor, set)| set.iter().map(|counter| OpId(counter, *actor)))
            .flatten()
    }
}
