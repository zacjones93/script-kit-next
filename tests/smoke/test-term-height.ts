// Name: Test Terminal Height
// Description: Tests that terminal fills the full 700px window height

import '../../scripts/kit-sdk';

console.error('[SMOKE] test-term-height.ts starting...');

// Terminal should trigger a window resize to MAX_HEIGHT (700px)
// and the terminal content should fill the entire window
await term(`echo "=== Terminal Height Test ==="
echo ""
echo "This terminal should fill the full 700px window height."
echo "The window should resize from 500px to 700px when terminal opens."
echo ""
echo "You should see many rows available for terminal output."
echo "Press any key or Escape to exit."
echo ""
echo "Line 10"
echo "Line 11"
echo "Line 12"
echo "Line 13"
echo "Line 14"
echo "Line 15"
echo "Line 16"
echo "Line 17"
echo "Line 18"
echo "Line 19"
echo "Line 20"
echo "Line 21"
echo "Line 22"
echo "Line 23"
echo "Line 24"
echo "Line 25"
echo ""
echo "If you can see all lines without scrolling, terminal height is correct!"
read -n 1 -s
`);

console.error('[SMOKE] test-term-height.ts completed!');
