# Test Suite Documentation

This directory contains comprehensive tests for the lmdb-tui application, covering functionality, performance, integration, and user interface testing.

## Test Structure

```
tests/
├── README.md                    # This documentation file
├── performance.rs              # Performance and optimization tests
├── unit.rs                     # Unit tests entry point
├── integration.rs              # Integration tests entry point
├── tui_comprehensive.rs        # Comprehensive TUI behavior tests
├── tui_harness.rs             # TUI test harness and utilities
├── crud_integration_test.rs    # CRUD operations integration tests
├── export_cli.rs              # CLI export functionality tests
├── parse_query.rs             # Query parsing tests
├── remote.rs                  # Remote/gRPC functionality tests
├── unit/                      # Unit test modules
│   ├── app.rs                 # Application state and logic tests
│   ├── commands.rs            # Command execution tests
│   ├── crud.rs                # CRUD operation tests
│   ├── env.rs                 # Database environment tests
│   ├── errors.rs              # Error handling tests
│   ├── export.rs              # Export functionality tests
│   └── query.rs               # Query engine tests
├── integration/               # Integration test modules
│   ├── cli.rs                 # CLI interface tests
│   └── pipeline.rs            # Data processing pipeline tests
└── ui/                        # User interface tests
    ├── help_ui.rs             # Help system UI tests
    └── query_ui.rs            # Query interface UI tests
```

## Test Categories

### 1. Performance Tests (`performance.rs`)

**Purpose**: Validate performance optimizations and prevent regression.

#### Key Tests:
- **`test_count_matches_performance`** - Verifies optimized counting operations complete quickly
- **`test_scan_paginated_performance`** - Tests paginated scanning with timing constraints
- **`test_lazy_loading_pagination`** - Validates efficient pagination across multiple pages
- **`test_count_entries_accuracy`** - Ensures counting functions return accurate results
- **`test_prefix_query_early_termination`** - Verifies prefix queries stop early for performance
- **`test_range_query_early_termination`** - Tests range query optimization
- **`test_count_vs_scan_consistency`** - Ensures count and scan operations return consistent results
- **`test_pagination_completeness`** - Verifies pagination doesn't miss or duplicate entries
- **`test_memory_efficiency_pagination`** - Tests memory usage with different page sizes
- **`test_performance_regression`** - Benchmark test to catch performance regressions

**Coverage**: Query optimization, lazy loading, memory efficiency, early termination algorithms.

### 2. Unit Tests (`unit/`)

#### Application Tests (`unit/app.rs`)
- **State Management**: Tests for Redux-style state updates and view transitions
- **Spinner Animation**: Loading indicator functionality and state management
- **Lazy Loading**: Pagination and lazy loading configuration
- **File Event Debouncing**: File system change detection and debouncing logic
- **Navigation**: Database and entry navigation actions

#### Query Engine Tests (`unit/query.rs`)
- **Prefix Queries**: String prefix matching with early termination
- **Range Queries**: Key range filtering with boundary conditions
- **Regex Queries**: Regular expression pattern matching
- **JSONPath Queries**: JSON value filtering with JSONPath expressions
- **Value Decoding**: JSON, MessagePack, and plugin-based value decoding
- **Query Parsing**: User input parsing into query modes

#### CRUD Tests (`unit/crud.rs`)
- **Create Operations**: Entry creation with validation
- **Read Operations**: Entry retrieval and pagination
- **Update Operations**: Entry modification and persistence
- **Delete Operations**: Entry removal and cleanup

#### Environment Tests (`unit/env.rs`)
- **Database Lifecycle**: Opening, closing, and managing database environments
- **Transaction Management**: Read/write transaction handling
- **Error Handling**: Database-level error scenarios

#### Command Tests (`unit/commands.rs`)
- **Command Execution**: Individual command processing
- **Undo/Redo**: Operation history and reversal
- **Validation**: Input validation and error handling

#### Export Tests (`unit/export.rs`)
- **Format Support**: JSON, CSV, YAML export formats
- **Data Integrity**: Export accuracy and completeness
- **Error Handling**: Export failure scenarios

#### Error Tests (`unit/errors.rs`)
- **Error Types**: Custom error type validation
- **Error Propagation**: Error handling through application layers
- **User Messaging**: Error message formatting and context

### 3. Integration Tests (`integration/`)

#### CLI Tests (`integration/cli.rs`)
- **Command Line Interface**: Argument parsing and command execution
- **Output Formatting**: Plain text and JSON output modes
- **Exit Codes**: Proper exit code handling for different scenarios

#### Pipeline Tests (`integration/pipeline.rs`)
- **Data Flow**: End-to-end data processing pipelines
- **Component Integration**: Module interaction testing
- **Resource Management**: Memory and file handle management

### 4. TUI Tests

#### Comprehensive TUI Tests (`tui_comprehensive.rs`)
- **User Workflows**: Complete user interaction scenarios
- **Keyboard Navigation**: Key binding and navigation testing
- **State Transitions**: View changes and state management
- **Responsive Design**: Layout adaptation to different terminal sizes

#### TUI Harness (`tui_harness.rs`)
- **Test Utilities**: Helper functions for TUI testing
- **Mock Terminal**: Simulated terminal environment for testing
- **Event Simulation**: Keyboard and mouse event generation

#### UI Component Tests (`ui/`)
- **Help UI** (`help_ui.rs`): Help system rendering and interaction
- **Query UI** (`query_ui.rs`): Query interface layout and functionality

### 5. Specialized Tests

