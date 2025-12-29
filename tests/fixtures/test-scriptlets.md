# Test Scriptlets

Sample scriptlets for smoke testing the scriptlet parsing and execution system.
These scriptlets are used by `test-scriptlet-execution.ts` for integration testing.

## Hello World

A simple scriptlet that echoes a greeting.

```bash
echo "Hello from scriptlet!"
```

## Echo Input

A scriptlet with a named input placeholder.

<!-- description: Echoes user input back -->

```bash
echo "You said: {{message}}"
```

## Greet Person

A scriptlet with multiple inputs.

<!-- 
shortcut: cmd g
description: Greets a person by name
-->

```bash
echo "Hello, {{name}}! Welcome to {{place}}."
```

# Python Scriptlets

## Python Hello

A simple Python scriptlet.

<!-- description: Basic Python print test -->

```python
print("Hello from Python!")
```

## Python Variable

A Python scriptlet with variable substitution.

```python
name = "{{name}}"
print(f"Hello, {name}!")
```

## Python Exit Code

A Python scriptlet that returns a custom exit code.

<!-- description: Tests exit code propagation -->

```python
import sys
sys.exit({{exit_code}})
```

# Node.js Scriptlets

## Node Hello

A simple Node.js scriptlet.

<!-- description: Basic Node console.log test -->

```node
console.log("Hello from Node.js!");
```

## Node Variable

A Node.js scriptlet with variable substitution.

```js
const name = "{{name}}";
console.log(`Hello, ${name}!`);
```

## Node stderr

A Node.js scriptlet that writes to stderr.

```node
console.error("This goes to stderr");
console.log("This goes to stdout");
```

# TypeScript Snippets

```ts
// Common imports
import { log } from 'console';
```

## Log Message

A TypeScript scriptlet with prepended imports.

```ts
log("{{message}}");
```

## Calculate Sum

A TypeScript scriptlet demonstrating computation.

<!-- description: Adds two numbers -->

```ts
const a = parseInt("{{num1}}");
const b = parseInt("{{num2}}");
console.log(`Sum: ${a + b}`);
```

# Template Scriptlets

## Basic Template

A template that returns processed content without execution.

<!-- description: Returns substituted text -->

```template
Hello {{name}}! Welcome to {{place}}.
```

## Template with Conditionals

A template with conditional logic.

```template
{{#if formal}}Dear Sir/Madam,{{else}}Hey there!{{/if}} {{name}}
```

## Multi-line Template

A template with multiple lines and variables.

```template
Subject: Welcome, {{name}}!

Dear {{name}},

{{#if formal}}
We are pleased to inform you that your account has been created.
{{else}}
Your account is all set up and ready to go!
{{/if}}

Best regards,
The {{company}} Team
```

# Conditional Examples

## Conditional Output

A scriptlet demonstrating conditional blocks.

```bash
{{#if verbose}}echo "Running in verbose mode..."{{/if}}
echo "Result: {{result}}"
{{#if verbose}}echo "Done!"{{/if}}
```

## If-Else Example

A scriptlet with if-else conditional.

```bash
{{#if formal}}
echo "Dear {{name}},"
echo "We are pleased to inform you..."
{{else}}
echo "Hey {{name}}!"
echo "Just wanted to let you know..."
{{/if}}
```

# Positional Arguments

## Shell Args

A scriptlet using positional arguments.

```bash
echo "First arg: $1"
echo "Second arg: $2"
echo "All args: $@"
```

## Template with Args

A scriptlet combining named inputs and positional args.

<!-- description: Demonstrates mixed argument types -->

```bash
echo "Prefix: {{prefix}}"
echo "Args: $@"
```

# Exit Code Tests

## Exit Success

A scriptlet that exits with code 0.

```bash
exit 0
```

## Exit Failure

A scriptlet that exits with code 1.

```bash
exit 1
```

## Exit Custom

A scriptlet with a custom exit code.

```bash
exit {{code}}
```

# Multi-line Scripts

## Multi-line Bash

A multi-line bash script.

```bash
echo "Line 1"
echo "Line 2"
echo "Line 3"
```

## Bash with Logic

A bash script with control flow.

```bash
for i in 1 2 3; do
  echo "Number: $i"
done
```

# Error Cases (for testing error handling)

## Missing Command

A scriptlet referencing a non-existent command.

<!-- description: Tests error handling for missing commands -->

```bash
nonexistent_command_12345
```

## Syntax Error Python

A Python scriptlet with a syntax error.

<!-- description: Tests Python error capture -->

```python
print("unclosed string
```

# Utility Tools

## Open URL

A scriptlet to open a URL.

<!-- description: Opens a URL in default browser -->

```open
{{url}}
```

## Open File

A scriptlet to open a file.

```open
file://{{path}}
```

## Edit File

A scriptlet to edit a file.

<!-- description: Opens file in configured editor -->

```edit
{{file}}
```
