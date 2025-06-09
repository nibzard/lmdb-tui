# TUI Testing Framework

This document describes the comprehensive TUI testing framework implemented for lmdb-tui.

## Overview

The TUI testing framework provides automated testing capabilities for terminal user interfaces, including:

- **Snapshot Testing**: Capture and compare TUI output across test runs
- **User Interaction Simulation**: Automate key presses and navigation
- **Multi-format Output**: Generate text, ANSI, and JSON snapshots
- **AI-Ready Analysis**: Structured reports for AI-assisted code review
- **Responsive Testing**: Test across different terminal sizes
- **Regression Detection**: Compare snapshots to catch UI changes

## Quick Start

### Running Tests

```bash
# Run all TUI tests and generate snapshots
./scripts/run_tui_tests.sh

# Run specific test suites
cargo test test_main_view_navigation --test tui_comprehensive
cargo test test_responsive_layout --test tui_comprehensive
```

### Analyzing Results

```bash
# Generate AI analysis reports
python3 scripts/analyze_snapshots.py

# View generated snapshots
ls test_snapshots/
cat test_snapshots/main_navigation_initial_main_view_000.txt

# View colored output
cat test_snapshots/main_navigation_initial_main_view_000.ansi
```

## Architecture

### TuiTestHarness

The core testing infrastructure provides:

```rust
let mut harness = TuiTestHarness::new("test_name", 80, 24)?;

// Simulate user interactions
harness.send_key(KeyCode::Down)?;
harness.type_string("search query")?;

// Capture snapshots
let snapshot = harness.capture_snapshot("description")?;

// Generate reports
harness.generate_test_report(&snapshots)?;
```

### Test Categories

#### 1. Navigation Tests
- Database switching (up/down arrows)
- View transitions (main ↔ query ↔ help)
- Edge cases (empty databases, rapid navigation)

#### 2. Query Interface Tests
- Query entry and editing
- Result navigation
- Different query types (prefix, range, regex, jsonpath)
- Backspace and clearing

#### 3. Responsive Layout Tests
- Small terminals (40x10)
- Standard terminals (80x24)
- Large terminals (120x40, 200x60)
- Text wrapping and truncation

#### 4. User Workflow Tests
- Complete user journeys
- Multi-step interactions
- Error recovery scenarios

## Snapshot Formats

### Text Format (.txt)
Plain text representation of the terminal output:
```
┌Databases─────────────┐┌Entries──────────────────┐
│test_data             ││user:1: {"name":"Alice"} │
│                      ││user:2: {"name":"Bob"}   │
└──────────────────────┘└─────────────────────────┘
```

### ANSI Format (.ansi)
Terminal output with color codes intact for visual verification:
```
\x1b[0m┌Databases\x1b[43m\x1b[30m─────────────\x1b[0m┐
```

### JSON Format (.json)
Structured data for programmatic analysis:
```json
{
  "timestamp": "2024-06-08T12:42:00Z",
  "test_name": "main_navigation",
  "snapshot_id": "main_navigation_initial_state_000",
  "dimensions": [80, 24],
  "content": "...",
  "ansi_content": "...",
  "app_state": {
    "current_view": "Main",
    "selected_db": "test_data",
    "db_count": 1,
    "entry_count": 4,
    "query": "",
    "query_cursor": 0,
    "show_help": false
  },
  "metadata": {
    "description": "initial_state",
    "counter": "0"
  }
}
```

## AI Analysis Integration

The framework generates structured reports for AI analysis:

```bash
python3 scripts/analyze_snapshots.py
# Creates: ai_analysis_prompt.md
```

### Analysis Areas
1. **UI Consistency**: Layout, spacing, visual hierarchy
2. **User Experience**: Navigation flow, feedback, clarity
3. **Responsive Design**: Adaptation to different screen sizes
4. **Performance**: Render complexity, content efficiency
5. **Accessibility**: Text contrast, information density

