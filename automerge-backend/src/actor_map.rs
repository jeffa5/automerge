use std::{cmp::Ordering, collections::HashMap};

use automerge_protocol as amp;

use crate::internal::{ActorId, ElementId, InternalOp, InternalOpType, Key, ObjectId, OpId};

#[derive(PartialEq, Debug, Clone)]
pub(crate) struct ActorMap {
    id_to_index: HashMap<amp::ActorId, usize>,
    index_to_id: HashMap<usize, amp::ActorId>,
}

impl ActorMap {
    pub fn new() -> ActorMap {
        ActorMap {
            id_to_index: HashMap::new(),
            index_to_id: HashMap::new(),
        }
    }

    pub fn import_key(&mut self, key: &amp::Key) -> Key {
        match key {
            amp::Key::Map(string) => Key::Map(string.to_string()),
            amp::Key::Seq(eid) => Key::Seq(self.import_element_id(eid)),
        }
    }

    pub fn import_actor(&mut self, actor: &amp::ActorId) -> ActorId {
        if let Some(idx) = self.id_to_index.get(actor) {
            ActorId(*idx)
        } else {
            let index = self.id_to_index.len();
            self.id_to_index.insert(actor.clone(), index);
            self.index_to_id.insert(index, actor.clone());
            ActorId(index)
        }
    }

    pub fn import_opid(&mut self, opid: &amp::OpId) -> OpId {
        OpId(opid.0, self.import_actor(&opid.1))
    }

    pub fn import_obj(&mut self, obj: &amp::ObjectId) -> ObjectId {
        match obj {
            amp::ObjectId::Root => ObjectId::Root,
            amp::ObjectId::Id(ref opid) => ObjectId::Id(self.import_opid(opid)),
        }
    }

    pub fn import_element_id(&mut self, eid: &amp::ElementId) -> ElementId {
        match eid {
            amp::ElementId::Head => ElementId::Head,
            amp::ElementId::Id(ref opid) => ElementId::Id(self.import_opid(opid)),
        }
    }

    pub fn import_op(&mut self, op: amp::Op) -> InternalOp {
        InternalOp {
            action: Self::import_optype(&op.action),
            obj: self.import_obj(&op.obj),
            key: self.import_key(&op.key),
            pred: op
                .pred
                .into_iter()
                .map(|ref id| self.import_opid(id))
                .collect(),
            insert: op.insert,
        }
    }

    pub fn import_optype(optype: &amp::OpType) -> InternalOpType {
        match optype {
            amp::OpType::Make(val) => InternalOpType::Make(*val),
            amp::OpType::Del => InternalOpType::Del,
            amp::OpType::Inc(val) => InternalOpType::Inc(*val),
            amp::OpType::Set(val) => InternalOpType::Set(val.clone()),
        }
    }

    pub fn export_actor(&self, actor: ActorId) -> amp::ActorId {
        self.index_to_id[&actor.0].clone()
    }

    pub fn export_opid(&self, opid: &OpId) -> amp::OpId {
        amp::OpId(opid.0, self.export_actor(opid.1))
    }

    pub fn export_obj(&self, obj: &ObjectId) -> amp::ObjectId {
        match obj {
            ObjectId::Root => amp::ObjectId::Root,
            ObjectId::Id(opid) => amp::ObjectId::Id(self.export_opid(opid)),
        }
    }

    pub fn cmp(&self, eid1: &ElementId, eid2: &ElementId) -> Ordering {
        match (eid1, eid2) {
            (ElementId::Head, ElementId::Head) => Ordering::Equal,
            (ElementId::Head, _) => Ordering::Less,
            (_, ElementId::Head) => Ordering::Greater,
            (ElementId::Id(opid1), ElementId::Id(opid2)) => self.cmp_opid(opid1, opid2),
        }
    }

    pub fn opid_to_string(&self, id: &OpId) -> String {
        format!("{}@{}", id.0, self.export_actor(id.1).to_hex_string())
    }

    pub fn elementid_to_string(&self, eid: &ElementId) -> String {
        match eid {
            ElementId::Head => "_head".into(),
            ElementId::Id(id) => self.opid_to_string(id),
        }
    }

    pub fn key_to_string(&self, key: &Key) -> String {
        match &key {
            Key::Map(s) => s.clone(),
            Key::Seq(eid) => self.elementid_to_string(eid),
        }
    }

    fn cmp_opid(&self, op1: &OpId, op2: &OpId) -> Ordering {
        if op1.0 == op2.0 {
            let actor1 = &self.index_to_id[&(op1.1).0];
            let actor2 = &self.index_to_id[&(op2.1).0];
            actor1.cmp(actor2)
            //op1.1.cmp(&op2.1)
        } else {
            op1.0.cmp(&op2.0)
        }
    }
}
