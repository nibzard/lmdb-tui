#!/bin/bash
# Fast comprehensive test that shows results immediately

echo "🧪 lmdb-tui Fast Test Suite"
echo "============================"

BINARY="../target/release/lmdb-tui"
PASSED=0
FAILED=0

test_result() {
    if [ $1 -eq 0 ]; then
        echo "✅ PASS: $2"
        ((PASSED++))
    else
        echo "❌ FAIL: $2"
        ((FAILED++))
    fi
}

echo ""
echo "1. Basic Functionality"
echo "---------------------"

# Test help
$BINARY --help > /dev/null 2>&1
test_result $? "Help command"

# Test version
$BINARY --version > /dev/null 2>&1
test_result $? "Version command"

echo ""
echo "2. Database Detection"
echo "--------------------"

# Test conversations.lmdb
if [ -d "amp-yaml/conversations.lmdb" ]; then
    $BINARY --read-only --plain amp-yaml/conversations.lmdb | grep -q "(unnamed)"
    test_result $? "Unnamed database detection"
    
    $BINARY --read-only --json amp-yaml/conversations.lmdb | grep -q '"(unnamed)"'
    test_result $? "JSON output format"
else
    echo "⚠️  SKIP: conversations.lmdb not found"
fi

echo ""
echo "3. Error Handling"
echo "----------------"

# Test non-existent database
$BINARY --read-only --plain nonexistent > /dev/null 2>&1
if [ $? -eq 2 ]; then
    echo "✅ PASS: Non-existent database (exit code 2)"
    ((PASSED++))
else
    echo "❌ FAIL: Non-existent database (wrong exit code)"
    ((FAILED++))
fi

# Test missing arguments (shows help, which is correct)
$BINARY 2>&1 | grep -q "Usage:"
if [ $? -eq 0 ]; then
    echo "✅ PASS: Missing arguments shows help"
    ((PASSED++))
else
    echo "❌ FAIL: Missing arguments should show help"
    ((FAILED++))
fi

echo ""
echo "4. Pipeline Integration"
echo "----------------------"

if [ -d "amp-yaml/conversations.lmdb" ]; then
    # Test with wc
    COUNT=$($BINARY --read-only --plain amp-yaml/conversations.lmdb | wc -l | tr -d ' ')
    if [ "$COUNT" -eq "1" ]; then
        echo "✅ PASS: Pipeline with wc"
        ((PASSED++))
    else
        echo "❌ FAIL: Pipeline with wc (got $COUNT, expected 1)"
        ((FAILED++))
    fi
    
    # Test with grep
    $BINARY --read-only --plain amp-yaml/conversations.lmdb | grep -q "(unnamed)"
    test_result $? "Pipeline with grep"
fi

echo ""
echo "5. Output Consistency"
echo "--------------------"

if [ -d "amp-yaml/conversations.lmdb" ]; then
    # Test multiple runs produce same output
    OUT1=$($BINARY --read-only --plain amp-yaml/conversations.lmdb)
    OUT2=$($BINARY --read-only --plain amp-yaml/conversations.lmdb)
    if [ "$OUT1" = "$OUT2" ]; then
        echo "✅ PASS: Output consistency"
        ((PASSED++))
    else
        echo "❌ FAIL: Output inconsistency"
        ((FAILED++))
    fi
fi

echo ""
echo "📊 SUMMARY"
echo "=========="
echo "Passed: $PASSED"
echo "Failed: $FAILED"
echo "Total:  $((PASSED + FAILED))"

if [ $FAILED -eq 0 ]; then
    echo ""
    echo "🎉 All tests passed!"
    exit 0
else
    echo ""
    echo "❌ Some tests failed"
    exit 1
fi