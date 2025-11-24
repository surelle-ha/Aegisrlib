use aegisrlib::{AegCore, AegFileSystem};
use criterion::{criterion_group, criterion_main, Criterion, black_box};
use std::thread;
use std::time::Duration;

//
// ======================================================
//  Helpers
// ======================================================
fn setup() {
    // Reset config + engine for each benchmark
    AegFileSystem::initialize_config(Some(false), Some(true));
    let mut engine = AegCore::load();

    if engine.collections.is_empty() {
        engine.collections.push("default".into());
    }
    if engine.active_collection.is_empty() {
        engine.active_collection = "default".into();
    }
    engine.save();
}

//
// ======================================================
//  put_value benchmark
// ======================================================
fn bench_put_value(c: &mut Criterion) {
    setup();

    c.bench_function("AegCore::put_value", |b| {
        b.iter(|| {
            AegCore::put_value(black_box("bench_key"), black_box("bench_value"));
        });
    });
}

//
// ======================================================
//  get_value benchmark
// ======================================================
fn bench_get_value(c: &mut Criterion) {
    setup();
    AegCore::put_value("existing_key", "existing_value");

    c.bench_function("AegCore::get_value", |b| {
        b.iter(|| {
            let _ = AegCore::get_value(black_box("existing_key"));
        });
    });
}

//
// ======================================================
//  delete_value benchmark
// ======================================================
fn bench_delete_value(c: &mut Criterion) {
    setup();
    AegCore::put_value("tmp_delete", "remove_me");

    c.bench_function("AegCore::delete_value", |b| {
        b.iter(|| {
            AegCore::delete_value(black_box("tmp_delete"));
            AegCore::put_value("tmp_delete", "remove_me"); // reset for next run
        });
    });
}

//
// ======================================================
//  clear_values benchmark
// ======================================================
fn bench_clear_values(c: &mut Criterion) {
    setup();
    for i in 0..200 {
        AegCore::put_value(format!("key{}", i).as_str(), "value");
    }

    c.bench_function("AegCore::clear_values", |b| {
        b.iter(|| {
            AegCore::clear_values();
            // repopulate for next iteration
            for i in 0..200 {
                AegCore::put_value(format!("key{}", i).as_str(), "value");
            }
        });
    });
}

//
// ======================================================
//  Collection switching benchmark
// ======================================================
fn bench_collection_switch(c: &mut Criterion) {
    setup();
    AegCore::create_collection("bench_col1");
    AegCore::create_collection("bench_col2");
    let mut engine = AegCore::load();

    c.bench_function("AegCore::set_active_collection", |b| {
        b.iter(|| {
            engine.set_active_collection(black_box("bench_col1")).unwrap();
            engine.set_active_collection(black_box("bench_col2")).unwrap();
        });
    });
}

//
// ======================================================
//  Full round-trip read/write cycle benchmark
// ======================================================
fn bench_full_roundtrip(c: &mut Criterion) {
    setup();

    c.bench_function("AegCore full roundtrip (put → get → delete)", |b| {
        b.iter(|| {
            AegCore::put_value("cycle_key", "cycle_value");
            let _ = AegCore::get_value("cycle_key");
            AegCore::delete_value("cycle_key");
        });
    });
}

//
// ======================================================
//  Multi-collection stress test
// ======================================================
fn bench_multi_collection_stress(c: &mut Criterion) {
    setup();

    // Create multiple collections
    for i in 0..20 {
        AegCore::create_collection(format!("col{}", i).as_str());
    }

    let mut engine = AegCore::load();

    c.bench_function("multi-collection stress (switch → write → read)", |b| {
        b.iter(|| {
            for i in 0..20 {
                let col = format!("col{}", i);
                engine.set_active_collection(col.as_str()).unwrap();

                let key = format!("k{}", i);
                let val = format!("v{}", i);

                AegCore::put_value(&key, &val);
                let _ = AegCore::get_value(&key);
            }
        });
    });
}

//
// ======================================================
//  Background-saver concurrency impact benchmark
// ======================================================
fn bench_background_saver_concurrency(c: &mut Criterion) {
    setup();

    // Start background saver
    AegCore::start_background_saver(1);

    c.bench_function("concurrent write under background saver", |b| {
        b.iter(|| {
            AegCore::put_value("concurrent_key", "concurrent_value");
        });
    });

    // Stop saver
    AegCore::stop_background_saver();
}

//
// ======================================================
//  Criterion group + main
// ======================================================
criterion_group!(
    aegis_benches,
    bench_put_value,
    bench_get_value,
    bench_delete_value,
    bench_clear_values,
    bench_collection_switch,
    bench_full_roundtrip,
    bench_multi_collection_stress,
    bench_background_saver_concurrency,
);

criterion_main!(aegis_benches);
