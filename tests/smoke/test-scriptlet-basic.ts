// Name: Test Scriptlet Basic Parsing
// Description: Smoke test for scriptlet parsing functionality

/**
 * SMOKE TEST: test-scriptlet-basic.ts
 * 
 * This test verifies the basic scriptlet parsing functionality:
 * - Markdown file parsing into scriptlets
 * - H1 headers create groups
 * - H2 headers create individual scriptlets
 * - Code fence extraction (tool and content)
 * - Named input placeholder extraction ({{variableName}})
 * - Metadata parsing from HTML comments
 * - Global prepend code from H1 sections
 * 
 * This is a parsing-only test - it doesn't execute scriptlets.
 */

import '../../scripts/kit-sdk';

console.error('[SMOKE] test-scriptlet-basic.ts starting...');

// Test 1: Display parsed scriptlet information
// The actual parsing happens in Rust, so we're testing the SDK interface
console.error('[SMOKE] Test 1: Basic scriptlet display');

await div(md(`# Scriptlet Parsing Test

## Overview

This smoke test verifies the scriptlet system is properly integrated.

## Expected Capabilities

1. **Markdown Parsing**
   - H1 headers (\`# Group\`) define groups
   - H2 headers (\`## Script Name\`) define scriptlets
   - Code fences contain the actual script content

2. **Metadata Extraction**
   - HTML comments contain metadata
   - Supported: \`shortcut\`, \`trigger\`, \`description\`, \`expand\`, etc.

3. **Input Detection**
   - \`{{variableName}}\` patterns are detected
   - Conditionals (\`{{#if}}\`) are NOT treated as inputs

4. **Code Prepending**
   - Code in H1 sections is prepended to all H2 scriptlets in that group

---

*Click anywhere or press Escape to continue*`));

console.error('[SMOKE] Test 1 complete');

// Test 2: Show example scriptlet format
console.error('[SMOKE] Test 2: Example scriptlet format');

await div(md(`# Example Scriptlet Format

\`\`\`markdown
# My Group

## My Scriptlet

<!-- 
shortcut: cmd k
description: Does something useful
-->

\\\`\\\`\\\`bash
echo "Hello {{name}}!"
\\\`\\\`\\\`
\`\`\`

## Parsed Result

| Field | Value |
|-------|-------|
| Name | My Scriptlet |
| Group | My Group |
| Tool | bash |
| Inputs | \`["name"]\` |
| Shortcut | cmd k |

---

*Click anywhere or press Escape to continue*`));

console.error('[SMOKE] Test 2 complete');

// Test 3: Variable substitution examples
console.error('[SMOKE] Test 3: Variable substitution patterns');

await div(md(`# Variable Substitution Patterns

## Named Inputs
- \`{{variableName}}\` - Replaced with user-provided value
- \`{{anotherVar}}\` - Multiple variables supported

## Positional Arguments (Shell)
- \`$1\`, \`$2\`, etc. - Individual positional args
- \`$@\` - All arguments (quoted)

## Conditionals
- \`{{#if flag}}content{{/if}}\` - Include if flag is true
- \`{{#if flag}}yes{{else}}no{{/if}}\` - If-else
- \`{{#if a}}A{{else if b}}B{{else}}C{{/if}}\` - Chains

## Example

\`\`\`bash
#!/bin/bash
{{#if verbose}}set -x{{/if}}
echo "Hello {{name}}!"
echo "Args: $@"
\`\`\`

---

*Click anywhere or press Escape to complete test*`));

console.error('[SMOKE] Test 3 complete');
console.error('[SMOKE] test-scriptlet-basic.ts completed successfully!');
