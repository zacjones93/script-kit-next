// Visual Regression Test - Links, Images, and Special Elements
// Tests markdown links, images, horizontal rules, and line breaks via div(md(...))
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
    name: 'Basic link',
    filename: 'div-md-link.png',
    content: 'Click [this link](https://example.com) to visit the site.'
  },
  {
    name: 'Multiple links',
    filename: 'div-md-link-multiple.png',
    content: `Here are some useful links:

- [GitHub](https://github.com) - Code hosting
- [Google](https://google.com) - Search engine
- [Script Kit](https://scriptkit.com) - Automation tool

Visit any of these sites to learn more.`
  },
  {
    name: 'Image',
    filename: 'div-md-image.png',
    content: '![Script Kit Logo](https://scriptkit.com/logo.png)'
  },
  {
    name: 'Horizontal rule with dashes',
    filename: 'div-md-hr-dashes.png',
    content: `Content above the rule

---

Content below the rule`
  },
  {
    name: 'Horizontal rule with asterisks',
    filename: 'div-md-hr-asterisks.png',
    content: `Section one content

***

Section two content`
  },
  {
    name: 'Line breaks',
    filename: 'div-md-br.png',
    content: `First line  
Second line (with double space break)

Third line (new paragraph)  
Fourth line  
Fifth line`
  },
  {
    name: 'Mixed special elements',
    filename: 'div-md-mixed-special.png',
    content: `# Welcome to Script Kit

Check out our [documentation](https://docs.scriptkit.com) for help.

---

## Features

![Feature Icon](https://example.com/icon.png)

Visit [GitHub](https://github.com/johnlindquist/kit) for source code.

***

### Quick Links

- [Home](https://scriptkit.com)  
- [Blog](https://blog.scriptkit.com)  
- [Community](https://discord.scriptkit.com)

---

Thanks for using Script Kit!`
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
  console.error('[SMOKE] Starting links and special elements visual regression tests...');
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
  console.error('[SMOKE] Links and special elements visual regression tests complete');
  
  process.exit(failed > 0 ? 1 : 0);
}

main().catch(err => {
  console.error(`[SMOKE] Fatal error: ${err}`);
  process.exit(1);
});
