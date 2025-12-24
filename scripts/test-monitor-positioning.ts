// Name: Multi-Monitor Position Test
// Description: Verifies that the window appears on the same monitor as the mouse cursor
// Author: Script Kit Team

/**
 * MULTI-MONITOR TEST: test-monitor-positioning.ts
 *
 * This script tests that the Script Kit window appears on the correct monitor
 * (the one where the mouse cursor is located). It works by:
 *
 * 1. Displaying a prompt on the current monitor
 * 2. Reading the POSITION logs from the Rust app to determine which monitor
 *    the mouse was on and which monitor the window appeared on
 * 3. Verifying they match
 *
 * HOW TO USE:
 * 1. Move your mouse to a specific monitor
 * 2. Press the global hotkey (default: Ctrl+Cmd+O) to show Script Kit
 * 3. Run this script from the script list
 * 4. The script will show test results
 *
 * INTERPRETING RESULTS:
 * - Check the terminal/console output for [POSITION] logs from the Rust side
 * - The script will display a summary of which monitor it detected
 *
 * Expected log output from Rust (visible in terminal):
 * [POSITION] ╔════════════════════════════════════════════════════════════╗
 * [POSITION] ║  CALCULATING WINDOW POSITION FOR MOUSE DISPLAY             ║
 * [POSITION] ╚════════════════════════════════════════════════════════════╝
 * [POSITION] Available displays: 2
 * [POSITION]   Display 0: origin=(0, 0) size=1920x1080 [bounds: x=0..1920, y=0..1080]
 * [POSITION]   Display 1: origin=(1920, 0) size=2560x1440 [bounds: x=1920..4480, y=0..1440]
 * [POSITION] Mouse cursor at (2500, 720)
 * [POSITION]   -> Mouse is on display 1
 * [POSITION] Selected display: origin=(1920, 0) size=2560x1440
 */

import './kit-sdk';

// Log test start to stderr (visible in terminal)
console.error('[TEST] ════════════════════════════════════════════════════════');
console.error('[TEST] MULTI-MONITOR POSITIONING TEST');
console.error('[TEST] ════════════════════════════════════════════════════════');
console.error('[TEST] This script tests that windows appear on the monitor');
console.error('[TEST] where the mouse cursor is located.');
console.error('[TEST]');
console.error('[TEST] Check the terminal output for [POSITION] logs from the');
console.error('[TEST] Rust application to verify monitor detection.');
console.error('[TEST] ════════════════════════════════════════════════════════');

// Step 1: Show initial prompt to trigger window positioning
console.error('[TEST] Step 1: Displaying initial prompt...');
console.error('[TEST] The window should have appeared on the monitor where');
console.error('[TEST] your mouse cursor was when you activated Script Kit.');
console.error('[TEST]');

const testChoice = await arg('Multi-Monitor Test - Which monitor is this window on?', [
  {
    name: 'Monitor 1 (Primary)',
    value: 'monitor-1',
    description: 'This window is on my primary/main monitor',
  },
  {
    name: 'Monitor 2 (Secondary)',
    value: 'monitor-2',
    description: 'This window is on my secondary monitor',
  },
  {
    name: 'Monitor 3+ (Additional)',
    value: 'monitor-3+',
    description: 'This window is on a third or additional monitor',
  },
  {
    name: 'Wrong Monitor',
    value: 'wrong',
    description: 'The window appeared on a DIFFERENT monitor than my mouse!',
  },
]);

console.error(`[TEST] User selected: ${testChoice}`);

// Step 2: Determine test result
const passed = testChoice !== 'wrong';

// Step 3: Display result
const resultMessage = passed
  ? `# ✅ TEST PASSED

The window appeared on the expected monitor.

## Your Selection
${testChoice === 'monitor-1' ? '**Monitor 1 (Primary)**' : ''}
${testChoice === 'monitor-2' ? '**Monitor 2 (Secondary)**' : ''}
${testChoice === 'monitor-3+' ? '**Monitor 3+ (Additional)**' : ''}

## What This Means
The Script Kit window correctly detected your mouse cursor position
and appeared on the same monitor. Multi-monitor positioning is working!

## Verification Steps
1. Check the terminal for **[POSITION]** logs
2. The logs should show:
   - All available displays with their bounds
   - Mouse cursor coordinates
   - Which display was selected

---

*Press Enter or Escape to close*`
  : `# ❌ TEST FAILED

The window appeared on a **different** monitor than your mouse cursor.

## What Went Wrong
This indicates a problem with multi-monitor positioning. The window
should appear on the same monitor where your mouse cursor is located.

## Debugging Steps

### 1. Check Terminal Logs
Look for **[POSITION]** logs in the terminal output:
- "Mouse cursor at (x, y)" - Mouse coordinates
- "Mouse is on display N" - Which display was detected
- "Selected display: origin=(x, y)" - Where window was placed

### 2. Common Issues
- **Coordinate mismatch**: macOS uses bottom-left origin, but the mouse
  position might be in top-left coordinates
- **Display origin wrong**: Secondary displays have non-zero origins
- **Scaling issues**: Retina displays may have coordinate scaling

### 3. Report Issue
If this consistently fails, please file an issue with:
- Your display configuration (System Preferences > Displays)
- The [POSITION] logs from the terminal
- Which monitor had your mouse vs which had the window

---

*Press Enter or Escape to close*`;

console.error(`[TEST] Result: ${passed ? 'PASS' : 'FAIL'}`);
console.error('[TEST] ════════════════════════════════════════════════════════');

await div(md(resultMessage));

console.error('[TEST] Multi-monitor positioning test completed.');
console.error('[TEST] ════════════════════════════════════════════════════════');
