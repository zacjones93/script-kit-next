// Visual Regression Test - Text and Header Rendering
// Tests basic text display and all markdown header levels via div(md(...))
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

const RENDER_DELAY = 500; // ms to wait for render

interface TestScenario {
  name: string;
  filename: string;
  content: string;
}

const scenarios: TestScenario[] = [
  {
    name: 'Plain text without markdown',
    filename: 'div-md-plain-text.png',
    content: 'This is plain text without any markdown formatting. It should render as a simple paragraph.'
  },
  {
    name: 'Empty string',
    filename: 'div-md-empty.png',
    content: ''
  },
  {
    name: 'Long line (200+ chars)',
    filename: 'div-md-long-line.png',
    content: 'This is a very long line of text that exceeds 200 characters to test how the rendering engine handles text wrapping and overflow. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam.'
  },
  {
    name: 'Multiple paragraphs',
    filename: 'div-md-multi-paragraph.png',
    content: `First paragraph with some text content.

Second paragraph separated by a blank line.

Third paragraph to verify multiple paragraph spacing works correctly.`
  },
  {
    name: 'H1 Header',
    filename: 'div-md-h1.png',
    content: '# Heading Level 1'
  },
  {
    name: 'H2 Header',
    filename: 'div-md-h2.png',
    content: '## Heading Level 2'
  },
  {
    name: 'H3 Header',
    filename: 'div-md-h3.png',
    content: '### Heading Level 3'
  },
  {
    name: 'H4, H5, H6 Headers together',
    filename: 'div-md-h4-h6.png',
    content: `#### Heading Level 4

##### Heading Level 5

###### Heading Level 6`
  },
  {
    name: 'Headers H1-H6 in sequence',
    filename: 'div-md-headers-sequence.png',
    content: `# Heading 1

## Heading 2

### Heading 3

#### Heading 4

##### Heading 5

###### Heading 6`
  },
  {
    name: 'Header with bold formatting',
    filename: 'div-md-header-with-formatting.png',
    content: `# Header with **bold** text

## Another **bold** header

### Mixed **bold** and *italic* in header`
  }
];

async function runTest(scenario: TestScenario, screenshotDir: string): Promise<boolean> {
  console.error(`[TEST] Running: ${scenario.name}`);
  
  try {
    // Show content via div with md()
    const htmlContent = md(scenario.content);
    await div(htmlContent);
    
    // Wait for render
    await new Promise(resolve => setTimeout(resolve, RENDER_DELAY));
    
    // Capture screenshot using SDK function
    const screenshot = await captureScreenshot();
    console.error(`[TEST] Captured: ${screenshot.width}x${screenshot.height}`);
    
    // Save screenshot
    const filepath = join(screenshotDir, scenario.filename);
    writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
    console.error(`[SCREENSHOT] Saved: ${filepath}`);
    
    return true;
  } catch (error) {
    console.error(`[TEST] FAILED: ${scenario.name} - ${error}`);
    return false;
  }
}

async function main() {
  console.error('[SMOKE] Starting text and headers visual regression tests...');
  console.error(`[SMOKE] Total scenarios: ${scenarios.length}`);
  
  // Ensure screenshot directory exists
  const screenshotDir = join(process.cwd(), 'test-screenshots');
  mkdirSync(screenshotDir, { recursive: true });
  console.error(`[SMOKE] Screenshot dir: ${screenshotDir}`);
  
  let passed = 0;
  let failed = 0;
  
  for (const scenario of scenarios) {
    const success = await runTest(scenario, screenshotDir);
    if (success) {
      passed++;
    } else {
      failed++;
    }
  }
  
  console.error(`[SMOKE] Results: ${passed} passed, ${failed} failed`);
  console.error('[SMOKE] Text and headers visual regression tests complete');
  
  process.exit(failed > 0 ? 1 : 0);
}

main().catch(err => {
  console.error(`[SMOKE] Fatal error: ${err}`);
  process.exit(1);
});
