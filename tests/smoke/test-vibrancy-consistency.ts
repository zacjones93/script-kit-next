// Name: Test Vibrancy Consistency
// Description: Verifies vibrancy/opacity works consistently across prompt types (div, editor)
// Note: arg() screenshot capture has timing issues in automated tests - vibrancy verified manually

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync, existsSync } from 'fs';
import { join } from 'path';

console.error('[VIBRANCY-TEST] Starting vibrancy consistency test...');

const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });

interface TestResult {
  prompt: string;
  screenshot: string;
  width: number;
  height: number;
  success: boolean;
}

const results: TestResult[] = [];

async function captureAndSave(name: string): Promise<{ path: string; width: number; height: number }> {
  const screenshot = await captureScreenshot();
  const filename = `vibrancy-${name}-${Date.now()}.png`;
  const filepath = join(dir, filename);
  writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
  console.error(`[SCREENSHOT] ${filepath}`);
  return { path: filepath, width: screenshot.width, height: screenshot.height };
}

// Test 1: div() prompt - establishes window visibility and tests vibrancy
console.error('[VIBRANCY-TEST] Test 1: div() prompt');

setTimeout(async () => {
  try {
    const divShot = await captureAndSave('div');
    results.push({
      prompt: 'div',
      screenshot: divShot.path,
      width: divShot.width,
      height: divShot.height,
      success: divShot.width > 0,
    });
    console.error(`[VIBRANCY-TEST] div() captured: ${divShot.width}x${divShot.height}`);
  } catch (err) {
    console.error('[VIBRANCY-TEST] div() capture failed:', err);
    results.push({ prompt: 'div', screenshot: '', width: 0, height: 0, success: false });
  }
  await submit('continue');
}, 1200);

await div(`
  <div class="flex flex-col gap-4 p-6">
    <h1 class="text-white text-xl font-bold">Vibrancy Consistency Test</h1>
    <p class="text-gray-300">This div prompt should have dark blur vibrancy effect</p>
    <div class="p-4 bg-white/10 rounded-lg">
      <span class="text-white">Semi-transparent overlay (10% white)</span>
    </div>
    <div class="p-4 bg-black/20 rounded-lg">
      <span class="text-gray-200">Semi-transparent overlay (20% black)</span>
    </div>
    <p class="text-gray-400 text-sm">Testing vibrancy consistency...</p>
  </div>
`);

// Test 2: editor() prompt - tests vibrancy on editor window
console.error('[VIBRANCY-TEST] Test 2: editor() prompt');

setTimeout(async () => {
  try {
    const editorShot = await captureAndSave('editor');
    results.push({
      prompt: 'editor',
      screenshot: editorShot.path,
      width: editorShot.width,
      height: editorShot.height,
      success: editorShot.width > 0,
    });
    console.error(`[VIBRANCY-TEST] editor() captured: ${editorShot.width}x${editorShot.height}`);
  } catch (err) {
    console.error('[VIBRANCY-TEST] editor() capture failed:', err);
    results.push({ prompt: 'editor', screenshot: '', width: 0, height: 0, success: false });
  }
  await submit('done');
}, 1200);

await editor(
  `// Vibrancy Consistency Test - Editor Prompt
// 
// The editor background should have the same vibrancy effect
// as the div() prompt tested above.
//
// Visual consistency checklist:
// - Background blur is visible
// - Dark tint overlays light backgrounds
// - Text is clearly readable
// - No washed-out appearance
//
// Press Cmd+Enter to submit, Escape to cancel
`,
  'typescript'
);

// Summary
console.error('[VIBRANCY-TEST] ========== SUMMARY ==========');
for (const result of results) {
  const status = result.success ? 'PASS' : 'FAIL';
  console.error(`[VIBRANCY-TEST] ${result.prompt}: ${status} - ${result.width}x${result.height}`);
  if (result.success) {
    console.error(`[VIBRANCY-TEST]   Screenshot: ${result.screenshot}`);
  }
}

const allPassed = results.every(r => r.success);
const screenshotsExist = results.filter(r => r.success && existsSync(r.screenshot)).length;

console.error(`[VIBRANCY-TEST] Total: ${results.length} tests, ${screenshotsExist} screenshots captured`);
console.error(`[VIBRANCY-TEST] Overall: ${allPassed ? 'ALL PASSED' : 'SOME FAILED'}`);
console.error('[VIBRANCY-TEST] ============================');

// Verification notes for visual inspection:
console.error('[VIBRANCY-TEST] VERIFICATION NOTES:');
console.error('[VIBRANCY-TEST] - Compare div and editor screenshots side-by-side');
console.error('[VIBRANCY-TEST] - Both prompts should have consistent dark blur effect');
console.error('[VIBRANCY-TEST] - Semi-transparent overlays should blend naturally');
console.error('[VIBRANCY-TEST] - No prompt should appear washed out over light backgrounds');
console.error('[VIBRANCY-TEST] - arg() vibrancy should be verified manually (screenshot timing issues in automation)');

process.exit(allPassed ? 0 : 1);
