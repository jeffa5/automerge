use std::collections::HashMap;

use fxhash::FxBuildHasher;

use crate::{
    opid_set::OpIdSet,
    rle_map::RleMap,
    types::{ElemId, OpId},
};

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct VisibleElemIdMap {
    // a value that is a default value (1) gets put in the set
    opset: OpIdSet,
    // a value that is not the default (1) gets put in the map instead of the set
    map: HashMap<usize, RleMap, FxBuildHasher>,
    #[cfg(debug_assertions)]
    reference_map: HashMap<ElemId, usize>,
}

impl VisibleElemIdMap {
    pub fn get(&self, opid: &ElemId) -> Option<&usize> {
        if self.opset.contains(&opid.0) {
            Some(&1)
        } else {
            self.map
                .get(&opid.0.actor())
                .and_then(|map| map.get(opid.0.counter()))
        }
    }

    pub fn insert(&mut self, opid: ElemId, value: usize) {
        if value == 1 {
            self.opset.insert(opid.0);
        } else {
            self.map
                .entry(opid.0.actor())
                .or_default()
                .insert(opid.0.counter(), value);
        }
        #[cfg(debug_assertions)]
        {
            self.reference_map.insert(opid, value);
            assert_eq!(self.reference_map, self.iter().collect());
        }
    }

    pub fn remove(&mut self, opid: &ElemId) {
        if !self.opset.remove(&opid.0) {
            if let Some(set) = self.map.get_mut(&opid.0.actor()) {
                set.remove(opid.0.counter());
                if set.is_empty() {
                    self.map.remove(&opid.0.actor());
                }
            }
        }
        #[cfg(debug_assertions)]
        {
            self.reference_map.remove(opid);
            assert_eq!(self.reference_map, self.iter().collect());
        }
    }

    pub fn contains_key(&self, opid: &ElemId) -> bool {
        let b = self.opset.contains(&opid.0)
            || self
                .map
                .get(&opid.0.actor())
                .map_or(false, |set| set.contains_key(opid.0.counter()));
        #[cfg(debug_assertions)]
        {
            assert_eq!(b, self.reference_map.contains_key(opid), "{:?}", self);
        }
        b
    }

    pub fn iter(&self) -> impl Iterator<Item = (ElemId, usize)> + '_ {
        self.opset.iter().map(|opid| (ElemId(opid), 1)).chain(
            self.map
                .iter()
                .map(|(actor, set)| {
                    set.iter()
                        .map(|(counter, value)| (ElemId(OpId(counter, *actor)), value))
                })
                .flatten(),
        )
    }
}
