#!/bin/bash
# Install git hooks for Script Kit GPUI
#
# Usage: ./hooks/install.sh
#
# This installs the pre-push hook that prevents CI failures by checking
# formatting and compilation before pushing.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
GIT_HOOKS_DIR="$REPO_ROOT/.git/hooks"

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}Installing git hooks...${NC}"

# Install pre-push hook
if [ -f "$GIT_HOOKS_DIR/pre-push" ]; then
    echo -e "  Backing up existing pre-push hook to pre-push.backup"
    mv "$GIT_HOOKS_DIR/pre-push" "$GIT_HOOKS_DIR/pre-push.backup"
fi

cp "$SCRIPT_DIR/pre-push" "$GIT_HOOKS_DIR/pre-push"
chmod +x "$GIT_HOOKS_DIR/pre-push"
echo -e "  ${GREEN}✓${NC} pre-push hook installed"

echo -e "\n${GREEN}✅ Git hooks installed!${NC}"
echo -e "\nThe pre-push hook will now run before each push:"
echo -e "  • Fast mode (default): cargo fmt --check + cargo check"
echo -e "  • Full mode: FULL_CHECK=1 git push"
echo -e "  • Skip: git push --no-verify"
