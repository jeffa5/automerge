use automerge::Backend;
use automerge::Frontend;
use automerge::InvalidChangeRequest;
use automerge::LocalChange;
use automerge::MapType;
use automerge::Path;
use automerge::Primitive;
use automerge::Value;
use pretty_assertions::assert_eq;

use std::collections::HashMap;

#[test]
fn broken_reordering_of_values() {
    // setup
    let mut hm = HashMap::new();
    hm.insert(
        "".to_owned(),
        Value::Sequence(vec![Value::Primitive(Primitive::Null)]),
    );
    let mut backend = Backend::init();

    // new frontend with initial state
    let (mut frontend, change) =
        Frontend::new_with_initial_state(Value::Map(hm, MapType::Map)).unwrap();

    println!("change1 {:?}", change);

    // get patch and apply
    let (patch, _) = backend.apply_local_change(change).unwrap();
    frontend.apply_patch(patch).unwrap();

    // change first value and insert into the sequence
    let c = frontend
        .change::<_, InvalidChangeRequest>(None, |d| {
            d.add_change(LocalChange::set(
                Path::root().key("").index(0),
                Value::Primitive(Primitive::Int(0)),
            ))
            .unwrap();
            d.add_change(LocalChange::insert(
                Path::root().key("").index(1),
                Value::Primitive(Primitive::Boolean(false)),
            ))
            .unwrap();
            Ok(())
        })
        .unwrap();

    // setup first expected
    let mut ehm = HashMap::new();
    ehm.insert(
        "".to_owned(),
        Value::Sequence(vec![
            Value::Primitive(Primitive::Int(0)),
            Value::Primitive(Primitive::Boolean(false)),
        ]),
    );
    let expected = Value::Map(ehm.clone(), MapType::Map);

    // ok, sequence has int then bool
    assert_eq!(expected, frontend.get_value(&Path::root()).unwrap());

    // now apply the change to the backend and bring the patch back to the frontend
    if let Some(c) = c {
        println!("change2 {:?}", c);
        let (p, _) = backend.apply_local_change(c).unwrap();
        println!("patch {:#?}", p);
        frontend.apply_patch(p).unwrap();
    }
    let v = frontend.get_value(&Path::root()).unwrap();

    let expected = Value::Map(ehm, MapType::Map);
    // not ok! sequence has bool then int
    assert_eq!(expected, v);
}
