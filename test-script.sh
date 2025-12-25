#!/bin/bash
# Helper to test script execution via the running app
# Usage: ./test-script.sh <script-name>
#        ./test-script.sh --sdk              # Run all SDK tests
#        ./test-script.sh --sdk test-arg.ts  # Run single SDK test

set -e

# Configuration
CMD_FILE="/tmp/script-kit-gpui-cmd.txt"
LOG_FILE="${TMPDIR:-/tmp}/script-kit-gpui.log"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# =============================================================================
# Help
# =============================================================================
if [[ "$1" == "--help" || "$1" == "-h" ]]; then
    echo "Usage:"
    echo "  ./test-script.sh <script-name>        Run script via GPUI app"
    echo "  ./test-script.sh --sdk                Run all SDK tests"
    echo "  ./test-script.sh --sdk test-arg.ts    Run single SDK test"
    echo ""
    echo "Examples:"
    echo "  ./test-script.sh hello-world.ts"
    echo "  ./test-script.sh tests/smoke/hello-world-args.ts"
    echo "  ./test-script.sh --sdk"
    echo "  ./test-script.sh --sdk test-md.ts"
    echo ""
    echo "Environment:"
    echo "  SDK_TEST_TIMEOUT=30    Max seconds per test"
    echo "  SDK_TEST_VERBOSE=true  Extra debug output"
    exit 0
fi

# =============================================================================
# SDK Test Mode
# =============================================================================
if [[ "$1" == "--sdk" ]]; then
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}SDK Test Suite${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    
    shift  # Remove --sdk from arguments
    
    # Check if bun is available
    if ! command -v bun &> /dev/null; then
        echo -e "${RED}Error: bun is not installed${NC}"
        echo "Install with: curl -fsSL https://bun.sh/install | bash"
        exit 1
    fi
    
    # Run the test runner
    if [[ -n "$1" ]]; then
        echo -e "${YELLOW}Running single test: $1${NC}"
        bun run "$PROJECT_ROOT/scripts/test-runner.ts" "$1"
    else
        echo -e "${YELLOW}Running all SDK tests...${NC}"
        bun run "$PROJECT_ROOT/scripts/test-runner.ts"
    fi
    
    exit $?
fi

# =============================================================================
# Interactive Test Mode (original behavior)
# =============================================================================
SCRIPT_NAME="${1:-smoke-test-simple.ts}"

echo -e "${BLUE}Testing script: ${YELLOW}$SCRIPT_NAME${NC}"
echo "run:$SCRIPT_NAME" > "$CMD_FILE"

sleep 2

echo ""
echo -e "${BLUE}=== Recent log entries ===${NC}"
if [[ -f "$LOG_FILE" ]]; then
    grep -E "\[TEST\]|\[EXEC\]|\[SMOKE\]|\[SDK\]" "$LOG_FILE" 2>/dev/null | tail -20 || echo "No matching log entries found"
else
    echo -e "${YELLOW}Warning: Log file not found at $LOG_FILE${NC}"
    echo "The app may not have created a log file yet."
fi


