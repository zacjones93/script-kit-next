# Test Scriptlets

Sample scriptlets for smoke testing the scriptlet parsing and execution system.

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
