#!/bin/bash
# Master test runner for lmdb-tui
# Runs all tests and generates a comprehensive report

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}🧪 lmdb-tui Comprehensive Test Suite${NC}"
echo "===================================="
echo "Date: $(date)"
echo ""

# Check prerequisites
echo -e "${YELLOW}📋 Checking prerequisites...${NC}"

# Check if release binary exists
if [ ! -f "../target/release/lmdb-tui" ]; then
    echo -e "${RED}❌ Release binary not found!${NC}"
    echo "Please run: cargo build --release"
    exit 1
fi

# Check if Python is available
if ! command -v python3 &> /dev/null; then
    echo -e "${RED}❌ Python 3 not found!${NC}"
    echo "Python 3 is required for creating test databases"
    exit 1
fi

# Check if lmdb Python module is installed
if ! python3 -c "import lmdb" 2>/dev/null; then
    echo -e "${YELLOW}⚠️  Python lmdb module not installed${NC}"
    echo "Note: On macOS with Homebrew Python, you may need to:"
    echo "  brew install python-lmdb"
    echo "  OR use: pip3 install --user lmdb"
    echo "  OR use: pip3 install --break-system-packages lmdb"
    echo ""
    echo "Skipping Python database creation tests..."
    SKIP_PYTHON_TESTS=1
else
    SKIP_PYTHON_TESTS=0
fi

echo -e "${GREEN}✅ All prerequisites met${NC}"
echo ""

# Create test databases
if [ "$SKIP_PYTHON_TESTS" -eq 0 ]; then
    echo -e "${YELLOW}🗄️  Creating test databases...${NC}"
    cd experiments 2>/dev/null || true
    python3 create_test_databases.py || {
        echo -e "${RED}Failed to create test databases${NC}"
        exit 1
    }
    echo ""
else
    echo -e "${YELLOW}ℹ️  Using existing test databases only${NC}"
    cd experiments 2>/dev/null || true
    echo ""
fi

# Run the test suite
echo -e "${YELLOW}🚀 Running test suite...${NC}"
./test_lmdb_tui.sh

# Check test results
if [ $? -eq 0 ]; then
    echo ""
    echo -e "${GREEN}✅ All tests passed!${NC}"
    echo ""
    echo "📊 Test artifacts are in: experiments/test_results/"
    echo "📄 Full report: experiments/test_results/report.md"
else
    echo ""
    echo -e "${RED}❌ Some tests failed${NC}"
    echo "Check the detailed report in: experiments/test_results/report.md"
    exit 1
fi