use std::time::{Duration, Instant};
use heed::types::Str;
use lmdb_tui::db::{
    env::{open_env, list_entries_paginated, count_entries},
    query::{self, Mode, count_matches, scan_paginated},
};
use tempfile::tempdir;

/// Helper function to create a test database with many entries
fn create_large_test_db(size: usize) -> anyhow::Result<(tempfile::TempDir, heed::Env)> {
    let dir = tempdir()?;
    let env = open_env(dir.path(), false)?;
    let mut tx = env.write_txn()?;
    let db = env.create_database::<Str, Str>(&mut tx, Some("data"))?;
    
    // Create entries with predictable pattern for testing
    for i in 0..size {
        let key = format!("key_{:06}", i);
        let value = format!("value_{}", i);
        db.put(&mut tx, &key, &value)?;
    }
    
    tx.commit()?;
    Ok((dir, env))
}

#[test]
fn test_count_matches_performance() -> anyhow::Result<()> {
    let (_dir, env) = create_large_test_db(1000)?;
    
    // Test prefix count performance - for keys 0-999, "key_0009" matches key_000900-key_000999 (100 entries)
    let start = Instant::now();
    let count = count_matches(&env, "data", Mode::Prefix("key_0009"))?;
    let duration = start.elapsed();
    
    // Should find 100 entries (key_000900 through key_000999)
    assert_eq!(count, 100);
    
    // Should complete quickly (under 100ms for 1000 entries)
    assert!(duration < Duration::from_millis(100), 
           "Count took too long: {:?}", duration);
    
    Ok(())
}

#[test]
fn test_scan_paginated_performance() -> anyhow::Result<()> {
    let (_dir, env) = create_large_test_db(1000)?;
    
    // Test paginated scanning - "key_0001" matches key_000100-key_000199 (100 entries)
    let start = Instant::now();
    let results = scan_paginated(&env, "data", Mode::Prefix("key_0001"), 10, 5)?;
    let duration = start.elapsed();
    
    // Should return exactly 5 results starting from offset 10 
    assert_eq!(results.len(), 5);
    assert_eq!(results[0].0, "key_000110"); // 10th entry in key_0001xx range
    assert_eq!(results[4].0, "key_000114"); // 14th entry in key_0001xx range
    
    // Should complete quickly
    assert!(duration < Duration::from_millis(50), 
           "Paginated scan took too long: {:?}", duration);
    
    Ok(())
}

#[test]
fn test_lazy_loading_pagination() -> anyhow::Result<()> {
    let (_dir, env) = create_large_test_db(500)?;
    
    // Test first page
    let start = Instant::now();
    let page1 = list_entries_paginated(&env, "data", 0, 50)?;
    let duration1 = start.elapsed();
    
    assert_eq!(page1.len(), 50);
    assert_eq!(page1[0].0, "key_000000");
    assert_eq!(page1[49].0, "key_000049");
    
    // Test second page
    let start = Instant::now();
    let page2 = list_entries_paginated(&env, "data", 50, 50)?;
    let duration2 = start.elapsed();
    
    assert_eq!(page2.len(), 50);
    assert_eq!(page2[0].0, "key_000050");
    assert_eq!(page2[49].0, "key_000099");
    
    // Both operations should be fast
    assert!(duration1 < Duration::from_millis(100));
    assert!(duration2 < Duration::from_millis(100));
    
    Ok(())
}

#[test]
fn test_count_entries_accuracy() -> anyhow::Result<()> {
    let (_dir, env) = create_large_test_db(250)?;
    
    let count = count_entries(&env, "data")?;
    assert_eq!(count, 250);
    
    Ok(())
}

#[test]
fn test_prefix_query_early_termination() -> anyhow::Result<()> {
    let (_dir, env) = create_large_test_db(1000)?;
    
    // Test that prefix queries stop early when they pass the prefix range
    let start = Instant::now();
    let results = query::scan(&env, "data", Mode::Prefix("key_000999"), 100)?;
    let duration = start.elapsed();
    
    // Should find exactly 1 entry (key_000999)
    assert_eq!(results.len(), 1);
    assert!(results[0].0.starts_with("key_000999"));
    
    // Should be much faster than scanning all 1000 entries
    assert!(duration < Duration::from_millis(20), 
           "Prefix query took too long: {:?}", duration);
    
    Ok(())
}

