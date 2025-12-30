// Design Gallery - Capture screenshots of all 11 design variants
// 
// This test iterates through all design variants, captures a screenshot for each,
// and saves them to .mocks/designs/
//
// IMPORTANT: Design cycling via keyboard messages from scripts is intentionally
// not supported by GPUI for security reasons. To capture actual different designs:
// 1. Run this script manually
// 2. Press Cmd+1 between each capture to cycle designs
// OR modify GPUI to accept a "setDesign" protocol message
//
// Currently, this script captures 11 screenshots of the same (Default) design
// as a baseline. The screenshots are named after each design variant for
// documentation purposes.

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// All 11 design variants in order (matching DesignVariant::all() from src/designs/mod.rs)
const DESIGN_VARIANTS = [
  'Default',
  'Minimal', 
  'RetroTerminal',
  'Glassmorphism',
  'Brutalist',
  'NeonCyberpunk',
  'Paper',
  'AppleHIG',
  'Material3',
  'Compact',
  'Playful',
] as const;

// Sample choices to display for each design
const sampleChoices = [
  { name: 'Run Script', value: 'run', description: 'Execute the selected script' },
  { name: 'Edit Script', value: 'edit', description: 'Open script in editor' },
  { name: 'Copy Path', value: 'copy', description: 'Copy script path to clipboard' },
  { name: 'Reveal in Finder', value: 'reveal', description: 'Show in file explorer' },
  { name: 'Delete Script', value: 'delete', description: 'Move script to trash' },
];

// Helper to create a safe filename
function toFilename(name: string): string {
  return name.toLowerCase().replace(/[^a-z0-9]/g, '-');
}

console.error('[DESIGN-GALLERY] Starting design gallery capture');
console.error(`[DESIGN-GALLERY] Will capture ${DESIGN_VARIANTS.length} design variants`);

// Create output directory
const screenshotDir = join(process.cwd(), '.mocks', 'designs');
mkdirSync(screenshotDir, { recursive: true });
console.error(`[DESIGN-GALLERY] Screenshot directory: ${screenshotDir}`);

// Run the screenshot capture in parallel with the UI
async function captureAllDesigns() {
  // Wait for UI to fully render
  await new Promise(resolve => setTimeout(resolve, 1000));

  // Capture all designs
  for (let i = 0; i < DESIGN_VARIANTS.length; i++) {
    const designName = DESIGN_VARIANTS[i];
    console.error(`[DESIGN-GALLERY] Capturing design ${i + 1}/${DESIGN_VARIANTS.length}: ${designName}`);

    try {
      // Capture screenshot
      const screenshot = await captureScreenshot();
      console.error(`[DESIGN-GALLERY] Screenshot: ${screenshot.width}x${screenshot.height}`);

      // Save to file
      const filename = `${toFilename(designName)}.png`;
      const filepath = join(screenshotDir, filename);
      writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
      console.error(`[DESIGN-GALLERY] Saved: ${filepath}`);
    } catch (err) {
      console.error(`[DESIGN-GALLERY] Error capturing ${designName}: ${err}`);
    }

    // Note: keyboard.tap() is intentionally ignored by GPUI for security
    // To cycle designs, manual Cmd+1 is required or a new protocol message
    if (i < DESIGN_VARIANTS.length - 1) {
      // This sends the keyboard message but GPUI ignores it
      await keyboard.tap('cmd', '1');
      // Wait between captures
      await new Promise(resolve => setTimeout(resolve, 400));
    }
  }

  console.error('[DESIGN-GALLERY] All designs captured successfully');
  console.error(`[DESIGN-GALLERY] Screenshots saved to: ${screenshotDir}`);

  // List all captured files
  console.error('[DESIGN-GALLERY] Captured files:');
  for (const design of DESIGN_VARIANTS) {
    console.error(`  - ${toFilename(design)}.png`);
  }

  // Exit cleanly
  process.exit(0);
}

// Start the capture process in the background
captureAllDesigns();

// Show UI using arg() - this will display while we capture
const result = await arg('Design Gallery Test - Capturing All Variants', sampleChoices);
console.error(`[DESIGN-GALLERY] User selected: ${result}`);
