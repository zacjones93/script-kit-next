// Name: Test Auto Submit
// Description: Tests that window hides after script completes

import '../../scripts/kit-sdk';

console.error('[TEST] Script starting');

// Auto-submit after 500ms
setTimeout(() => {
  console.error('[TEST] Calling submit("auto-value")');
  submit("auto-value");
}, 500);

const result = await arg("Waiting for auto-submit...");
console.error('[TEST] arg returned:', result);
console.error('[TEST] Script ending - window should hide now');
