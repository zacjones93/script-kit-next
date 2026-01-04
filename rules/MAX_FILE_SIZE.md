# File Size Guidelines for Rust Code

## Thresholds

| Threshold | Action |
|-----------|--------|
| **500 lines** | Target for production code |
| **1000 lines** | Warning threshold - consider splitting |
| **2000 lines** | Critical - must split before adding more |

## Why This Matters

Large files are harder to:
- Navigate and understand
- Test in isolation
- Review in PRs
- Parallelize work across team members/agents

## How to Split a Large File

### 1. Analyze the file structure

Use `rs-hack` to understand the file's contents:

```bash
# Install rs-hack if not present
cargo install rs-hack

# Analyze a file's structure
rs-hack find --paths src/my_large_file.rs --kind function --format json | jq '.[] | .name'

# See all items (functions, structs, enums, etc.)
rs-hack find --paths src/my_large_file.rs --format json | jq -r '.[] | "\(.kind): \(.name)"'
```

### 2. Identify logical boundaries

Look for:
- `// ===` section markers in the code
- Related functions that operate on the same data
- Types and their associated methods
- Test-only code that can be `#[cfg(test)]` gated

### 3. Create a module directory

```bash
# Create the module directory
mkdir src/my_module

# Create mod.rs with re-exports
cat > src/my_module/mod.rs << 'EOF'
//! Module description here

mod submodule_a;
mod submodule_b;

// Re-export public API for backwards compatibility
pub use submodule_a::{FunctionA, TypeA};
pub use submodule_b::{FunctionB, TypeB};

// Test-only exports
#[cfg(test)]
pub(crate) use submodule_a::internal_helper;

#[cfg(test)]
#[path = "../my_module_tests.rs"]
mod tests;
EOF
```

### 4. Move code to submodules

Create each submodule file with the appropriate code:

```rust
// src/my_module/submodule_a.rs
//! Description of this submodule

use super::*;  // Import from parent module if needed

pub struct TypeA { ... }

pub fn function_a() { ... }

pub(crate) fn internal_helper() { ... }
```

### 5. Update lib.rs

Change:
```rust
mod my_large_file;
```

To:
```rust
mod my_module;
```

### 6. Verify

```bash
# Must all pass before committing
cargo check && cargo clippy --all-targets -- -D warnings && cargo test
```

## Pattern: Test-Only Re-exports

When tests use `super::*`, they need access to types. Use `#[cfg(test)]`:

```rust
// In mod.rs

// Always exported (used by production code)
pub use types::{Script, Scriptlet, SearchResult};

// Only exported for tests
#[cfg(test)]
pub use types::{ScriptMatch, MatchIndices};

#[cfg(test)]
pub use crate::external_module::ExternalType;
```

## Example Splits in This Codebase

| Original | Lines | Split Into |
|----------|-------|------------|
| `src/utils.rs` | 1,969 | `src/utils/` (html, assets, paths, tailwind) |
| `src/executor.rs` | 2,264 | `src/executor/` (auto_submit, errors, runner, scriptlet, selected_text) |
| `src/scripts.rs` | 2,036 | `src/scripts/` (types, metadata, loader, scriptlet_loader, search, grouping, scheduling) |

## Quick Reference Commands

```bash
# Check file sizes
find src -name "*.rs" -exec wc -l {} \; | sort -n | tail -20

# Find files over 1000 lines
find src -name "*.rs" -exec wc -l {} \; | awk '$1 > 1000 {print}'

# Analyze a specific file
rs-hack find --paths src/my_file.rs --format json | jq 'group_by(.kind) | map({kind: .[0].kind, count: length})'
```

## Asking for Help

If you're an AI agent and unsure how to split a file:

1. Check `./audits/FILE_SIZE.md` for detailed analysis and recommendations
2. Look at existing module directories (`src/utils/`, `src/executor/`, `src/scripts/`) for patterns
3. Use the section markers (`// ===`) as natural split points
