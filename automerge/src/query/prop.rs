use crate::op_tree::{OpSetMetadata, OpTreeNode};
use crate::query::{binary_search_by, binary_search_by_in, QueryResult, TreeQuery};
use crate::types::{Key, Op};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Prop<'a> {
    key: Key,
    single: bool,
    pub(crate) op: Option<&'a Op>,
    pub(crate) op_pos: Option<usize>,
    pub(crate) ops: Vec<&'a Op>,
    pub(crate) ops_pos: Vec<usize>,
    pub(crate) pos: usize,
    start: Option<usize>,
    done_root: bool,
    cached_value: Option<(Key, usize)>,
}

impl<'a> Prop<'a> {
    pub(crate) fn new(prop: usize, single: bool) -> Self {
        Prop {
            key: Key::Map(prop),
            single,
            op: None,
            op_pos: None,
            ops: vec![],
            ops_pos: vec![],
            pos: 0,
            start: None,
            done_root: false,
            cached_value: None,
        }
    }
}

impl<'a> TreeQuery<'a> for Prop<'a> {
    fn cache_lookup_map(&mut self, _cache: &crate::object_data::MapOpsCache) -> bool {
        // FIXME: should be able to cache this and the insert position
        // self.cached_value = cache.last;
        // don't have all of the result yet
        false
    }

    fn cache_update_map(&self, cache: &mut crate::object_data::MapOpsCache) {
        cache.last = self.start.map(|start| (self.key, start));
    }

    fn query_node_with_metadata(
        &mut self,
        child: &'a OpTreeNode,
        m: &OpSetMetadata,
    ) -> QueryResult {
        if self.done_root {
            if self.pos + child.len() >= self.start.expect("should have generated start by now") {
                // skip empty nodes
                if child.index.visible_len() == 0 {
                    self.pos += child.len();
                    QueryResult::Next
                } else {
                    QueryResult::Descend
                }
            } else {
                self.pos += child.len();
                QueryResult::Next
            }
        } else {
            self.done_root = true;

            // in the root node find the first op position for the key
            let start = if let Some((key, index)) = self.cached_value {
                // using cached start value
                match m.key_cmp(&key, &self.key) {
                    std::cmp::Ordering::Less => {
                        // cached value was for something less, use as lower bound
                        binary_search_by_in(
                            child,
                            |op| m.key_cmp(&op.key, &self.key),
                            index,
                            child.len(),
                        )
                    }
                    std::cmp::Ordering::Equal => index,
                    std::cmp::Ordering::Greater => {
                        // cached value was for something greater, use as upper bound
                        binary_search_by_in(child, |op| m.key_cmp(&op.key, &self.key), 0, index)
                    }
                }
            } else {
                // no valid cached start so find it again
                binary_search_by(child, |op| m.key_cmp(&op.key, &self.key))
            };
            self.start = Some(start);
            self.pos = start;
            QueryResult::Skip(start)
        }
    }

    fn query_element(&mut self, op: &'a Op) -> QueryResult {
        // don't bother looking at things past our key
        if op.key != self.key {
            return QueryResult::Finish;
        }
        if op.visible() {
            if self.single {
                self.op = Some(op);
                self.op_pos = Some(self.pos);
            } else {
                self.ops.push(op);
                self.ops_pos.push(self.pos);
            }
        }
        self.pos += 1;
        QueryResult::Next
    }
}
