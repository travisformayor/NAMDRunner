#!/bin/bash
# Comprehensive Test Coverage Script for NAMDRunner
# Follows NAMDRunner testing philosophy: test our logic, not external libraries

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REQUIRED_COVERAGE=80
PROJECT_ROOT=$(pwd)
COVERAGE_DIR="$PROJECT_ROOT/coverage"

echo -e "${BLUE}🧪 NAMDRunner Comprehensive Testing Suite${NC}"
echo -e "${BLUE}===========================================${NC}"

# Create coverage directory
mkdir -p "$COVERAGE_DIR"

# Function to print section headers
print_section() {
    echo -e "\n${BLUE}📋 $1${NC}"
    echo -e "${BLUE}$(printf '=%.0s' $(seq 1 ${#1}))${NC}"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Validate environment
print_section "Environment Validation"

# Check Rust environment
if ! command_exists cargo; then
    echo -e "${RED}❌ Cargo not found. Please install Rust toolchain.${NC}"
    exit 1
fi

# Check Node.js environment
if ! command_exists npm; then
    echo -e "${RED}❌ npm not found. Please install Node.js.${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Environment validated${NC}"

# Run Rust unit tests with coverage
print_section "Rust Unit Tests (Backend Business Logic)"

cd src-tauri

echo "Running Rust tests for security, validation, and business logic..."

# Install cargo-tarpaulin for coverage if not present
if ! command_exists cargo-tarpaulin; then
    echo "Installing cargo-tarpaulin for code coverage..."
    cargo install cargo-tarpaulin
fi

# Run tests with coverage reporting
echo "Executing Rust tests with coverage analysis..."
cargo tarpaulin \
    --out Json \
    --output-dir "../$COVERAGE_DIR" \
    --timeout 120 \
    --verbose \
    --skip-clean

# Parse coverage results
RUST_COVERAGE=$(python3 -c "
import json
try:
    with open('../$COVERAGE_DIR/tarpaulin-report.json', 'r') as f:
        data = json.load(f)
    coverage = data['files']['coverage']
    print(f'{coverage:.1f}')
except:
    print('0.0')
")

echo -e "Rust test coverage: ${RUST_COVERAGE}%"

if (( $(echo "$RUST_COVERAGE >= $REQUIRED_COVERAGE" | bc -l) )); then
    echo -e "${GREEN}✅ Rust coverage meets requirement (${RUST_COVERAGE}% >= ${REQUIRED_COVERAGE}%)${NC}"
    RUST_PASS=true
else
    echo -e "${RED}❌ Rust coverage below requirement (${RUST_COVERAGE}% < ${REQUIRED_COVERAGE}%)${NC}"
    RUST_PASS=false
fi

cd ..

# Run TypeScript unit tests
print_section "TypeScript Unit Tests (Frontend Business Logic)"

echo "Running TypeScript/Svelte unit tests..."

# Run Vitest with coverage
npm run test -- --coverage --reporter=json --outputFile="$COVERAGE_DIR/vitest-results.json"

# Check if coverage report exists
if [ -f "coverage/coverage-summary.json" ]; then
    TS_COVERAGE=$(node -e "
        const fs = require('fs');
        try {
            const coverage = JSON.parse(fs.readFileSync('coverage/coverage-summary.json', 'utf8'));
            console.log(coverage.total.lines.pct);
        } catch (e) {
            console.log('0');
        }
    ")

    echo -e "TypeScript test coverage: ${TS_COVERAGE}%"

    if (( $(echo "$TS_COVERAGE >= $REQUIRED_COVERAGE" | bc -l) )); then
        echo -e "${GREEN}✅ TypeScript coverage meets requirement (${TS_COVERAGE}% >= ${REQUIRED_COVERAGE}%)${NC}"
        TS_PASS=true
    else
        echo -e "${RED}❌ TypeScript coverage below requirement (${TS_COVERAGE}% < ${REQUIRED_COVERAGE}%)${NC}"
        TS_PASS=false
    fi
else
    echo -e "${YELLOW}⚠️ TypeScript coverage report not generated${NC}"
    TS_COVERAGE=0
    TS_PASS=false
fi

# Run Autonomous UI Tests
print_section "Autonomous UI Testing (Demo Mode Validation)"

echo "Checking if Vite dev server is running..."

# Check if dev server is running
if curl -s http://localhost:1420 > /dev/null; then
    echo -e "${GREEN}✅ Dev server is running${NC}"
    SERVER_RUNNING=true
else
    echo -e "${YELLOW}⚠️ Dev server not running, starting it...${NC}"
    echo "Starting Vite dev server in background..."
    npm run dev &
    DEV_SERVER_PID=$!

    # Wait for server to start
    echo "Waiting for dev server to start..."
    for i in {1..60}; do
        if curl -s http://localhost:1420 > /dev/null; then
            echo -e "${GREEN}✅ Dev server started successfully${NC}"
            SERVER_RUNNING=true
            break
        fi
        sleep 2
        echo -n "."
    done

    if [ "$SERVER_RUNNING" != true ]; then
        echo -e "${RED}❌ Dev server failed to start${NC}"
        SERVER_RUNNING=false
    fi
fi

if [ "$SERVER_RUNNING" = true ]; then
    echo "Running autonomous UI tests..."

    # Set headless mode for CI environments
    export DISPLAY=${DISPLAY:-:99}

    # Run autonomous UI validation
    if node tests/ui/autonomous-demo-validation.js; then
        echo -e "${GREEN}✅ Autonomous UI tests passed${NC}"
        UI_PASS=true
    else
        echo -e "${RED}❌ Autonomous UI tests failed${NC}"
        UI_PASS=false
    fi

    # Clean up dev server if we started it
    if [ -n "$DEV_SERVER_PID" ]; then
        echo "Stopping dev server..."
        kill $DEV_SERVER_PID 2>/dev/null || true
    fi
else
    echo -e "${RED}❌ Cannot run UI tests without dev server${NC}"
    UI_PASS=false
fi

# Security Testing Validation
print_section "Security Testing Validation"

echo "Validating security test coverage..."

# Count security tests in validation module
SECURITY_TESTS=$(cd src-tauri && grep -r "test.*malicious\|malicious.*test\|security.*test\|injection.*test" src/validation.rs | wc -l)

if [ "$SECURITY_TESTS" -gt 5 ]; then
    echo -e "${GREEN}✅ Security tests present (${SECURITY_TESTS} security-related tests found)${NC}"
    SECURITY_PASS=true
else
    echo -e "${RED}❌ Insufficient security tests (${SECURITY_TESTS} found, need > 5)${NC}"
    SECURITY_PASS=false
fi

# Test Performance
print_section "Test Performance Validation"

echo "Validating test execution performance..."

# Check if unit tests run in reasonable time (< 30 seconds)
START_TIME=$(date +%s)
cd src-tauri && timeout 30s cargo test --quiet > /dev/null 2>&1
TEST_EXIT_CODE=$?
cd ..
END_TIME=$(date +%s)
TEST_DURATION=$((END_TIME - START_TIME))

if [ $TEST_EXIT_CODE -eq 0 ] && [ $TEST_DURATION -lt 30 ]; then
    echo -e "${GREEN}✅ Unit tests complete in ${TEST_DURATION}s (< 30s requirement)${NC}"
    PERF_PASS=true
else
    echo -e "${RED}❌ Unit tests too slow or failed (${TEST_DURATION}s)${NC}"
    PERF_PASS=false
fi

# Generate Final Report
print_section "Test Results Summary"

# Create comprehensive report
cat > "$COVERAGE_DIR/comprehensive-test-report.json" << EOF
{
    "timestamp": "$(date -Iseconds)",
    "testing_philosophy": "Business logic focus - test our code, not external libraries",
    "coverage_requirement": $REQUIRED_COVERAGE,
    "results": {
        "rust_backend": {
            "coverage": $RUST_COVERAGE,
            "passed": $RUST_PASS,
            "focus": "Security validation, shell safety, database transactions, automation workflows"
        },
        "typescript_frontend": {
            "coverage": $TS_COVERAGE,
            "passed": $TS_PASS,
            "focus": "Business logic, stores, validation utilities, form logic"
        },
        "autonomous_ui": {
            "passed": $UI_PASS,
            "focus": "Demo mode workflow validation, autonomous interaction testing"
        },
        "security_testing": {
            "passed": $SECURITY_PASS,
            "focus": "Malicious input handling, injection prevention, path traversal"
        },
        "performance": {
            "test_duration": "${TEST_DURATION}s",
            "passed": $PERF_PASS,
            "focus": "Fast execution for development feedback"
        }
    }
}
EOF

# Print summary
echo -e "\n${BLUE}📊 NAMDRunner Test Results${NC}"
echo -e "${BLUE}=====================================${NC}"

echo -e "Rust Backend Coverage:     ${RUST_COVERAGE}% $([ "$RUST_PASS" = true ] && echo -e "${GREEN}✅" || echo -e "${RED}❌")${NC}"
echo -e "TypeScript Frontend:       ${TS_COVERAGE}% $([ "$TS_PASS" = true ] && echo -e "${GREEN}✅" || echo -e "${RED}❌")${NC}"
echo -e "Autonomous UI Tests:       $([ "$UI_PASS" = true ] && echo -e "${GREEN}✅ PASSED" || echo -e "${RED}❌ FAILED")${NC}"
echo -e "Security Testing:          $([ "$SECURITY_PASS" = true ] && echo -e "${GREEN}✅ PASSED" || echo -e "${RED}❌ FAILED")${NC}"
echo -e "Performance:               $([ "$PERF_PASS" = true ] && echo -e "${GREEN}✅ ${TEST_DURATION}s" || echo -e "${RED}❌ ${TEST_DURATION}s")${NC}"

# Determine overall result
if [ "$RUST_PASS" = true ] && [ "$TS_PASS" = true ] && [ "$UI_PASS" = true ] && [ "$SECURITY_PASS" = true ] && [ "$PERF_PASS" = true ]; then
    echo -e "\n${GREEN}🎉 ALL TESTS PASSED - Complete!${NC}"
    echo -e "${GREEN}✅ Production-ready testing coverage achieved${NC}"
    echo -e "${GREEN}✅ NAMDRunner testing philosophy followed${NC}"
    echo -e "${GREEN}✅ Security and business logic comprehensively tested${NC}"
    exit 0
else
    echo -e "\n${RED}❌ SOME TESTS FAILED${NC}"
    echo -e "${RED}Requirements not fully met${NC}"

    if [ "$RUST_PASS" != true ]; then
        echo -e "${RED}- Rust backend coverage below ${REQUIRED_COVERAGE}%${NC}"
    fi
    if [ "$TS_PASS" != true ]; then
        echo -e "${RED}- TypeScript frontend coverage below ${REQUIRED_COVERAGE}%${NC}"
    fi
    if [ "$UI_PASS" != true ]; then
        echo -e "${RED}- Autonomous UI tests failed${NC}"
    fi
    if [ "$SECURITY_PASS" != true ]; then
        echo -e "${RED}- Insufficient security test coverage${NC}"
    fi
    if [ "$PERF_PASS" != true ]; then
        echo -e "${RED}- Test performance requirements not met${NC}"
    fi

    exit 1
fi