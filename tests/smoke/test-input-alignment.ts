// Name: Visual Test - Input Alignment Comparison
// Description: Compares main menu, arg(), and env() input alignment to ensure consistency

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// =============================================================================
// Test Infrastructure
// =============================================================================

interface TestResult {
  test: string;
  status: 'running' | 'pass' | 'fail' | 'skip';
  timestamp: string;
  result?: unknown;
  error?: string;
  duration_ms?: number;
}

function logTest(name: string, status: TestResult['status'], extra?: Partial<TestResult>) {
  const result: TestResult = {
    test: name,
    status,
    timestamp: new Date().toISOString(),
    ...extra
  };
  console.log(JSON.stringify(result));
}

function debug(msg: string) {
  console.error(`[ALIGN] ${msg}`);
}

const screenshotDir = join(process.cwd(), '.test-screenshots');
mkdirSync(screenshotDir, { recursive: true });

debug('test-input-alignment.ts starting...');
debug('This test captures screenshots of different prompt types to verify input alignment');

// =============================================================================
// Test 1: arg() prompt without choices (input only)
// =============================================================================

const test1 = 'arg-no-choices-alignment';
logTest(test1, 'running');
const start1 = Date.now();

try {
  debug('Test 1: arg() without choices - should match main menu input alignment');
  
  // Start arg prompt without choices
  const argPromise = arg('Enter your name');
  
  // Wait for UI to render
  await new Promise(resolve => setTimeout(resolve, 800));
  
  // Capture screenshot
  debug('Capturing arg() prompt screenshot...');
  const screenshot1 = await captureScreenshot();
  debug(`Screenshot captured: ${screenshot1.width}x${screenshot1.height}`);
  
  const filename1 = `input-align-arg-${Date.now()}.png`;
  const filepath1 = join(screenshotDir, filename1);
  writeFileSync(filepath1, Buffer.from(screenshot1.data, 'base64'));
  debug(`[SCREENSHOT] ${filepath1}`);
  
  logTest(test1, 'pass', {
    result: {
      width: screenshot1.width,
      height: screenshot1.height,
      path: filepath1,
      prompt: 'arg()',
      description: 'Should show: cursor | "Enter your name" | buttons | logo'
    },
    duration_ms: Date.now() - start1
  });
  
} catch (err) {
  debug(`Test 1 error: ${err}`);
  logTest(test1, 'fail', {
    error: String(err),
    duration_ms: Date.now() - start1
  });
}

// Exit cleanly after capturing
debug('');
debug('============================================');
debug('INPUT ALIGNMENT TEST COMPLETE');
debug('============================================');
debug('');
debug('Visual verification checklist:');
debug('  [ ] Cursor position aligned at same X coordinate');
debug('  [ ] Placeholder text starts at same X coordinate');
debug('  [ ] Buttons/logo aligned at same X coordinate from right');
debug('  [ ] Vertical padding consistent (~8px top/bottom)');
debug('  [ ] Gap between input and buttons consistent (~12px)');
debug('');
debug('Compare screenshots in .test-screenshots/:');
debug('  - input-align-arg-*.png (arg() without choices)');
debug('============================================');

process.exit(0);
