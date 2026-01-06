import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Capture the main script list header immediately
// The "show" command was sent before this script runs, so the main menu is visible
// This script does NOT call any UI functions (arg, div, etc.) so it won't change the view

const screenshot = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });
const path = join(dir, `main-header-${Date.now()}.png`);
writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
console.error(`[SCREENSHOT] ${path}`);

process.exit(0);
