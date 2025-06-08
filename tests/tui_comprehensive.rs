use std::path::PathBuf;

use crossterm::event::KeyCode;

mod tui_harness;
use tui_harness::{TuiTestHarness, TuiSnapshot};

/// Test main view rendering and basic navigation
#[test]
fn test_main_view_navigation() -> anyhow::Result<()> {
    let mut harness = TuiTestHarness::new("main_navigation", 80, 24)?;
    let mut snapshots = Vec::new();

    // Capture initial state
    snapshots.push(harness.capture_snapshot("initial_main_view")?);

    // Test database navigation if multiple databases exist
    if harness.app.db_names.len() > 1 {
        // Navigate down
        harness.send_key(KeyCode::Down)?;
        snapshots.push(harness.capture_snapshot("after_down_navigation")?);

        // Navigate up
        harness.send_key(KeyCode::Up)?;
        snapshots.push(harness.capture_snapshot("after_up_navigation")?);
    }

    // Test help toggle
    harness.send_key(KeyCode::Char('?'))?;
    snapshots.push(harness.capture_snapshot("help_displayed")?);

    // Close help
    harness.send_key(KeyCode::Esc)?;
    snapshots.push(harness.capture_snapshot("help_closed")?);

    // Generate test report
    harness.generate_test_report(&snapshots)?;

    // Basic assertions
    assert!(!snapshots.is_empty());
    assert_eq!(snapshots[0].app_state.current_view, "Main");
    
    Ok(())
}

/// Test query view functionality
#[test]
fn test_query_view_functionality() -> anyhow::Result<()> {
    let mut harness = TuiTestHarness::new("query_functionality", 80, 24)?;
    let mut snapshots = Vec::new();

    // Start in main view
    snapshots.push(harness.capture_snapshot("main_before_query")?);

    // Enter query mode
    harness.send_key(KeyCode::Char('/'))?;
    snapshots.push(harness.capture_snapshot("query_mode_entered")?);

    // Type a query
    harness.type_string("user")?;
    snapshots.push(harness.capture_snapshot("query_typed_user")?);

    // Test query navigation
    if harness.app.entries.len() > 1 {
        harness.send_key(KeyCode::Down)?;
        snapshots.push(harness.capture_snapshot("query_navigation_down")?);

        harness.send_key(KeyCode::Up)?;
        snapshots.push(harness.capture_snapshot("query_navigation_up")?);
    }

    // Test backspace
    harness.send_key(KeyCode::Backspace)?;
    snapshots.push(harness.capture_snapshot("query_after_backspace")?);

    // Exit query mode
    harness.send_key(KeyCode::Esc)?;
    snapshots.push(harness.capture_snapshot("query_mode_exited")?);

    // Generate test report
    harness.generate_test_report(&snapshots)?;

    // Assertions
    assert!(snapshots.len() >= 5);
    assert_eq!(snapshots.last().unwrap().app_state.current_view, "Main");
    
    Ok(())
}

/// Test different query types and patterns
#[test]
fn test_query_patterns() -> anyhow::Result<()> {
    let mut harness = TuiTestHarness::new("query_patterns", 80, 24)?;
    let mut snapshots = Vec::new();

    let test_queries = vec![
        ("prefix user", "prefix_query"),
        ("range a z", "range_query"),
        ("regex .*user.*", "regex_query"),
        ("jsonpath $.name", "jsonpath_query"),
    ];

    for (query, description) in test_queries {
        // Enter query mode
        harness.send_key(KeyCode::Char('/'))?;
        
        // Clear any existing query
        while !harness.app.query.is_empty() {
            harness.send_key(KeyCode::Backspace)?;
        }

        // Type the query
        harness.type_string(query)?;
        snapshots.push(harness.capture_snapshot(&format!("{}_entered", description))?);

        // Exit query mode
        harness.send_key(KeyCode::Esc)?;
        snapshots.push(harness.capture_snapshot(&format!("{}_results", description))?);
    }

    // Generate test report
    harness.generate_test_report(&snapshots)?;

    Ok(())
}

