// Name: Notes Actions Panel Visual Test
// Description: Captures Notes actions panel (Cmd+K) for visual verification
//
// Run:
//   printf '{"type":"openNotes"}\n{"type":"run","path":"'$(pwd)'/tests/smoke/test-notes-actions-panel-visual.ts"}\n' \
//     | SCRIPT_KIT_TEST_NOTES_ACTIONS_PANEL=1 SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
//
// Expected:
//   - Actions panel opens automatically via SCRIPT_KIT_TEST_NOTES_ACTIONS_PANEL=1
//   - Search input at top with placeholder "Search for actions..."
//   - Raycast-style action list with keycaps and section dividers
//   - Move list item actions appear disabled

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

console.error('[TEST] Notes Actions Panel Visual Test');
console.error('[TEST] Waiting for actions panel to render...');

await new Promise(resolve => setTimeout(resolve, 1000));

const screenshot = await captureScreenshot();
console.error(`[TEST] Captured: ${screenshot.width}x${screenshot.height}`);

const dir = join(process.cwd(), '.test-screenshots');
mkdirSync(dir, { recursive: true });

const filepath = join(dir, `notes-actions-panel-${Date.now()}.png`);
writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
console.error(`[SCREENSHOT] ${filepath}`);

console.error('[TEST] Verify: search input at top, keycaps on right, separators between groups');
console.error('[TEST] Verify: window resized to fit actions panel');

process.exit(0);
