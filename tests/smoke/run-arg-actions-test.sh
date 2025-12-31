#!/bin/bash
# Test script for arg actions panel with SimulateKey
# This sends a sequence of commands to test Cmd+K and arrow navigation

set -e

cd "$(dirname "$0")/../.."

echo "[TEST] Building app..."
cargo build 2>&1 | tail -3

echo "[TEST] Starting arg actions simulation test..."

# Create a pipe for sending commands
PIPE=$(mktemp -u)
mkfifo "$PIPE"

# Start the app with stdin from pipe, capture output
timeout 15 ./target/debug/script-kit-gpui < "$PIPE" 2>&1 &
APP_PID=$!

# Give the app time to start
sleep 0.5

# Send run command
echo "[TEST] Sending run command..."
echo '{"type": "run", "path": "'$(pwd)'/tests/smoke/test-arg-actions-simulate.ts"}' > "$PIPE"

# Wait for arg prompt to appear
sleep 1

echo "[TEST] Sending Cmd+K to open actions panel..."
echo '{"type": "simulateKey", "key": "k", "modifiers": ["cmd"]}' > "$PIPE"

# Wait for actions panel to open
sleep 0.5

echo "[TEST] Sending Down arrow to select second action..."
echo '{"type": "simulateKey", "key": "down", "modifiers": []}' > "$PIPE"

sleep 0.3

echo "[TEST] Sending another Down arrow to select third action..."
echo '{"type": "simulateKey", "key": "down", "modifiers": []}' > "$PIPE"

sleep 0.3

echo "[TEST] Sending Escape to close actions panel..."
echo '{"type": "simulateKey", "key": "escape", "modifiers": []}' > "$PIPE"

sleep 0.3

echo "[TEST] Sending Enter to select from arg choices..."
echo '{"type": "simulateKey", "key": "enter", "modifiers": []}' > "$PIPE"

# Wait for completion
sleep 1

# Clean up
rm -f "$PIPE"
wait $APP_PID 2>/dev/null || true

echo "[TEST] Test complete!"