/// Test edge cases and error conditions
#[test]
fn test_edge_cases() -> anyhow::Result<()> {
    let mut harness = TuiTestHarness::new("edge_cases", 80, 24)?;
    let mut snapshots = Vec::new();

    // Test empty query
    harness.send_key(KeyCode::Char('/'))?;
    snapshots.push(harness.capture_snapshot("empty_query_mode")?);
    harness.send_key(KeyCode::Esc)?;

    // Test rapid navigation
    for _ in 0..5 {
        harness.send_key(KeyCode::Down)?;
    }
    snapshots.push(harness.capture_snapshot("rapid_navigation_down")?);

    for _ in 0..10 {
        harness.send_key(KeyCode::Up)?;
    }
    snapshots.push(harness.capture_snapshot("rapid_navigation_up")?);

    // Test long query string
    harness.send_key(KeyCode::Char('/'))?;
    harness.type_string("this_is_a_very_long_query_string_that_might_cause_issues_with_display")?;
    snapshots.push(harness.capture_snapshot("long_query_string")?);
    harness.send_key(KeyCode::Esc)?;

    // Generate test report
    harness.generate_test_report(&snapshots)?;

    Ok(())
}

/// Test with real database files
#[test]
fn test_with_real_databases() -> anyhow::Result<()> {
    let test_databases = vec![
        "experiments/test_data/unnamed_db",
        "experiments/test_data/mixed_db",
        "experiments/test_data/empty_db",
    ];

    for db_path in test_databases {
        let path = PathBuf::from(db_path);
        if !path.exists() {
            continue; // Skip if test database doesn't exist
        }

        let test_name = format!("real_db_{}", path.file_name().unwrap().to_string_lossy());
        let mut harness = TuiTestHarness::with_database(&test_name, &path, 80, 24)?;
        let mut snapshots = Vec::new();

        // Capture initial state
        snapshots.push(harness.capture_snapshot("initial_state")?);

        // Test basic navigation
        harness.send_key(KeyCode::Down)?;
        snapshots.push(harness.capture_snapshot("after_navigation")?);

        // Test query mode
        harness.send_key(KeyCode::Char('/'))?;
        harness.type_string("test")?;
        snapshots.push(harness.capture_snapshot("query_entered")?);
        harness.send_key(KeyCode::Esc)?;

        // Generate test report
        harness.generate_test_report(&snapshots)?;
    }

    Ok(())
}

/// Test UI responsiveness with different terminal sizes
#[test]
fn test_responsive_layout() -> anyhow::Result<()> {
    let terminal_sizes = vec![
        (40, 10, "small"),
        (80, 24, "medium"),
        (120, 40, "large"),
        (200, 60, "extra_large"),
    ];

    for (width, height, size_name) in terminal_sizes {
        let test_name = format!("responsive_{}", size_name);
        let mut harness = TuiTestHarness::new(&test_name, width, height)?;
        let mut snapshots = Vec::new();

        // Test main view
        snapshots.push(harness.capture_snapshot("main_view")?);

        // Test query view
        harness.send_key(KeyCode::Char('/'))?;
        harness.type_string("user:1")?;
        snapshots.push(harness.capture_snapshot("query_view")?);
        harness.send_key(KeyCode::Esc)?;

        // Test help view
        harness.send_key(KeyCode::Char('?'))?;
        snapshots.push(harness.capture_snapshot("help_view")?);
        harness.send_key(KeyCode::Esc)?;

        // Generate test report
        harness.generate_test_report(&snapshots)?;

        // Basic assertions for layout
        assert_eq!(snapshots[0].dimensions, (width, height));
    }

    Ok(())
}

/// Test complete user workflows
#[test]
fn test_user_workflows() -> anyhow::Result<()> {
    let mut harness = TuiTestHarness::new("user_workflows", 80, 24)?;
    let mut snapshots = Vec::new();

    // Workflow 1: Browse databases and entries
    snapshots.push(harness.capture_snapshot("workflow_start")?);
    
    // Navigate through databases
    harness.send_key(KeyCode::Down)?;
    snapshots.push(harness.capture_snapshot("database_selected")?);

    // Workflow 2: Search for specific data
    harness.send_key(KeyCode::Char('/'))?;
    snapshots.push(harness.capture_snapshot("search_initiated")?);
    
    harness.type_string("user")?;
    snapshots.push(harness.capture_snapshot("search_results")?);

    // Navigate through search results
    harness.send_key(KeyCode::Down)?;
    snapshots.push(harness.capture_snapshot("result_navigation")?);

    // Workflow 3: Get help and return
    harness.send_key(KeyCode::Esc)?;
    harness.send_key(KeyCode::Char('?'))?;
    snapshots.push(harness.capture_snapshot("help_accessed")?);

    harness.send_key(KeyCode::Esc)?;
    snapshots.push(harness.capture_snapshot("workflow_complete")?);

    // Generate comprehensive test report
    harness.generate_test_report(&snapshots)?;

    // Verify workflow completion
    assert_eq!(snapshots.last().unwrap().app_state.current_view, "Main");

    Ok(())
}