### Example AI Prompt
```markdown
# TUI Testing Analysis Report

## Test Session Summary
- Total Snapshots: 15
- Views Tested: Query, Main
- Terminal Sizes: [(40, 10), (80, 24), (120, 40), (200, 60)]

## State Transitions
[detailed transition data]

## Performance Patterns
[render complexity and query response data]

## Analysis Instructions
Please analyze this TUI testing data and provide feedback on:
1. UI Consistency
2. User Experience
3. Visual Design
4. Responsive Design
5. Error Handling
```

## Writing New Tests

### Basic Test Structure

```rust
#[test]
fn test_my_feature() -> anyhow::Result<()> {
    let mut harness = TuiTestHarness::new("my_test", 80, 24)?;
    let mut snapshots = Vec::new();

    // Initial state
    snapshots.push(harness.capture_snapshot("initial_state")?);

    // User interactions
    harness.send_key(KeyCode::Char('/'))?;
    harness.type_string("test query")?;
    snapshots.push(harness.capture_snapshot("query_entered")?);

    // Verification
    harness.send_key(KeyCode::Esc)?;
    snapshots.push(harness.capture_snapshot("final_state")?);

    // Generate report
    harness.generate_test_report(&snapshots)?;

    // Assertions
    assert_eq!(snapshots.last().unwrap().app_state.current_view, "Main");
    
    Ok(())
}
```

### Testing with Real Databases

```rust
#[test]
fn test_with_existing_db() -> anyhow::Result<()> {
    let path = PathBuf::from("path/to/test/database");
    let mut harness = TuiTestHarness::with_database(
        "real_db_test", 
        &path, 
        80, 24
    )?;
    
    // Test with actual data...
    Ok(())
}
```

## Best Practices

### 1. Snapshot Naming
Use descriptive names that indicate:
- Test category
- User action
- Expected state

```rust
harness.capture_snapshot("after_database_navigation")?;
harness.capture_snapshot("query_results_displayed")?;
harness.capture_snapshot("help_overlay_shown")?;
```

### 2. Test Coverage
Ensure tests cover:
- Happy path scenarios
- Edge cases (empty results, long strings)
- Error conditions
- Different terminal sizes

### 3. Performance Considerations
- Use appropriate terminal sizes for tests
- Limit snapshot frequency for performance tests
- Clean up test databases

### 4. Regression Testing
- Commit snapshots to version control for baseline comparison
- Review snapshot changes in PRs
- Use diff tools to compare snapshot changes

## Continuous Integration

### GitHub Actions Integration

```yaml
- name: Run TUI Tests
  run: |
    ./scripts/run_tui_tests.sh
    python3 scripts/analyze_snapshots.py

- name: Upload Test Artifacts
  uses: actions/upload-artifact@v3
  with:
    name: tui-test-snapshots
    path: test_snapshots/
```

### Automated Analysis

```bash
# In CI pipeline
if [ -f ai_analysis_prompt.md ]; then
    # Send to AI analysis service
    curl -X POST "https://api.ai-service.com/analyze" \
         -H "Content-Type: text/markdown" \
         --data-binary @ai_analysis_prompt.md
fi
```

## Troubleshooting

### Common Issues

1. **Terminal Size Mismatches**
   - Ensure consistent terminal dimensions
   - Check for hardcoded assumptions about screen size

2. **Timing Issues**
   - Add delays between rapid key presses if needed
   - Use `harness.render()` to force updates

3. **Database State**
   - Use temporary databases for isolated tests
   - Clean up test data between runs

4. **Snapshot Differences**
   - Check for non-deterministic content (timestamps, etc.)
   - Normalize dynamic content in comparisons

### Debug Tips

```rust
// Force render update
harness.render()?;

// Check current app state
println!("Current view: {:?}", harness.app.current_view());
println!("DB count: {}", harness.app.db_names.len());

// Save debug snapshot
harness.capture_snapshot("debug_state")?;
```

## Future Enhancements

- **Video Recording**: Capture terminal sessions as videos
- **Property-Based Testing**: Generate random interaction sequences
- **Performance Benchmarking**: Measure render times and memory usage
- **Accessibility Testing**: Screen reader compatibility
- **Cross-Platform Testing**: Different terminal emulators and OS