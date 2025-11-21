#!/bin/bash
# Feature Gate Testing Script
# Tests various feature combinations to ensure clean compilation

set -e

echo "=== Feature Gate Testing ==="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track failures
FAILURES=0

run_test() {
    local name=$1
    shift
    echo -e "${YELLOW}Testing: $name${NC}"
    if "$@"; then
        echo -e "${GREEN}✓ $name passed${NC}"
    else
        echo -e "${RED}✗ $name failed${NC}"
        ((FAILURES++))
    fi
    echo ""
}

# 1. No default features
run_test "no-default-features" \
    cargo check --workspace --no-default-features

# 2. Default features only
run_test "default-features" \
    cargo check --workspace

# 3. All features
run_test "all-features" \
    cargo check --workspace --all-features

# 4. Individual critical features
echo -e "${YELLOW}Testing individual features...${NC}"

run_test "gemini only" \
    cargo check --workspace --no-default-features --features gemini

run_test "database only" \
    cargo check --workspace --no-default-features --features database

run_test "discord only" \
    cargo check --workspace --no-default-features --features discord

run_test "tui only" \
    cargo check --workspace --no-default-features --features tui

# 5. Common feature combinations
echo -e "${YELLOW}Testing feature combinations...${NC}"

run_test "gemini + database" \
    cargo check --workspace --no-default-features --features gemini,database

run_test "gemini + discord" \
    cargo check --workspace --no-default-features --features gemini,discord

run_test "database + tui" \
    cargo check --workspace --no-default-features --features database,tui

# 6. Clippy checks
echo -e "${YELLOW}Running clippy checks...${NC}"

run_test "clippy no-default-features" \
    cargo clippy --workspace --no-default-features -- -D warnings

run_test "clippy default-features" \
    cargo clippy --workspace -- -D warnings

run_test "clippy all-features" \
    cargo clippy --workspace --all-features -- -D warnings

# Summary
echo ""
echo "=== Summary ==="
if [ $FAILURES -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}$FAILURES test(s) failed${NC}"
    exit 1
fi
