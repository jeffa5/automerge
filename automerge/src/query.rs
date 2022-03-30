use crate::op_tree::{OpSetMetadata, OpTreeNode};
use crate::opid_set::OpIdSet;
use crate::types::{Clock, Counter, ElemId, Op, OpId, OpType, ScalarValue};
use fxhash::FxBuildHasher;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;

mod insert;
mod keys;
mod keys_at;
mod len;
mod len_at;
mod list_vals;
mod list_vals_at;
mod nth;
mod nth_at;
mod opid;
mod prop;
mod prop_at;
mod seek_op;

pub(crate) use insert::InsertNth;
pub(crate) use keys::Keys;
pub(crate) use keys_at::KeysAt;
pub(crate) use len::Len;
pub(crate) use len_at::LenAt;
pub(crate) use list_vals::ListVals;
pub(crate) use list_vals_at::ListValsAt;
pub(crate) use nth::Nth;
pub(crate) use nth_at::NthAt;
pub(crate) use opid::OpIdSearch;
pub(crate) use prop::Prop;
pub(crate) use prop_at::PropAt;
pub(crate) use seek_op::SeekOp;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CounterData {
    pos: usize,
    val: i64,
    succ: HashSet<OpId>,
    op: Op,
}

pub(crate) trait TreeQuery<const B: usize> {
    #[inline(always)]
    fn query_node_with_metadata(
        &mut self,
        child: &OpTreeNode<B>,
        _m: &OpSetMetadata,
    ) -> QueryResult {
        self.query_node(child)
    }

    fn query_node(&mut self, _child: &OpTreeNode<B>) -> QueryResult {
        QueryResult::Descend
    }

    #[inline(always)]
    fn query_element_with_metadata(&mut self, element: &Op, _m: &OpSetMetadata) -> QueryResult {
        self.query_element(element)
    }

    fn query_element(&mut self, _element: &Op) -> QueryResult {
        panic!("invalid element query")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum QueryResult {
    Next,
    Descend,
    Finish,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Index {
    /// The map of visible elements to the number of operations targetting them.
    pub visible: HashMap<ElemId, usize, FxBuildHasher>,
    /// Set of opids found in this node and below.
    pub ops: OpIdSet,
}

impl Index {
    pub fn new() -> Self {
        Index {
            visible: Default::default(),
            ops: Default::default(),
        }
    }

    /// Get the number of visible elements in this index.
    pub fn visible_len(&self) -> usize {
        self.visible.len()
    }

    pub fn has_visible(&self, e: &Option<ElemId>) -> bool {
        if let Some(seen) = e {
            self.visible.contains_key(seen)
        } else {
            false
        }
    }

    pub fn replace(&mut self, old: &Op, new: &Op) {
        if old.id != new.id {
            self.ops.remove(&old.id);
            self.ops.insert(new.id);
        }

        assert!(new.key == old.key);

        match (new.visible(), old.visible(), new.elemid()) {
            (false, true, Some(elem)) => match self.visible.get(&elem).copied() {
                Some(n) if n == 1 => {
                    self.visible.remove(&elem);
                }
                Some(n) => {
                    self.visible.insert(elem, n - 1);
                }
                None => panic!("remove overun in index"),
            },
            (true, false, Some(elem)) => *self.visible.entry(elem).or_default() += 1,
            _ => {}
        }
    }

    pub fn insert(&mut self, op: &Op) {
        self.ops.insert(op.id);
        if op.visible() {
            if let Some(elem) = op.elemid() {
                *self.visible.entry(elem).or_default() += 1;
            }
        }
    }

    pub fn remove(&mut self, op: &Op) {
        self.ops.remove(&op.id);
        if op.visible() {
            if let Some(elem) = op.elemid() {
                match self.visible.get(&elem).copied() {
                    Some(n) if n == 1 => {
                        self.visible.remove(&elem);
                    }
                    Some(n) => {
                        self.visible.insert(elem, n - 1);
                    }
                    None => panic!("remove overun in index"),
                }
            }
        }
    }

    pub fn merge(&mut self, other: &Index) {
        for id in other.ops.iter() {
            self.ops.insert(id);
        }
        for (elem, n) in other.visible.iter() {
            *self.visible.entry(*elem).or_default() += n;
        }
    }
}

impl Default for Index {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct VisWindow {
    counters: HashMap<OpId, CounterData>,
}

impl VisWindow {
    fn visible_at(&mut self, op: &Op, pos: usize, clock: &Clock) -> bool {
        if !clock.covers(&op.id) {
            return false;
        }

        let mut visible = false;
        match op.action {
            OpType::Set(ScalarValue::Counter(Counter { start, .. })) => {
                self.counters.insert(
                    op.id,
                    CounterData {
                        pos,
                        val: start,
                        succ: op.succ.iter().cloned().collect(),
                        op: op.clone(),
                    },
                );
                if !op.succ.iter().any(|i| clock.covers(i)) {
                    visible = true;
                }
            }
            OpType::Inc(inc_val) => {
                for id in &op.pred {
                    // pred is always before op.id so we can see them
                    if let Some(mut entry) = self.counters.get_mut(id) {
                        entry.succ.remove(&op.id);
                        entry.val += inc_val;
                        entry.op.action = OpType::Set(ScalarValue::counter(entry.val));
                        if !entry.succ.iter().any(|i| clock.covers(i)) {
                            visible = true;
                        }
                    }
                }
            }
            _ => {
                if !op.succ.iter().any(|i| clock.covers(i)) {
                    visible = true;
                }
            }
        };
        visible
    }

    pub fn seen_op(&self, op: &Op, pos: usize) -> Vec<(usize, Op)> {
        let mut result = vec![];
        for pred in &op.pred {
            if let Some(entry) = self.counters.get(pred) {
                result.push((entry.pos, entry.op.clone()));
            }
        }
        if result.is_empty() {
            vec![(pos, op.clone())]
        } else {
            result
        }
    }
}

pub(crate) fn binary_search_by<F, const B: usize>(node: &OpTreeNode<B>, f: F) -> usize
where
    F: Fn(&Op) -> Ordering,
{
    let mut right = node.len();
    let mut left = 0;
    while left < right {
        let seq = (left + right) / 2;
        if f(node.get(seq).unwrap()) == Ordering::Less {
            left = seq + 1;
        } else {
            right = seq;
        }
    }
    left
}
