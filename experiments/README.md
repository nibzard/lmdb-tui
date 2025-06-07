# lmdb-tui Test Suite

This directory contains a comprehensive automated test suite for lmdb-tui.

## 📁 Files

- **`run_all_tests.sh`** - Master test runner (start here!)
- **`test_lmdb_tui.sh`** - Main test suite with all test cases
- **`create_test_databases.py`** - Creates various LMDB test databases
- **`test_data/`** - Generated test databases (created by tests)
- **`test_results/`** - Test outputs and reports

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

## 🧪 Test Categories

### 1. **Basic Functionality**
- Help command (`--help`)
- Version command (`--version`)
- Basic argument parsing

### 2. **Error Handling**
- Missing arguments
- Non-existent databases
- Permission errors
- Correct exit codes

### 3. **Database Types**
- Unnamed databases (like conversations.lmdb)
- Named databases
- Empty databases
- Mixed databases
- Large databases (performance testing)

### 4. **Output Formats**
- Plain text mode (`--plain`)
- JSON mode (`--json`)
- Quiet mode (`--quiet`)
- Verbose mode (`--verbose`)

### 5. **CLI Integration**
- Pipeline compatibility (`|`, `>`)
- Integration with Unix tools (grep, wc, jq)
- Exit codes for scripting

### 6. **Performance**
- Execution time benchmarks
- Large database handling
- Memory usage (indirect)

## 📊 Test Results

After running, check:
- **`test_results/report.md`** - Human-readable test report
- **`test_results/*.txt`** - Individual test outputs
- **`test_results/*.log`** - Detailed test logs

## 🔧 Running Individual Tests

```bash
# Just create test databases
python3 create_test_databases.py

# Run only the test suite (assumes databases exist)
./test_lmdb_tui.sh

# Test specific database
../target/release/lmdb-tui --read-only --plain test_data/unnamed_db
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

## 🤝 Contributing

When adding new features to lmdb-tui:
1. Add corresponding tests here
2. Ensure all tests still pass
3. Update this README if needed