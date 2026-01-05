#!/bin/bash
# Install git hooks for Script Kit GPUI
#
# Usage: ./hooks/install.sh
#
# This installs hooks that prevent CI failures:
#   - pre-commit: cargo fmt --check (catches formatting before commit)
#   - pre-push: cargo fmt --check + cargo check (final safety net)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
GIT_HOOKS_DIR="$REPO_ROOT/.git/hooks"

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}Installing git hooks...${NC}"

# Install pre-commit hook
if [ -f "$GIT_HOOKS_DIR/pre-commit" ]; then
    echo -e "  Backing up existing pre-commit hook"
    mv "$GIT_HOOKS_DIR/pre-commit" "$GIT_HOOKS_DIR/pre-commit.backup"
fi

cp "$SCRIPT_DIR/pre-commit" "$GIT_HOOKS_DIR/pre-commit"
chmod +x "$GIT_HOOKS_DIR/pre-commit"
echo -e "  ${GREEN}✓${NC} pre-commit hook installed"

# Install pre-push hook
if [ -f "$GIT_HOOKS_DIR/pre-push" ]; then
    echo -e "  Backing up existing pre-push hook"
    mv "$GIT_HOOKS_DIR/pre-push" "$GIT_HOOKS_DIR/pre-push.backup"
fi

cp "$SCRIPT_DIR/pre-push" "$GIT_HOOKS_DIR/pre-push"
chmod +x "$GIT_HOOKS_DIR/pre-push"
echo -e "  ${GREEN}✓${NC} pre-push hook installed"

echo -e "\n${GREEN}✅ Git hooks installed!${NC}"
echo -e "\nHooks will now run automatically:"
echo -e "  pre-commit: cargo fmt --check"
echo -e "  pre-push:   cargo fmt --check + cargo check"
echo -e "\nSkip with --no-verify, full checks with FULL_CHECK=1 git push"
