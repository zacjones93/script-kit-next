import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

export const metadata = {
  name: "Visual Audit - Fields Prompt",
  description: "Captures fields() prompt for visual audit",
};

console.error('[AUDIT] Starting fields visual audit...');

// Create screenshot directory
const screenshotDir = join(process.cwd(), '.test-screenshots', 'grid-audit');
mkdirSync(screenshotDir, { recursive: true });

// Capture after render
setTimeout(async () => {
  try {
    const screenshot = await captureScreenshot();
    console.error(`[AUDIT] Captured fields: ${screenshot.width}x${screenshot.height}`);
    
    const filename = '05-fields.png';
    const filepath = join(screenshotDir, filename);
    writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
    console.error(`[SCREENSHOT] ${filepath}`);
    
    process.exit(0);
  } catch (err) {
    console.error('[AUDIT] Screenshot failed:', err);
    process.exit(1);
  }
}, 1500);

// Display fields form
await fields([
  { name: "firstName", label: "First Name", placeholder: "Enter your first name" },
  { name: "lastName", label: "Last Name", placeholder: "Enter your last name" },
  { name: "email", label: "Email Address", placeholder: "you@example.com" },
  { name: "phone", label: "Phone Number", placeholder: "+1 (555) 123-4567" },
]);
