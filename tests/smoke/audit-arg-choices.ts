import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

export const metadata = {
  name: "Visual Audit - Arg with Choices",
  description: "Captures arg() prompt with string choices for visual audit",
};

console.error('[AUDIT] Starting arg choices visual audit...');

// Create screenshot directory
const screenshotDir = join(process.cwd(), '.test-screenshots', 'grid-audit');
mkdirSync(screenshotDir, { recursive: true });

// Set up arg prompt with choices - this displays a list
// We need to capture before user interaction
setTimeout(async () => {
  try {
    const screenshot = await captureScreenshot();
    console.error(`[AUDIT] Captured arg choices: ${screenshot.width}x${screenshot.height}`);
    
    const filename = '04-arg-choices.png';
    const filepath = join(screenshotDir, filename);
    writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
    console.error(`[SCREENSHOT] ${filepath}`);
    
    // Exit after capture
    process.exit(0);
  } catch (err) {
    console.error('[AUDIT] Screenshot failed:', err);
    process.exit(1);
  }
}, 1500);

// Display arg with choices - this will show the list UI
const choices = [
  { name: "Apple", value: "apple", description: "A red fruit" },
  { name: "Banana", value: "banana", description: "A yellow fruit" },
  { name: "Cherry", value: "cherry", description: "A small red fruit" },
  { name: "Date", value: "date", description: "A sweet dried fruit" },
  { name: "Elderberry", value: "elderberry", description: "A dark purple berry" },
];

await arg("Select a fruit:", choices);
