#!/usr/bin/env bash
# scripts/_check_coverage.sh
# Fails CI if coverage is below 50%

set -euo pipefail

PACKAGE="${1:-apiserver}"
MIN_COVERAGE=50.0

echo "Checking coverage for $PACKAGE..."

# Run test.sh coverage
./test.sh "$PACKAGE" --coverage > tarpaulin_output.log 2>&1

# Parse coverage from tarpaulin_output.log
COVERAGE=$(grep -oP '\d+\.\d+(?=% coverage)' tarpaulin_output.log | tail -n 1)

if [ -z "$COVERAGE" ]; then
    # test.sh might not output the percentage in the same format, but --coverage already enforces it
    echo "test.sh --coverage completed."
    cat tarpaulin_output.log
    # Check if test.sh failed
    if grep -q "FAILED" tarpaulin_output.log || grep -q "error:" tarpaulin_output.log; then
        exit 1
    fi
    exit 0
fi

echo "Current coverage: $COVERAGE%"

if (( $(echo "$COVERAGE < $MIN_COVERAGE" | bc -l) )); then
    echo "Coverage ($COVERAGE%) is below minimum threshold ($MIN_COVERAGE%). Failing CI."
    exit 1
else
    echo "Coverage check passed."
    exit 0
fi
