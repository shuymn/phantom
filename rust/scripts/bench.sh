#!/bin/bash
# Benchmark runner script for Phantom

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Default values
BENCH_FILTER=""
SAVE_BASELINE=""
COMPARE_BASELINE=""
PROFILE_TIME=""

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -f|--filter)
            BENCH_FILTER="$2"
            shift 2
            ;;
        -s|--save)
            SAVE_BASELINE="$2"
            shift 2
            ;;
        -c|--compare)
            COMPARE_BASELINE="$2"
            shift 2
            ;;
        -p|--profile-time)
            PROFILE_TIME="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  -f, --filter <PATTERN>    Run only benchmarks matching pattern"
            echo "  -s, --save <NAME>         Save results as baseline with name"
            echo "  -c, --compare <NAME>      Compare against baseline with name"
            echo "  -p, --profile-time <SEC>  Set profiling time in seconds (default: 5)"
            echo "  -h, --help                Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0                        # Run all benchmarks"
            echo "  $0 -f string              # Run string-related benchmarks"
            echo "  $0 -s main                # Save baseline named 'main'"
            echo "  $0 -c main                # Compare against 'main' baseline"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

# Build benchmark command
BENCH_CMD="cargo bench"

if [ -n "$BENCH_FILTER" ]; then
    BENCH_CMD="$BENCH_CMD -- $BENCH_FILTER"
fi

if [ -n "$SAVE_BASELINE" ]; then
    BENCH_CMD="$BENCH_CMD --save-baseline $SAVE_BASELINE"
elif [ -n "$COMPARE_BASELINE" ]; then
    BENCH_CMD="$BENCH_CMD --baseline $COMPARE_BASELINE"
fi

if [ -n "$PROFILE_TIME" ]; then
    BENCH_CMD="$BENCH_CMD --profile-time $PROFILE_TIME"
fi

# Run benchmarks
echo -e "${GREEN}Running benchmarks...${NC}"
echo -e "${YELLOW}Command: $BENCH_CMD${NC}"
echo ""

eval $BENCH_CMD

echo ""
echo -e "${GREEN}Benchmarks complete!${NC}"
echo -e "HTML reports available at: ${YELLOW}target/criterion/report/index.html${NC}"