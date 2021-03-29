use automerge::{LocalChange, Path, Primitive, Value};
use automerge_protocol::Patch;
use criterion::{criterion_group, criterion_main, Criterion};

fn create_new_frontend(c: &mut Criterion) {
    c.bench_function("create a new frontend", |b| {
        b.iter(automerge::Frontend::new)
    });
}

fn make_patch_from_n_changes(n: u64) -> Patch {
    let mut backend = automerge::Backend::init();
    let mut frontend = automerge::Frontend::new();
    for i in 0..n {
        let change = frontend
            .change(None, |doc| {
                doc.add_change(LocalChange::set(
                    Path::root().key("a"),
                    Value::Primitive(Primitive::Uint(i)),
                ))
            })
            .unwrap()
            .unwrap();
        let (patch, _) = backend.apply_local_change(change).unwrap();
        frontend.apply_patch(patch).unwrap();
    }
    backend.get_patch().unwrap()
}

fn create_new_frontend_1(c: &mut Criterion) {
    c.bench_function("create a new frontend with 1 change", |b| {
        b.iter_batched(
            || make_patch_from_n_changes(1),
            |patch| {
                let mut frontend = automerge::Frontend::new();
                frontend.apply_patch(patch).unwrap()
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

fn create_new_frontend_10(c: &mut Criterion) {
    c.bench_function("create a new frontend with 10 changes", |b| {
        b.iter_batched(
            || make_patch_from_n_changes(10),
            |patch| {
                let mut frontend = automerge::Frontend::new();
                frontend.apply_patch(patch).unwrap()
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

fn create_new_frontend_1000(c: &mut Criterion) {
    c.bench_function("create a new frontend with 1000 changes", |b| {
        b.iter_batched(
            || make_patch_from_n_changes(1000),
            |patch| {
                let mut frontend = automerge::Frontend::new();
                frontend.apply_patch(patch).unwrap()
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(100);
    targets = create_new_frontend, create_new_frontend_1, create_new_frontend_10, create_new_frontend_1000
}
criterion_main!(benches);
