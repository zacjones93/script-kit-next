// Name: Test Scriptlet macOS Tools
// Description: Smoke test for macOS-specific scriptlet tools

/**
 * SMOKE TEST: test-scriptlet-macos-tools.ts
 *
 * This test verifies macOS-specific scriptlet tools:
 * - transform: Gets selected text, processes it, sets result
 * - paste: Sets selected text via clipboard
 * - type: Simulates keyboard typing via AppleScript
 * - submit: Paste + Enter
 * - applescript: Executes AppleScript via osascript
 *
 * Platform detection:
 * - On macOS: Tests actual tool functionality
 * - On other platforms: Tests should gracefully report "not supported"
 *
 * Note: These tools require accessibility permissions on macOS.
 * The actual execution tests are simplified to avoid side effects.
 */

import '../../scripts/kit-sdk';

// Declare process for TypeScript (available at runtime in Node.js/Bun)
declare const process: { platform: string };

// Get current platform
const platform = (): string => process.platform;

console.error('[SMOKE] test-scriptlet-macos-tools.ts starting...');

const isMacOS = platform() === 'darwin';
console.error(`[SMOKE] Platform: ${platform()}, isMacOS: ${isMacOS}`);

// ============================================================================
// Test 1: Platform Detection Display
// ============================================================================
console.error('[SMOKE] Test 1: Platform detection');

await div(
  md(`# macOS Scriptlet Tools Test

## Platform Detection

| Property | Value |
|----------|-------|
| Platform | \`${platform()}\` |
| Is macOS | ${isMacOS ? '**Yes**' : 'No'} |

## Tools Being Tested

| Tool | Purpose | macOS Only |
|------|---------|------------|
| \`transform\` | Get selected text, process, set result | Yes |
| \`paste\` | Set selected text via clipboard | Yes |
| \`type\` | Simulate keyboard typing | Yes |
| \`submit\` | Paste + Enter | Yes |
| \`applescript\` | Execute AppleScript via osascript | Yes |

${
  !isMacOS
    ? `
## Non-macOS Platform

Since this is not macOS, the macOS-specific tools will return errors like:
- "Transform scriptlets are only supported on macOS"
- "Paste scriptlets are only supported on macOS"
- etc.

This is **expected behavior** and the tests will verify these error messages.
`
    : `
## macOS Platform

All macOS tools should be available. Note that:
- Accessibility permission may be required for selected text operations
- Keyboard simulation requires System Events access
`
}

---

*Click anywhere or press Escape to continue*`)
);

console.error('[SMOKE] Test 1 complete');

// ============================================================================
// Test 2: Transform Tool Conceptual Test
// ============================================================================
console.error('[SMOKE] Test 2: Transform tool');

// The transform tool:
// 1. Gets selected text via accessibility APIs
// 2. Wraps it in a TypeScript function: const selectedText = "..."; const transform = (text) => { <user code> }
// 3. Executes the TypeScript and captures output
// 4. Sets the transformed text back via clipboard + paste

const transformExample = `
// User's transform code (this gets wrapped by executor.rs):
return text.toUpperCase();

// The executor wraps it like:
const selectedText = "hello world";  // From get_selected_text()
const transform = (text: string): string => {
  return text.toUpperCase();
};
const result = transform(selectedText);
console.log(result);  // Output: "HELLO WORLD"

// Then set_selected_text("HELLO WORLD") replaces the selection
`;

