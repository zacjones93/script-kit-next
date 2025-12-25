/**
 * TIER 4B: Widget, Term, and Media Prompt Tests
 * 
 * These test scripts demonstrate the widget, terminal, and media APIs
 * added in TIER 4B of the kit-sdk implementation.
 */

import '../../scripts/kit-sdk';

// =============================================================================
// Widget Tests
// =============================================================================

/**
 * Test 1: Basic Widget
 * Creates a simple floating widget with HTML content
 */
async function testBasicWidget() {
  const w = await widget(`
    <div style="padding: 20px; background: #1e1e1e; color: white; border-radius: 8px;">
      <h2>Hello Widget!</h2>
      <p>This is a floating HTML widget.</p>
      <button data-action="close">Close</button>
    </div>
  `);

  w.onClick((event) => {
    if (event.dataset.action === 'close') {
      w.close();
    }
  });
}

/**
 * Test 2: Widget with Options
 * Creates a widget with position and appearance options
 */
async function testWidgetWithOptions() {
  const w = await widget(
    `<div style="padding: 16px;">
      <h3>Positioned Widget</h3>
      <p>I'm always on top!</p>
    </div>`,
    {
      x: 100,
      y: 100,
      width: 300,
      height: 200,
      alwaysOnTop: true,
      transparent: false,
      draggable: true,
      hasShadow: true,
    }
  );

  // Close after 5 seconds
  setTimeout(() => w.close(), 5000);
}

/**
 * Test 3: Widget with State
 * Creates a widget that updates its state dynamically
 */
async function testWidgetWithState() {
  let count = 0;

  const w = await widget(`
    <div style="padding: 20px; text-align: center;">
      <h2>Counter: <span id="count">0</span></h2>
      <button data-action="increment">+1</button>
      <button data-action="decrement">-1</button>
      <button data-action="close">Close</button>
    </div>
  `);

  w.onClick((event) => {
    if (event.dataset.action === 'increment') {
      count++;
      w.setState({ count });
    } else if (event.dataset.action === 'decrement') {
      count--;
      w.setState({ count });
    } else if (event.dataset.action === 'close') {
      w.close();
    }
  });
}

/**
 * Test 4: Widget with Input
 * Creates a widget with input field handling
 */
async function testWidgetWithInput() {
  const w = await widget(`
    <div style="padding: 20px;">
      <h3>Type something:</h3>
      <input type="text" id="myInput" data-field="name" style="padding: 8px; width: 100%;">
      <p>Value: <span id="value"></span></p>
    </div>
  `);

  w.onInput((event) => {
    w.setState({ value: event.value });
  });
}

/**
 * Test 5: Widget with Move/Resize Events
 * Creates a widget that tracks its position and size
 */
async function testWidgetEvents() {
  const w = await widget(
    `<div style="padding: 20px;">
      <h3>Draggable Widget</h3>
      <p>Position: <span id="pos">-</span></p>
      <p>Size: <span id="size">-</span></p>
    </div>`,
    { draggable: true }
  );

  w.onMoved((pos) => {
    w.setState({ pos: `${pos.x}, ${pos.y}` });
  });

  w.onResized((size) => {
    w.setState({ size: `${size.width}x${size.height}` });
  });

  w.onClose(() => {
    console.log('Widget closed!');
  });
}

// =============================================================================
// Terminal Tests
// =============================================================================

/**
 * Test 6: Basic Terminal
 * Opens an interactive terminal
 */
async function testBasicTerm() {
  const output = await term();
  console.log('Terminal closed with output:', output);
}

/**
 * Test 7: Terminal with Command
 * Runs a command in the terminal and returns output
 */
async function testTermWithCommand() {
  const output = await term('ls -la');
  console.log('Command output:', output);
}

/**
 * Test 8: Terminal with Long Running Command
 * Runs a longer command
 */
async function testTermLongCommand() {
  const output = await term('echo "Starting..." && sleep 2 && echo "Done!"');
  console.log('Output:', output);
}

// =============================================================================
// Media Tests
// =============================================================================

/**
 * Test 9: Webcam Capture
 * Opens webcam preview and captures photo on Enter
 */
async function testWebcam() {
  const imageBuffer = await webcam();
  console.log('Captured image:', imageBuffer.length, 'bytes');
  
  // Could save to file:
  // require('fs').writeFileSync('capture.png', imageBuffer);
}

/**
 * Test 10: Microphone Recording
 * Records audio from microphone
 */
async function testMic() {
  const audioBuffer = await mic();
  console.log('Recorded audio:', audioBuffer.length, 'bytes');
  
  // Could save to file:
  // require('fs').writeFileSync('recording.wav', audioBuffer);
}

/**
 * Test 11: Eye Dropper
 * Picks a color from the screen
 */
async function testEyeDropper() {
  const color = await eyeDropper();
  console.log('Selected color:', color);
  console.log('  Hex:', color.sRGBHex);
  console.log('  RGB:', color.rgb);
  console.log('  HSL:', color.hsl);
}

// =============================================================================
// Find Tests
// =============================================================================

/**
 * Test 12: Basic Find
 * File search using Spotlight/mdfind
 */
async function testBasicFind() {
  const filePath = await find('Search for a file...');
  console.log('Selected file:', filePath);
}

/**
 * Test 13: Find with Directory Filter
 * Search within a specific directory
 */
async function testFindInDirectory() {
  const filePath = await find('Search in Documents...', {
    onlyin: `/Users/${process.env.USER}/Documents`,
  });
  console.log('Selected file:', filePath);
}

// =============================================================================
// Combined Test
// =============================================================================

/**
 * Test 14: Combined Demo
 * Demonstrates multiple TIER 4B features together
 */
async function testCombined() {
  // First, pick a color
  const color = await eyeDropper();
  
  // Show it in a widget
  const w = await widget(`
    <div style="padding: 20px; background: ${color.sRGBHex}; color: white; text-shadow: 0 0 2px black;">
      <h2>Color Picker Result</h2>
      <p>You picked: ${color.sRGBHex}</p>
      <p>RGB: ${color.rgb}</p>
      <button data-action="done">Done</button>
    </div>
  `, {
    width: 300,
    height: 200,
    alwaysOnTop: true,
  });

  // Wait for user to click done
  return new Promise<void>((resolve) => {
    w.onClick((event) => {
      if (event.dataset.action === 'done') {
        w.close();
        resolve();
      }
    });
  });
}

// =============================================================================
// Run selected test
// =============================================================================

// Export all tests for external running
export {
  testBasicWidget,
  testWidgetWithOptions,
  testWidgetWithState,
  testWidgetWithInput,
  testWidgetEvents,
  testBasicTerm,
  testTermWithCommand,
  testTermLongCommand,
  testWebcam,
  testMic,
  testEyeDropper,
  testBasicFind,
  testFindInDirectory,
  testCombined,
};

// Default: run basic widget test
testBasicWidget();
