// Name: Actions Focus Test
// Description: Verifies typing only goes to actions popup search when open

import '../../scripts/kit-sdk';

console.error('[TEST] Actions popup focus test starting...');
console.error('[TEST] SDK globals available:', typeof arg, typeof div, typeof md);

// Show instructions
await div(md(`# Actions Popup Focus Test

## Test Steps:
1. **Press Cmd+K** to open the actions popup
2. **Type "testing"** - text should appear ONLY in the popup search
3. **Verify main input is empty** (left side should have no text)
4. **Press Escape** to close popup
5. **Check main input** - should still be empty after closing

## Expected Behavior:
- When actions popup is open, keyboard input should go to the popup's search
- Main filter input should NOT receive text while popup is visible
- After closing, main input should remain empty

---

*Press Escape when done testing*`));

console.error('[TEST] Actions focus test completed');
