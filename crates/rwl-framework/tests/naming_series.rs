use proptest::prelude::*;
use rwl_framework::naming::{NamingEngine, NamingSeriesStore};
use std::collections::HashSet;
use std::sync::Arc;
use std::thread;

#[test]
fn test_naming_series_monotonicity() {
    let store = NamingSeriesStore::new();
    let prefix = "ACC-SINV-2026-";

    let mut last = 0;
    for _ in 0..100 {
        let next = store.get_next(prefix);
        assert_eq!(next, last + 1, "Counter must strictly increment by 1");
        last = next;
    }
}

#[test]
fn test_prefix_isolation() {
    let store = NamingSeriesStore::new();
    let prefix_a = "SALES-INV-";
    let prefix_b = "PURCH-ORD-";

    store.get_next(prefix_a);
    store.get_next(prefix_a);
    assert_eq!(store.get_current(prefix_a), 2);
    assert_eq!(store.get_current(prefix_b), 0);

    store.get_next(prefix_b);
    assert_eq!(store.get_current(prefix_b), 1);
    assert_eq!(store.get_current(prefix_a), 2);
}

#[test]
fn test_concurrent_naming_uniqueness() {
    let store = Arc::new(NamingSeriesStore::new());
    let prefix = "CONCURRENT-SERIES-";
    let num_threads = 10;
    let iterations_per_thread = 50;

    let handles: Vec<_> = (0..num_threads)
        .map(|_| {
            let store_clone = Arc::clone(&store);
            thread::spawn(move || {
                let mut generated = Vec::with_capacity(iterations_per_thread);
                for _ in 0..iterations_per_thread {
                    let num = store_clone.get_next(prefix);
                    let formatted = NamingEngine::parse_series_template(
                        "CONCURRENT-SERIES-.#####",
                        2026,
                        11,
                        1,
                        &store_clone,
                    );
                    generated.push((num, formatted));
                }
                generated
            })
        })
        .collect();

    let mut all_numbers = HashSet::new();
    let mut all_formatted = HashSet::new();

    for handle in handles {
        let results = handle.join().expect("Worker thread failed");
        for (num, formatted) in results {
            assert!(
                all_numbers.insert(num),
                "Duplicate counter value detected: {num}"
            );
            assert!(
                all_formatted.insert(formatted.clone()),
                "Duplicate formatted name detected: {formatted}"
            );
        }
    }

    assert_eq!(
        all_numbers.len(),
        num_threads * iterations_per_thread,
        "All generated counters must be globally unique"
    );
    assert_eq!(
        all_formatted.len(),
        num_threads * iterations_per_thread,
        "All generated formatted names must be globally unique"
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn prop_series_template_formatting(
        prefix in "[A-Z]{2,6}",
        hashes in 1..8usize,
        year in 2020..2050i32,
        month in 1..12u32,
        day in 1..28u32
    ) {
        let store = NamingSeriesStore::new();
        let hash_str = "#".repeat(hashes);
        let template = format!("{prefix}-.YYYY.-.{hash_str}");

        let name1 = NamingEngine::parse_series_template(&template, year, month, day, &store);
        let name2 = NamingEngine::parse_series_template(&template, year, month, day, &store);

        let expected_prefix = format!("{prefix}-{year}-");
        prop_assert!(name1.starts_with(&expected_prefix));
        prop_assert!(name2.starts_with(&expected_prefix));
        prop_assert_ne!(name1, name2, "Consecutive calls must produce distinct names");
    }
}
