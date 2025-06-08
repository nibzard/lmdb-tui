#!/bin/bash

set -e

echo "🧪 Running Comprehensive TUI Tests"
echo "=================================="

# Create test snapshots directory
mkdir -p test_snapshots

# Run the comprehensive TUI tests
echo "📊 Running TUI test suite..."
cargo test tui_comprehensive --test tui_comprehensive -- --nocapture

echo ""
echo "📁 Test artifacts generated in test_snapshots/"
echo ""

# List generated snapshots
if [ -d "test_snapshots" ] && [ "$(ls -A test_snapshots)" ]; then
    echo "📸 Generated snapshots:"
    find test_snapshots -name "*.json" | head -10
    
    if [ $(find test_snapshots -name "*.json" | wc -l) -gt 10 ]; then
        echo "   ... and $(( $(find test_snapshots -name "*.json" | wc -l) - 10 )) more"
    fi
    
    echo ""
    echo "📈 Test reports:"
    find test_snapshots -name "*_report.json"
    
    echo ""
    echo "💾 Total files created: $(find test_snapshots -type f | wc -l)"
    echo "📦 Total size: $(du -sh test_snapshots | cut -f1)"
else
    echo "⚠️  No snapshots generated"
fi

echo ""
echo "🎯 Next steps:"
echo "  1. Review test_snapshots/*.json files"
echo "  2. Use AI tools to analyze UI consistency and UX"
echo "  3. Compare snapshots across test runs for regression detection"
echo "  4. View .ansi files in terminal for colored output"
echo ""
echo "📖 Example analysis command:"
echo "  cat test_snapshots/main_navigation_*_report.json | jq '.test_session.snapshots[] | .app_state'"