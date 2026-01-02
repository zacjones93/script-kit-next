// Name: Visual Audit
// Description: Non-interactive visual tests - captures screenshots of UI states

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync, existsSync } from 'fs';
import { join } from 'path';

export const metadata = {
  name: "Visual Audit",
  description: "Automated visual regression testing for arg() and div() prompts",
};

const screenshotDir = join(process.cwd(), '.test-screenshots', 'visual-audit');
mkdirSync(screenshotDir, { recursive: true });

interface TestResult {
  name: string;
  path: string;
  width: number;
  height: number;
  timestamp: string;
}

const results: TestResult[] = [];

async function captureTest(name: string): Promise<TestResult> {
  // Wait for render to complete
  await new Promise(r => setTimeout(r, 600));
  
  const ss = await captureScreenshot();
  const filename = `${name}.png`;
  const filepath = join(screenshotDir, filename);
  writeFileSync(filepath, Buffer.from(ss.data, 'base64'));
  
  const result: TestResult = {
    name,
    path: filepath,
    width: ss.width,
    height: ss.height,
    timestamp: new Date().toISOString(),
  };
  
  results.push(result);
  console.error(`[CAPTURED] ${name}: ${ss.width}x${ss.height} -> ${filepath}`);
  return result;
}

// ============================================================
// TEST SUITE: div() prompts (non-interactive, auto-progress)
// ============================================================

console.error('[SUITE] Starting div() visual tests');

// Test 1: Basic HTML
console.error('[TEST] 01-div-basic-html');
await div(`
  <div style="padding: 24px; font-family: system-ui;">
    <h1 style="color: #3b82f6; margin: 0 0 12px 0; font-size: 24px;">Basic HTML Test</h1>
    <p style="color: #9ca3af; margin: 0 0 16px 0;">Testing inline styles and basic HTML elements</p>
    <ul style="margin: 0; padding-left: 20px; color: #d1d5db;">
      <li style="margin-bottom: 4px;">First list item</li>
      <li style="margin-bottom: 4px;">Second list item</li>
      <li style="margin-bottom: 4px;">Third list item</li>
    </ul>
    <div style="margin-top: 16px; padding: 12px; background: rgba(59,130,246,0.1); border-radius: 8px;">
      <code style="color: #60a5fa;">console.log("Hello World")</code>
    </div>
  </div>
`);
await captureTest('01-div-basic-html');

// Test 2: Tailwind CSS
console.error('[TEST] 02-div-tailwind');
await div({
  html: `
    <div class="text-center">
      <h1 class="text-2xl font-bold text-blue-400 mb-2">Tailwind CSS Test</h1>
      <p class="text-gray-400 mb-4">Using Tailwind utility classes for styling</p>
      <div class="flex gap-2 justify-center flex-wrap">
        <span class="px-3 py-1 bg-green-500/20 text-green-400 rounded-full text-sm">Success</span>
        <span class="px-3 py-1 bg-blue-500/20 text-blue-400 rounded-full text-sm">Info</span>
        <span class="px-3 py-1 bg-yellow-500/20 text-yellow-400 rounded-full text-sm">Warning</span>
        <span class="px-3 py-1 bg-red-500/20 text-red-400 rounded-full text-sm">Error</span>
      </div>
      <div class="mt-4 p-4 bg-gray-800/50 rounded-lg text-left">
        <p class="text-gray-300 text-sm font-mono">code block with bg-gray-800/50</p>
      </div>
    </div>
  `,
  containerClasses: 'p-6'
});
await captureTest('02-div-tailwind');

// Test 3: Markdown rendering
console.error('[TEST] 03-div-markdown');
await div(md(`
# Markdown Rendering Test

This tests **bold text**, *italic text*, and \`inline code\`.

## List Items
- First bullet point
- Second bullet point  
- Third bullet point

## Code Block
\`\`\`typescript
function greet(name: string): string {
  return \`Hello, \${name}!\`;
}
\`\`\`

> This is a blockquote to test styling

---

Footer text with [a link](https://example.com)
`));
await captureTest('03-div-markdown');

