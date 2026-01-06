// @ts-nocheck
// Test to capture main menu header and verify Ask AI hint
import '../../scripts/kit-sdk';
import { mkdirSync, writeFileSync } from 'fs';
import { join } from 'path';

// The main menu should already be showing - just wait for render
await new Promise(r => setTimeout(r, 1000));

// Capture screenshot of the main menu
const shot = await captureScreenshot();
const dir = join(process.cwd(), '.test-screenshots');
mkdirSync(dir, { recursive: true });

const path = join(dir, `main-menu-header-${Date.now()}.png`);
writeFileSync(path, Buffer.from(shot.data, 'base64'));
console.error(`[SCREENSHOT] ${path}`);
process.exit(0);
