// HTML Rendering Test - Verifies that div() can render various HTML elements
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

const testHtml = `
<h1>HTML Rendering Test</h1>
<h2>Headers H2</h2>
<h3>Headers H3</h3>
<h4>Headers H4</h4>
<h5>Headers H5</h5>
<h6>Headers H6</h6>

<p>This is a paragraph with <strong>bold text</strong> and <em>italic text</em>.</p>

<p>Here is some <code>inline code</code> in a paragraph.</p>

<pre><code class="language-typescript">const greeting = "Hello World";
console.log(greeting);</code></pre>

<ul>
  <li>Unordered list item 1</li>
  <li>Unordered list item 2</li>
  <li>Unordered list item 3</li>
</ul>

<ol>
  <li>Ordered list item 1</li>
  <li>Ordered list item 2</li>
  <li>Ordered list item 3</li>
</ol>

<blockquote>This is a blockquote with some wisdom.</blockquote>

<hr>

<p>Links: <a href="https://example.com">Example Link</a> (styled but not clickable)</p>

<p>Line<br>Break</p>
`;

console.error('[SMOKE] Starting HTML rendering test...');

// Show the div with HTML content
const divPromise = div(testHtml);

// Wait for render
await new Promise(resolve => setTimeout(resolve, 1500));

// Capture screenshot
const screenshot = await captureScreenshot();
console.error(`[SMOKE] Screenshot captured: ${screenshot.width}x${screenshot.height}`);

// Save to test-screenshots
const screenshotDir = join(process.cwd(), 'test-screenshots');
mkdirSync(screenshotDir, { recursive: true });

const filename = `html-rendering-${Date.now()}.png`;
const filepath = join(screenshotDir, filename);
writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));

console.error(`[SCREENSHOT] Saved to: ${filepath}`);
console.error('[SMOKE] HTML rendering test complete');

process.exit(0);