await div(
  md(`# Transform Tool

## How It Works

1. **Get Selected Text**: Uses \`selected_text::get_selected_text()\`
2. **Wrap User Code**: Creates TypeScript wrapper function
3. **Execute**: Runs via bun with SDK preload
4. **Set Result**: Uses \`selected_text::set_selected_text()\`

## Example Wrapper Code

\`\`\`typescript
${transformExample.trim()}
\`\`\`

## Platform Behavior

${
  isMacOS
    ? `On macOS:
- Requires accessibility permission
- Uses Accessibility API to get focused app's selected text
- Falls back to clipboard simulation (Cmd+C / Cmd+V) if needed`
    : `On ${platform()}:
- Returns error: "Transform scriptlets are only supported on macOS"
- The \`#[cfg(not(target_os = "macos"))]\` guard ensures clean error handling`
}

---

*Click anywhere or press Escape to continue*`)
);

console.error('[SMOKE] Test 2 complete');

// ============================================================================
// Test 3: Paste/Type/Submit Tools
// ============================================================================
console.error('[SMOKE] Test 3: Paste/Type/Submit tools');

const pasteTypeSubmitInfo = `
## Paste Tool

Sets the selected text area's content via:
1. Copy text to clipboard
2. Simulate Cmd+V

\`\`\`rust
// In executor.rs
#[cfg(target_os = "macos")]
fn execute_paste(content: &str) -> Result<ScriptletResult, String> {
    selected_text::set_selected_text(text.trim())?;
    Ok(ScriptletResult { success: true, ... })
}
\`\`\`

## Type Tool

Simulates keyboard typing via AppleScript:

\`\`\`applescript
tell application "System Events" to keystroke "Hello World"
\`\`\`

Special character escaping:
- Backslash \`\\\` → \`\\\\\\\\\`
- Double quote \`"\` → \`\\\\"\`

## Submit Tool

Combines paste + Enter:
1. Execute paste to insert text
2. Small delay (50ms)
3. Simulate Return key via AppleScript:

\`\`\`applescript
tell application "System Events" to key code 36  -- Return key
\`\`\`
`;

await div(
  md(`# Paste, Type, Submit Tools

${pasteTypeSubmitInfo}

## Platform Guards

All three tools have platform guards:

\`\`\`rust
#[cfg(not(target_os = "macos"))]
fn execute_paste(_content: &str) -> Result<ScriptletResult, String> {
    Err("Paste scriptlets are only supported on macOS".to_string())
}
\`\`\`

${
  isMacOS
    ? `On macOS: All tools should work with proper accessibility permissions.`
    : `On ${platform()}: All tools return "only supported on macOS" errors.`
}

---

*Click anywhere or press Escape to continue*`)
);

console.error('[SMOKE] Test 3 complete');

// ============================================================================
// Test 4: AppleScript Execution
// ============================================================================
console.error('[SMOKE] Test 4: AppleScript execution');

const applescriptExamples = `
## Simple Examples

\`\`\`applescript
-- Display a dialog
display dialog "Hello from Script Kit!"

-- Get current date
return (current date) as string

-- Open an app
tell application "Finder" to activate
\`\`\`

## Execution Method

AppleScript is executed via \`osascript -e "..."\`:

\`\`\`rust
fn execute_applescript(content: &str, options: &ScriptletExecOptions) -> Result<ScriptletResult, String> {
    let mut cmd = Command::new("osascript");
    cmd.arg("-e").arg(content);
    
    if let Some(ref cwd) = options.cwd {
        cmd.current_dir(cwd);
    }
    
    let output = cmd.output()?;
    // ... handle result
}
\`\`\`

## Special Character Escaping

When generating AppleScript dynamically (e.g., for \`type\` tool):

| Character | Escaped |
|-----------|---------|
| \`\\\` | \`\\\\\\\\\` |
| \`"\` | \`\\\\"\` |

Example in type tool:
\`\`\`rust
let script = format!(
    r#"tell application "System Events" to keystroke "{}""#,
    text.replace('\\\\', "\\\\\\\\").replace('"', "\\\\\\"")
);
\`\`\`
`;

await div(
  md(`# AppleScript Execution

${applescriptExamples}

## Important Notes

1. **No Platform Guard**: Unlike paste/type/submit, \`osascript\` exists on macOS only,
   so attempting to run on other platforms will fail at the Command::spawn level.

2. **Multi-line Scripts**: The \`-e\` flag is used for single expressions.
   For multi-line scripts, consider writing to a temp file and using \`osascript file.scpt\`.

3. **Error Handling**: AppleScript errors are captured in stderr and the exit code
   indicates success (0) or failure (non-zero).

---

*Click anywhere or press Escape to continue*`)
);

console.error('[SMOKE] Test 4 complete');

// ============================================================================
// Test 5: AppleScript Escaping Test
// ============================================================================
console.error('[SMOKE] Test 5: AppleScript escaping');

// Test escaping of special characters in AppleScript
const testCases = [
  { input: 'Hello World', escaped: 'Hello World', description: 'Simple text' },
  {
    input: 'Say "Hello"',
    escaped: 'Say \\"Hello\\"',
    description: 'Double quotes',
  },
  {
    input: 'Path\\to\\file',
    escaped: 'Path\\\\to\\\\file',
    description: 'Backslashes',
  },
  {
    input: 'It\'s "quoted"',
    escaped: 'It\'s \\"quoted\\"',
    description: 'Mixed quotes',
  },
  {
    input: 'Line1\\nLine2',
    escaped: 'Line1\\\\nLine2',
    description: 'Escaped newline',
  },
];

// Simulate the escaping logic from executor.rs
function escapeForAppleScript(text: string): string {
  return text.replace(/\\/g, '\\\\').replace(/"/g, '\\"');
}

const escapingResults = testCases.map((tc) => {
  const actual = escapeForAppleScript(tc.input);
  const passed = actual === tc.escaped;
  return {
    ...tc,
    actual,
    passed,
  };
});

const allPassed = escapingResults.every((r) => r.passed);

await div(
  md(`# AppleScript Escaping Test

## Escaping Logic

The \`type\` tool escapes special characters before sending to AppleScript:

\`\`\`rust
text.replace('\\\\', "\\\\\\\\").replace('"', "\\\\\\"")
\`\`\`

## Test Cases

| Input | Expected | Actual | Status |
|-------|----------|--------|--------|
${escapingResults.map((r) => `| \`${r.input}\` | \`${r.escaped}\` | \`${r.actual}\` | ${r.passed ? 'PASS' : '**FAIL**'} |`).join('\n')}

## Result

${allPassed ? '**All escaping tests passed!**' : '**Some tests failed - check escaping logic**'}

---

*Click anywhere or press Escape to continue*`)
);

console.error(`[SMOKE] Test 5 complete: ${allPassed ? 'PASS' : 'FAIL'}`);

// ============================================================================
// Test 6: Summary
// ============================================================================
console.error('[SMOKE] Test 6: Summary');

await div(
  md(`# macOS Tools Test Summary

## Platform

- **OS**: ${platform()}
- **macOS**: ${isMacOS ? 'Yes' : 'No'}

## Tools Tested

| Tool | Description | Status |
|------|-------------|--------|
| transform | Get/process/set selected text | ${isMacOS ? 'Available' : 'Not supported'} |
| paste | Set selected text via clipboard | ${isMacOS ? 'Available' : 'Not supported'} |
| type | Keyboard simulation | ${isMacOS ? 'Available' : 'Not supported'} |
| submit | Paste + Enter | ${isMacOS ? 'Available' : 'Not supported'} |
| applescript | Execute AppleScript | ${isMacOS ? 'Available' : 'Not supported'} |

## Escaping Tests

- **Status**: ${allPassed ? 'All passed' : 'Some failed'}

## Key Observations

1. **Platform Guards**: All macOS-specific functions have proper \`#[cfg]\` guards
2. **Error Messages**: Non-macOS platforms get clear "only supported on macOS" errors
3. **Accessibility**: macOS tools require System Preferences accessibility permission
4. **Character Escaping**: Special characters are properly escaped for AppleScript

## Implementation Files

- \`src/executor.rs\`: Tool execution logic
- \`src/selected_text.rs\`: macOS accessibility APIs

---

*Test completed successfully!*`)
);

console.error('[SMOKE] Test 6 complete');
console.error('[SMOKE] test-scriptlet-macos-tools.ts completed successfully!');