// Test 4: Unicode and emoji
console.error('[TEST] 04-div-unicode');
await div(`
  <div style="padding: 20px; font-family: system-ui;">
    <h2 style="color: #f59e0b; margin-bottom: 16px;">🌍 Unicode Support Test</h2>
    <div style="display: grid; gap: 8px; color: #e5e7eb;">
      <div>🇺🇸 English: Hello World</div>
      <div>🇯🇵 日本語: こんにちは世界</div>
      <div>🇨🇳 中文: 你好世界</div>
      <div>🇸🇦 العربية: مرحبا بالعالم</div>
      <div>🇬🇷 Ελληνικά: Γειά σου κόσμε</div>
      <div>🇷🇺 Русский: Привет мир</div>
    </div>
    <div style="margin-top: 16px; font-size: 24px;">
      Emoji test: 🎉 🚀 ✅ ❌ ⚡ 🔥 💡 🎨
    </div>
  </div>
`);
await captureTest('04-div-unicode');

// Test 5: Complex layout
console.error('[TEST] 05-div-complex-layout');
await div({
  html: `
    <div class="grid grid-cols-2 gap-4">
      <div class="p-4 bg-blue-500/10 rounded-lg border border-blue-500/30">
        <h3 class="text-blue-400 font-semibold mb-2">Card 1</h3>
        <p class="text-gray-400 text-sm">Left column content with some description text.</p>
      </div>
      <div class="p-4 bg-purple-500/10 rounded-lg border border-purple-500/30">
        <h3 class="text-purple-400 font-semibold mb-2">Card 2</h3>
        <p class="text-gray-400 text-sm">Right column content with some description text.</p>
      </div>
      <div class="col-span-2 p-4 bg-green-500/10 rounded-lg border border-green-500/30">
        <h3 class="text-green-400 font-semibold mb-2">Full Width Card</h3>
        <p class="text-gray-400 text-sm">This card spans both columns using col-span-2.</p>
      </div>
    </div>
  `,
  containerClasses: 'p-4'
});
await captureTest('05-div-complex-layout');

// Test 6: Table rendering
console.error('[TEST] 06-div-table');
await div(`
  <div style="padding: 16px;">
    <h3 style="color: #a78bfa; margin-bottom: 12px;">Data Table Test</h3>
    <table style="width: 100%; border-collapse: collapse; font-size: 14px;">
      <thead>
        <tr style="border-bottom: 1px solid #374151;">
          <th style="text-align: left; padding: 8px; color: #9ca3af;">Name</th>
          <th style="text-align: left; padding: 8px; color: #9ca3af;">Type</th>
          <th style="text-align: right; padding: 8px; color: #9ca3af;">Size</th>
        </tr>
      </thead>
      <tbody style="color: #e5e7eb;">
        <tr style="border-bottom: 1px solid #1f2937;">
          <td style="padding: 8px;">main.rs</td>
          <td style="padding: 8px;">Rust</td>
          <td style="text-align: right; padding: 8px;">12.4 KB</td>
        </tr>
        <tr style="border-bottom: 1px solid #1f2937;">
          <td style="padding: 8px;">kit-sdk.ts</td>
          <td style="padding: 8px;">TypeScript</td>
          <td style="text-align: right; padding: 8px;">8.2 KB</td>
        </tr>
        <tr>
          <td style="padding: 8px;">theme.json</td>
          <td style="padding: 8px;">JSON</td>
          <td style="text-align: right; padding: 8px;">2.1 KB</td>
        </tr>
      </tbody>
    </table>
  </div>
`);
await captureTest('06-div-table');

// ============================================================
// TEST SUITE: arg() preview (show what arg looks like via div)
// ============================================================

