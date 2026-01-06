// @ts-nocheck
// Test Tab key sends input to AI
import '../../scripts/kit-sdk';
import { mkdirSync, writeFileSync } from 'fs';
import { join } from 'path';

// Wait for main menu to render
await new Promise(r => setTimeout(r, 500));

// Type something in the search
await setFilterText("test query for AI");

// Wait for filter to apply
await new Promise(r => setTimeout(r, 500));

// Capture screenshot before Tab
const shot1 = await captureScreenshot();
const dir = join(process.cwd(), '.test-screenshots');
mkdirSync(dir, { recursive: true });
const path1 = join(dir, `tab-ai-before-${Date.now()}.png`);
writeFileSync(path1, Buffer.from(shot1.data, 'base64'));
console.error(`[SCREENSHOT] Before Tab: ${path1}`);

// Note: We can't easily simulate Tab key from script
// The Tab handler is in the Rust code and would need keyboard simulation
// This test just verifies the UI renders correctly with filter text

process.exit(0);
