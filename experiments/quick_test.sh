#!/bin/bash
# Quick test of lmdb-tui functionality

echo "🧪 Quick lmdb-tui Test"
echo "====================="

# Test with conversations.lmdb
echo ""
echo "1. Testing with conversations.lmdb:"
echo "-----------------------------------"
../target/release/lmdb-tui --read-only --plain amp-yaml/conversations.lmdb
echo ""

echo "2. JSON output:"
echo "---------------"
../target/release/lmdb-tui --read-only --json amp-yaml/conversations.lmdb
echo ""

echo "3. Version check:"
echo "-----------------"
../target/release/lmdb-tui --version
echo ""

echo "4. Pipeline test:"
echo "-----------------"
echo -n "Database count: "
../target/release/lmdb-tui --read-only --plain amp-yaml/conversations.lmdb | wc -l
echo ""

echo "5. Error handling (non-existent):"
echo "---------------------------------"
../target/release/lmdb-tui --read-only --plain nonexistent 2>&1 || echo "Exit code: $?"
echo ""

echo "✅ Quick test complete!"