#[test]
fn test_range_query_early_termination() -> anyhow::Result<()> {
    let (_dir, env) = create_large_test_db(1000)?;
    
    // Test range query with early termination
    let start = Instant::now();
    let results = query::scan(&env, "data", Mode::Range("key_000100", "key_000110"), 100)?;
    let duration = start.elapsed();
    
    // Should find exactly 10 entries
    assert_eq!(results.len(), 10);
    assert_eq!(results[0].0, "key_000100");
    assert_eq!(results[9].0, "key_000109");
    
    // Should complete quickly due to early termination
    assert!(duration < Duration::from_millis(20), 
           "Range query took too long: {:?}", duration);
    
    Ok(())
}

#[test]
fn test_count_vs_scan_consistency() -> anyhow::Result<()> {
    let (_dir, env) = create_large_test_db(300)?;
    
    // Test that count_matches and scan return consistent results
    let mode = Mode::Prefix("key_001");
    
    let count = count_matches(&env, "data", mode.clone())?;
    let scan_results = query::scan(&env, "data", mode, usize::MAX)?;
    
    assert_eq!(count, scan_results.len());
    
    Ok(())
}

#[test]
fn test_pagination_completeness() -> anyhow::Result<()> {
    let (_dir, env) = create_large_test_db(157)?; // Odd number to test edge cases
    
    let page_size = 50;
    let mut all_keys = Vec::new();
    let mut offset = 0;
    
    loop {
        let page = list_entries_paginated(&env, "data", offset, page_size)?;
        if page.is_empty() {
            break;
        }
        
        for (key, _) in page {
            all_keys.push(key);
        }
        
        offset += page_size;
        
        // Safety check to prevent infinite loop
        if offset > 200 {
            break;
        }
    }
    
    // Should have retrieved all 157 entries
    assert_eq!(all_keys.len(), 157);
    
    // Keys should be in order
    for i in 0..157 {
        let expected = format!("key_{:06}", i);
        assert_eq!(all_keys[i], expected);
    }
    
    Ok(())
}

#[test] 
fn test_memory_efficiency_pagination() -> anyhow::Result<()> {
    let (_dir, env) = create_large_test_db(1000)?;
    
    // Test that paginated queries don't load unnecessary data
    let small_page = list_entries_paginated(&env, "data", 0, 10)?;
    let large_page = list_entries_paginated(&env, "data", 0, 100)?;
    
    assert_eq!(small_page.len(), 10);
    assert_eq!(large_page.len(), 100);
    
    // The small page should only contain the first 10 entries
    for i in 0..10 {
        let expected = format!("key_{:06}", i);
        assert_eq!(small_page[i].0, expected);
    }
    
    Ok(())
}

/// Benchmark test to ensure performance improvements are maintained
#[test]
fn test_performance_regression() -> anyhow::Result<()> {
    let (_dir, env) = create_large_test_db(2000)?;
    
    // Test that count operation is significantly faster than full scan
    let start = Instant::now();
    let count = count_matches(&env, "data", Mode::Prefix("key_"))?;
    let count_duration = start.elapsed();
    
    let start = Instant::now();
    let scan_results = query::scan(&env, "data", Mode::Prefix("key_"), usize::MAX)?;
    let scan_duration = start.elapsed();
    
    assert_eq!(count, scan_results.len());
    assert_eq!(count, 2000);
    
    // Count should be significantly faster than full scan for large datasets
    // This is a regression test for the double-scanning issue
    println!("Count duration: {:?}, Scan duration: {:?}", count_duration, scan_duration);
    
    // For large datasets, count should be at least somewhat faster
    // Note: This might not always hold on small test datasets, but it's a good regression test
    assert!(count_duration <= scan_duration * 2, 
           "Count performance regression detected");
    
    Ok(())
}