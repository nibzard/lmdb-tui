use criterion::{criterion_group, criterion_main, Criterion};
use heed::types::{Bytes, Str};
use lmdb_tui::db::{
    env::open_env,
    query::{self, Mode},
};
use tempfile::tempdir;

fn bench_scan(c: &mut Criterion) {
    let dir = tempdir().unwrap();
    let env = open_env(dir.path(), false).unwrap();
    let mut tx = env.write_txn().unwrap();
    let db = env
        .create_database::<Str, Bytes>(&mut tx, Some("bench"))
        .unwrap();
    for i in 0..100_000u32 {
        let k = format!("k{:06}", i);
        db.put(&mut tx, &k, b"v").unwrap();
    }
    tx.commit().unwrap();

    c.bench_function("scan", |b| {
        b.iter(|| {
            query::scan(&env, "bench", Mode::Prefix("k"), 100_000).unwrap();
        });
    });
}

criterion_group!(benches, bench_scan);
criterion_main!(benches);
