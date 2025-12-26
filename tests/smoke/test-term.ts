// Name: Terminal Smoke Test
// Description: Basic terminal prompt for smoke testing the GPUI executor

/**
 * SMOKE TEST: test-term.ts
 * 
 * This is a simple terminal test for verifying:
 * - SDK term() function works correctly
 * - JSONL protocol sends 'term' messages
 * - Terminal window appears and accepts commands
 * - Script exits cleanly after terminal closes
 * 
 * Expected log output from executor.rs:
 * [EXEC] execute_script_interactive: tests/smoke/test-term.ts
 * [EXEC] Received from script: {"type":"term","id":"1","command":"ls"}
 */

// Import SDK - registers global functions (arg, div, md, term)
import '../../scripts/kit-sdk';

// Log to stderr for debugging (stderr is passed through to terminal)
console.error('[SMOKE] test-term.ts starting...');
console.error('[SMOKE] SDK globals available:', typeof term);

// Simple term display - this tests:
// 1. term() sends JSONL message to Rust
// 2. Terminal window appears
// 3. Command executes (ls lists directory)
// 4. User can dismiss (closes terminal, sends submit back)
await term('ls');

console.error('[SMOKE] test-term.ts completed successfully!');