#### CRUD Integration (`crud_integration_test.rs`)
- **End-to-End CRUD**: Complete create/read/update/delete workflows
- **Data Persistence**: Verification of data durability
- **Concurrent Operations**: Multi-user scenario testing

#### Export CLI (`export_cli.rs`)
- **CLI Export Commands**: Command-line export functionality
- **Format Validation**: Output format correctness
- **Large Dataset Handling**: Performance with large exports

#### Query Parsing (`parse_query.rs`)
- **Query Syntax**: Parser validation for all query types
- **Error Handling**: Invalid query syntax handling
- **Edge Cases**: Boundary condition testing

#### Remote Functionality (`remote.rs`)
- **gRPC Interface**: Remote API testing
- **Network Protocols**: Communication protocol validation
- **Service Integration**: Remote service interaction

## Running Tests

### Run All Tests
```bash
cargo test
```

### Run Specific Test Categories
```bash
# Performance tests only
cargo test --test performance

# Unit tests only
cargo test --test unit

# Integration tests only
cargo test --test integration

# TUI tests only
cargo test tui_comprehensive
```

### Run Individual Test Functions
```bash
# Specific performance test
cargo test test_count_matches_performance

# Specific unit test
cargo test app::test_spinner_animation

# Verbose output
cargo test --verbose
```

### Test with Coverage
```bash
# Install cargo-tarpaulin for coverage
cargo install cargo-tarpaulin

# Run with coverage
cargo tarpaulin --out Html
```

## Test Data and Fixtures

### Database Test Data
- **Small Datasets**: 10-100 entries for unit tests
- **Medium Datasets**: 250-500 entries for integration tests
- **Large Datasets**: 1000+ entries for performance tests
- **Edge Cases**: Empty databases, single entries, special characters

### Test Patterns
- **Deterministic Keys**: `key_{:06}` format (e.g., `key_000000`, `key_000123`)
- **Predictable Values**: Simple string or JSON values for validation
- **Temporary Databases**: Using `tempfile` crate for isolated test environments

## Performance Benchmarks

### Timing Constraints
- **Count Operations**: < 100ms for 1000 entries
- **Paginated Scans**: < 50ms for page retrieval
- **Range Queries**: < 20ms for small ranges
- **Prefix Queries**: < 20ms with early termination

### Memory Constraints
- **Pagination**: Proper capacity pre-allocation
- **Large Queries**: Memory usage limits and overflow protection
- **Streaming**: Lazy evaluation without full materialization

## Continuous Integration

### Pre-commit Checks
```bash
# Format code
cargo fmt --all

# Lint code
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test

# Performance regression check
cargo test --test performance
```

### Test Reliability
- **Deterministic Results**: Tests produce consistent results across runs
- **Isolated Environments**: Each test uses temporary resources
- **Cleanup**: Proper resource cleanup prevents test interference
- **Timeout Protection**: Tests have reasonable time limits

## Debugging Tests

### Verbose Output
```bash
# Show test output
cargo test -- --nocapture

# Show timing information
cargo test --verbose

# Run single test with output
cargo test test_name -- --exact --nocapture
```

### Test Debugging Tools
- **`println!` Debugging**: Use `println!` for debug output (visible with `--nocapture`)
- **`env_logger`**: Set `RUST_LOG=debug` for detailed logging
- **`tempfile` Inspection**: Temporary test databases can be manually inspected
- **Panic Backtraces**: Set `RUST_BACKTRACE=1` for detailed error traces

## Adding New Tests

### Test Organization
1. **Unit Tests**: Add to appropriate module in `unit/`
2. **Integration Tests**: Add to `integration/` or create new top-level test file
3. **Performance Tests**: Add to `performance.rs` with timing constraints
4. **UI Tests**: Add to `ui/` for user interface components

### Test Patterns
```rust
#[test]
fn test_feature_name() -> anyhow::Result<()> {
    // Setup: Create test environment
    let dir = tempdir()?;
    let env = open_env(dir.path(), false)?;
    
    // Test: Execute the functionality
    let result = function_under_test(&env, "test_input")?;
    
    // Verify: Assert expected behavior
    assert_eq!(result.len(), expected_count);
    assert!(result.contains(&expected_value));
    
    Ok(())
}
```

### Performance Test Patterns
```rust
#[test]
fn test_performance_feature() -> anyhow::Result<()> {
    let (_dir, env) = create_large_test_db(1000)?;
    
    let start = Instant::now();
    let result = optimized_function(&env)?;
    let duration = start.elapsed();
    
    // Verify correctness
    assert_eq!(result.len(), expected_count);
    
    // Verify performance
    assert!(duration < Duration::from_millis(100), 
           "Operation took too long: {:?}", duration);
    
    Ok(())
}
```

## Test Maintenance

### Regular Tasks
- **Update Test Data**: Refresh test datasets when application behavior changes
- **Performance Baselines**: Review and update timing constraints periodically
- **Cleanup Dead Code**: Remove tests for deprecated functionality
- **Documentation**: Keep test documentation synchronized with implementation

### Best Practices
- **Descriptive Names**: Test function names should clearly describe what is being tested
- **Single Responsibility**: Each test should focus on one specific behavior
- **Isolation**: Tests should not depend on each other or external state
- **Readability**: Tests should be easy to understand and maintain
- **Coverage**: Aim for comprehensive coverage of edge cases and error conditions

---

*This test suite ensures the reliability, performance, and correctness of lmdb-tui across all supported platforms and usage scenarios.*