use automerge::{Backend, Frontend, InvalidChangeRequest, LocalChange, Path, Primitive, Value};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn change_backend(size: usize) -> Backend {
    let mut frontends = (0..10).map(|_| Frontend::new()).collect::<Vec<_>>();
    let mut backend = Backend::init();
    for i in 0..size {
        let (_, change) = frontends[i % 10]
            .change::<_, _, InvalidChangeRequest>(None, |doc| {
                doc.add_change(LocalChange::set(
                    Path::root().key(i.to_string()),
                    Value::Primitive(Primitive::Str(i.to_string())),
                ))?;
                Ok(())
            })
            .unwrap();
        backend.apply_local_change(change.unwrap()).unwrap();
    }
    backend
}

fn save(c: &mut Criterion) {
    for i in &[10, 1000, 10_000] {
        c.bench_function(&format!("save a backend with {} changes", i), |b| {
            b.iter_batched(
                || change_backend(*i),
                |b| black_box(b.save().unwrap()),
                criterion::BatchSize::SmallInput,
            )
        });
    }
}

fn load_empty(c: &mut Criterion) {
    c.bench_function("load an empty backend", |b| {
        b.iter_batched(
            || Backend::init().save().unwrap(),
            |v| black_box(Backend::load(v).unwrap()),
            criterion::BatchSize::SmallInput,
        )
    });
}

fn load_small(c: &mut Criterion) {
    c.bench_function("load a small history backend", |b| {
        b.iter_batched(
            || {
                let backend = change_backend(10);
                backend.save().unwrap()
            },
            |v| black_box(Backend::load(v).unwrap()),
            criterion::BatchSize::SmallInput,
        )
    });
}

fn load_medium(c: &mut Criterion) {
    c.bench_function("load a medium history backend", |b| {
        b.iter_batched(
            || {
                let backend = change_backend(1000);
                backend.save().unwrap()
            },
            |v| black_box(Backend::load(v).unwrap()),
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets =  save, load_empty, load_small, load_medium
}
criterion_main!(benches);