console.error('[SUITE] Starting arg() preview tests');

// Test 7: arg with string choices - preview
console.error('[TEST] 07-arg-preview-strings');
await div({
  html: `
    <div class="space-y-1">
      <div class="text-gray-400 text-sm mb-3">Select a fruit:</div>
      <div class="p-2 bg-blue-500/20 rounded text-blue-300 cursor-pointer">🍎 Apple</div>
      <div class="p-2 hover:bg-gray-700/50 rounded text-gray-300">🍌 Banana</div>
      <div class="p-2 hover:bg-gray-700/50 rounded text-gray-300">🍒 Cherry</div>
      <div class="p-2 hover:bg-gray-700/50 rounded text-gray-300">🍇 Date</div>
      <div class="p-2 hover:bg-gray-700/50 rounded text-gray-300">🫐 Elderberry</div>
    </div>
  `,
  containerClasses: 'p-3'
});
await captureTest('07-arg-preview-strings');

// Test 8: arg with structured choices - preview
console.error('[TEST] 08-arg-preview-structured');
await div({
  html: `
    <div class="space-y-2">
      <div class="text-gray-400 text-sm mb-3">Select an action:</div>
      <div class="p-3 bg-blue-500/20 rounded-lg border border-blue-500/30">
        <div class="text-blue-300 font-medium">▶ Run Script</div>
        <div class="text-gray-500 text-sm mt-1">Execute the current script</div>
      </div>
      <div class="p-3 hover:bg-gray-700/30 rounded-lg border border-transparent">
        <div class="text-gray-300 font-medium">✏️ Edit Script</div>
        <div class="text-gray-500 text-sm mt-1">Open script in editor</div>
      </div>
      <div class="p-3 hover:bg-gray-700/30 rounded-lg border border-transparent">
        <div class="text-gray-300 font-medium">🗑️ Delete Script</div>
        <div class="text-gray-500 text-sm mt-1">Remove script from disk</div>
      </div>
      <div class="p-3 hover:bg-gray-700/30 rounded-lg border border-transparent">
        <div class="text-gray-300 font-medium">🔗 Share Script</div>
        <div class="text-gray-500 text-sm mt-1">Copy shareable link</div>
      </div>
    </div>
  `,
  containerClasses: 'p-3'
});
await captureTest('08-arg-preview-structured');

// Test 9: Large list preview
console.error('[TEST] 09-arg-preview-large');
const largeListHtml = Array.from({ length: 15 }, (_, i) => 
  `<div class="${i === 0 ? 'bg-blue-500/20 text-blue-300' : 'text-gray-300'} p-2 rounded">${String(i + 1).padStart(2, '0')}. List Item ${i + 1}</div>`
).join('');
await div({
  html: `
    <div class="space-y-1">
      <div class="text-gray-400 text-sm mb-3">Large list (showing 15 of 100):</div>
      ${largeListHtml}
      <div class="text-gray-500 text-sm text-center mt-2">... and 85 more items</div>
    </div>
  `,
  containerClasses: 'p-3'
});
await captureTest('09-arg-preview-large');

// ============================================================
// SUMMARY
// ============================================================

console.error('[SUITE] All visual tests complete');

// Final summary screen
await div(md(`
# ✅ Visual Audit Complete

**${results.length} screenshots captured** to:
\`${screenshotDir}\`

## Captured Tests:
${results.map((r, i) => `${i + 1}. **${r.name}** (${r.width}x${r.height})`).join('\n')}

## Next Steps:
1. Review screenshots in \`.test-screenshots/visual-audit/\`
2. Compare against baseline images
3. Update baselines if changes are intentional

*Press Enter to exit*
`));
await captureTest('10-summary');

// Output JSON summary for parsing
console.log(JSON.stringify({ 
  suite: 'visual-audit',
  status: 'complete',
  count: results.length,
  results,
  outputDir: screenshotDir
}, null, 2));

process.exit(0);
