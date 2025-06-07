# lmdb-tui Test Suite

This directory contains a comprehensive automated test suite for lmdb-tui with multiple test levels and comprehensive coverage.

## 📁 Test Files Overview

### Current Test Scripts
- **`run_all_tests.sh`** - Master test runner with prerequisite checks
- **`test_lmdb_tui.sh`** - Comprehensive test suite (9 test categories)
- **`fast_test.sh`** - Quick comprehensive test (9 tests, <2 seconds)
- **`quick_test.sh`** - Simple demonstration test
- **`create_test_databases.py`** - Creates various LMDB test databases

### Test Data & Results
- **`test_data/`** - Generated test databases (5 different types)
- **`test_results/`** - Test outputs, logs, and reports
- **`amp-yaml/conversations.lmdb`** - Real-world database for testing

## 🚀 Quick Start

```bash
# From the lmdb-tui root directory:
cd experiments
./run_all_tests.sh
```

## 📋 Prerequisites

1. **Build the release binary**:
   ```bash
   cargo build --release
   ```

2. **Python 3** with `lmdb` module:
   ```bash
   pip3 install lmdb
   ```

## 🧪 Test Categories (9 Comprehensive Categories)

### 1. **Basic Functionality**
- Help command (`--help`) - validates help text content
- Version command (`--version`) - confirms version output format
- Basic argument parsing and validation

### 2. **Error Handling** 
- Missing arguments (shows help message)
- Non-existent databases (exit code 2)
- Permission errors and file access issues
- Proper exit codes for scripting integration

### 3. **Database Detection & Support**
- **Unnamed databases** (like conversations.lmdb) - automatic detection
- **Named databases** - multiple named DB support 
- **Empty databases** - graceful handling of empty environments
- **Mixed databases** - unnamed + named DB combinations
- **Large databases** - performance with 1000+ entries

### 4. **Output Formats & Modes**
- **Plain text mode** (`--plain`) - human-readable output
- **JSON mode** (`--json`) - structured data for processing
- **Quiet mode** (`--quiet`) - minimal output
- **Verbose mode** (`--verbose`) - detailed information

### 5. **CLI Pipeline Integration**
- **Unix tool compatibility** - works with `grep`, `wc`, `jq`
- **Pipeline support** - proper stdin/stdout handling
- **Exit codes** - 0 (success), 1 (error), 2 (not found)
- **Redirection support** - file output and piping

### 6. **Performance & Benchmarking**
- **Execution time** - sub-100ms targets for typical operations
- **Large database handling** - 1000+ entry performance tests
- **Consistency testing** - deterministic output verification
- **Memory efficiency** - indirect monitoring through execution

## 📊 Test Results

After running, check:
- **`test_results/report.md`** - Human-readable test report
- **`test_results/*.txt`** - Individual test outputs
- **`test_results/*.log`** - Detailed test logs

## 🔧 Running Individual Tests

### Quick Development Testing
```bash
# Fast test suite (9 tests, ~2 seconds)
./fast_test.sh

# Simple demonstration test  
./quick_test.sh

# Full comprehensive test suite
./test_lmdb_tui.sh
```

### Manual Database Testing
```bash
# Create test databases first
python3 create_test_databases.py

# Test specific database types
../target/release/lmdb-tui --read-only --plain test_data/unnamed_db
../target/release/lmdb-tui --read-only --json test_data/named_dbs  
../target/release/lmdb-tui --read-only --plain test_data/large_db

# Test real-world database
../target/release/lmdb-tui --read-only --plain amp-yaml/conversations.lmdb
```

### Debug Specific Issues
```bash
# Test error conditions
../target/release/lmdb-tui --read-only --plain nonexistent  # Should exit 2
../target/release/lmdb-tui  # Should show help

# Test output formats
../target/release/lmdb-tui --read-only --json amp-yaml/conversations.lmdb | jq
```

## 📝 Adding New Tests

1. Add test function to `test_lmdb_tui.sh`:
   ```bash
   test_my_new_feature() {
       log_test "My New Feature Test"
       # Your test code here
       log_success "Test passed"
   }
   ```

2. Call it from the main function:
   ```bash
   main() {
       # ... existing tests ...
       test_my_new_feature
   }
   ```

## 🐛 Debugging Failed Tests

1. Check the log file in `test_results/`
2. Look at specific test output files
3. Run the failing command manually
4. Use `--verbose` flag for more info

## 🎯 Expected Results

All tests should pass with:
- ✅ 100% pass rate
- ✅ Exit code 0
- ✅ No crashes or panics
- ✅ Correct output formats

## 🔄 Continuous Integration

These tests can be integrated into CI/CD:

```yaml
# Example GitHub Actions
- name: Build
  run: cargo build --release
  
- name: Run Tests
  run: |
    cd experiments
    ./run_all_tests.sh
```

## 📈 Performance Benchmarks

Current benchmarks (example):
- Simple database list: < 50ms
- JSON output: < 100ms
- Large database (1000 entries): < 200ms

## 🔄 Proposed Test Suite Improvements

### Structural Improvements
- [ ] **Test reorganization**: Split into `unit/`, `integration/`, `performance/` directories
- [ ] **Deterministic test data**: Replace dynamic generation with fixed test fixtures
- [ ] **Cross-platform testing**: Windows, macOS, Linux compatibility tests
- [ ] **Memory profiling**: Monitor memory usage during large database operations

### Coverage Gaps to Address
- [ ] **TUI mode testing**: Interactive mode testing (using expect/pexpect)
- [ ] **Unit tests**: Test individual Rust modules separately
- [ ] **Edge cases**: Unicode keys, binary values, corrupted databases
- [ ] **Concurrent access**: Read-only access during write operations
- [ ] **Configuration testing**: Config file interaction and override testing

### Enhanced Test Data
- [ ] **Binary data databases**: Test handling of non-UTF8 data
- [ ] **Unicode key databases**: International character support
- [ ] **Malformed databases**: Corrupted LMDB file handling
- [ ] **Permission scenarios**: Read-only filesystem testing
- [ ] **Size progression**: Empty → 10 → 1K → 100K → 1M entry databases

### Performance & Benchmarking
- [ ] **Baseline benchmarks**: Establish performance standards
- [ ] **Regression testing**: Detect performance degradation
- [ ] **Memory constraints**: Testing under limited RAM conditions
- [ ] **Large dataset handling**: Multi-GB database performance

### CI/CD Integration
- [ ] **GitHub Actions**: Multi-platform automated testing
- [ ] **Test reporting**: JUnit XML format for CI systems
- [ ] **Coverage reporting**: Code coverage metrics
- [ ] **Performance tracking**: Historical performance data

## 🤝 Contributing

When adding new features to lmdb-tui:
1. **Add corresponding tests** - Use appropriate test level (unit/integration/performance)
2. **Run full test suite** - Ensure `./run_all_tests.sh` passes
3. **Update test documentation** - Add new test categories if needed
4. **Consider edge cases** - Test error conditions and boundary cases
5. **Performance impact** - Run performance tests for significant changes