// Name: SDK Test - div()
// Description: Tests div() HTML content display

/**
 * SDK TEST: test-div.ts
 * 
 * Tests the div() function which displays HTML content to the user.
 * 
 * Test cases:
 * 1. div-html-content: Basic HTML rendering
 * 2. div-with-tailwind: HTML with Tailwind class hints
 * 3. div-complex-html: Complex nested HTML structure
 * 
 * Expected behavior:
 * - div() sends JSONL message with type: 'div'
 * - HTML is rendered in the GPUI panel
 * - Promise resolves when user dismisses
 */

import '../../scripts/kit-sdk';

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
  console.error(`[TEST] ${msg}`);
}

// =============================================================================
// Tests
// =============================================================================

debug('test-div.ts starting...');
debug(`SDK globals: arg=${typeof arg}, div=${typeof div}, md=${typeof md}`);

// -----------------------------------------------------------------------------
// Test 1: Basic HTML content
// -----------------------------------------------------------------------------
const test1 = 'div-html-content';
logTest(test1, 'running');
const start1 = Date.now();

try {
  debug('Test 1: div() with basic HTML');
  
  await div(`
    <div style="padding: 20px; font-family: system-ui;">
      <h1 style="color: #2563eb; margin-bottom: 16px;">div() Test - Basic HTML</h1>
      <p style="color: #374151; line-height: 1.6;">
        This tests that <code>div()</code> can render basic HTML content.
      </p>
      <ul style="margin-top: 12px; color: #4b5563;">
        <li>Headings render correctly</li>
        <li>Paragraphs have proper spacing</li>
        <li>Lists display as expected</li>
        <li>Inline styles are applied</li>
      </ul>
      <p style="margin-top: 16px; color: #6b7280; font-size: 14px;">
        <em>Click anywhere or press Escape to continue to next test...</em>
      </p>
    </div>
  `);
  
  debug('Test 1 completed - user dismissed div');
  logTest(test1, 'pass', { result: 'dismissed', duration_ms: Date.now() - start1 });
} catch (err) {
  logTest(test1, 'fail', { error: String(err), duration_ms: Date.now() - start1 });
}

// -----------------------------------------------------------------------------
// Test 2: HTML with Tailwind class hints
// -----------------------------------------------------------------------------
const test2 = 'div-with-tailwind';
logTest(test2, 'running');
const start2 = Date.now();

try {
  debug('Test 2: div() with containerClasses (DivConfig API)');
  
  await div({
    html: `
      <div class="content">
        <h1>div() Test - Tailwind Support</h1>
        <p>This tests containerClasses for styling hints.</p>
        <div class="info-box">
          <strong>Note:</strong> Tailwind classes provide styling hints to the renderer.
        </div>
      </div>
    `,
    containerClasses: 'p-6 bg-gray-50'
  });
  
  debug('Test 2 completed - user dismissed div');
  logTest(test2, 'pass', { result: 'dismissed', duration_ms: Date.now() - start2 });
} catch (err) {
  logTest(test2, 'fail', { error: String(err), duration_ms: Date.now() - start2 });
}

// -----------------------------------------------------------------------------
// Test 3: Complex nested HTML
// -----------------------------------------------------------------------------
const test3 = 'div-complex-html';
logTest(test3, 'running');
const start3 = Date.now();

try {
  debug('Test 3: div() with complex nested HTML');
  
  const tableHtml = `
    <div style="padding: 20px; font-family: system-ui;">
      <h2 style="color: #1f2937; margin-bottom: 16px;">Complex HTML Test</h2>
      
      <table style="width: 100%; border-collapse: collapse; margin-bottom: 16px;">
        <thead>
          <tr style="background: #f3f4f6;">
            <th style="padding: 8px; text-align: left; border-bottom: 2px solid #e5e7eb;">Feature</th>
            <th style="padding: 8px; text-align: left; border-bottom: 2px solid #e5e7eb;">Status</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td style="padding: 8px; border-bottom: 1px solid #e5e7eb;">Table rendering</td>
            <td style="padding: 8px; border-bottom: 1px solid #e5e7eb;">Testing</td>
          </tr>
          <tr>
            <td style="padding: 8px; border-bottom: 1px solid #e5e7eb;">Nested elements</td>
            <td style="padding: 8px; border-bottom: 1px solid #e5e7eb;">Testing</td>
          </tr>
          <tr>
            <td style="padding: 8px; border-bottom: 1px solid #e5e7eb;">Inline styles</td>
            <td style="padding: 8px; border-bottom: 1px solid #e5e7eb;">Testing</td>
          </tr>
        </tbody>
      </table>
      
      <div style="display: flex; gap: 8px;">
        <span style="padding: 4px 8px; background: #dbeafe; color: #1d4ed8; border-radius: 4px; font-size: 12px;">Tag 1</span>
        <span style="padding: 4px 8px; background: #dcfce7; color: #166534; border-radius: 4px; font-size: 12px;">Tag 2</span>
        <span style="padding: 4px 8px; background: #fef3c7; color: #92400e; border-radius: 4px; font-size: 12px;">Tag 3</span>
      </div>
      
      <p style="margin-top: 16px; color: #6b7280; font-size: 14px;">
        <em>All div() tests complete. Press Escape to exit.</em>
      </p>
    </div>
  `;
  
  await div(tableHtml);
  
  debug('Test 3 completed - user dismissed div');
  logTest(test3, 'pass', { result: 'dismissed', duration_ms: Date.now() - start3 });
} catch (err) {
  logTest(test3, 'fail', { error: String(err), duration_ms: Date.now() - start3 });
}

// -----------------------------------------------------------------------------
// Summary
// -----------------------------------------------------------------------------
debug('test-div.ts completed!');
debug('All div() tests executed successfully');
