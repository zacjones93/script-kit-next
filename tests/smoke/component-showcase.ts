// Name: Component Showcase Test
// Description: Captures screenshots of all UI component states for visual regression testing

/**
 * SMOKE TEST: component-showcase.ts
 * 
 * This script captures screenshots of all UI component variations including:
 * - Buttons (Primary, Ghost, Icon variants)
 * - Toasts (Success, Error, Warning, Info)
 * - Form Inputs (Text, Password, Textarea, Checkbox)
 * - List Items (Normal, Selected, Hover states)
 * - Scrollbar appearance
 * 
 * Screenshots are saved to: .mocks/components/
 * 
 * Usage:
 *   cargo build && echo '{"type":"run","path":"'$(pwd)'/tests/smoke/component-showcase.ts"}' | \
 *     SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
 */

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync, existsSync } from 'fs';
import { join } from 'path';

console.error('[SHOWCASE] component-showcase.ts starting...');

// Ensure screenshot directory exists
const screenshotDir = join(process.cwd(), '.mocks', 'components');
mkdirSync(screenshotDir, { recursive: true });
console.error(`[SHOWCASE] Screenshot directory: ${screenshotDir}`);

// Helper to render and capture component screenshot
async function captureComponent(name: string, html: string, renderDelayMs = 400): Promise<void> {
  // Start the div rendering (don't await it, we'll submit programmatically)
  const divPromise = div(wrapComponent(html));
  
  // Wait for render
  await new Promise(resolve => setTimeout(resolve, renderDelayMs));
  
  // Capture screenshot while div is displayed
  const screenshot = await captureScreenshot();
  const filepath = join(screenshotDir, `${name}.png`);
  writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
  console.error(`[COMPONENT] ${name} (${screenshot.width}x${screenshot.height})`);
  
  // Auto-submit to close the div and continue
  submit('');
  await divPromise;
}

// ============================================================================
// Component Definitions
// ============================================================================

// Button Components - matching src/components/button.rs styles
const BUTTONS = {
  'button-primary': `
    <button class="px-3 py-1.5 bg-yellow-500/20 text-yellow-400 rounded-md font-medium text-sm hover:bg-yellow-500/30 transition-colors">
      Primary Button
    </button>
  `,
  'button-primary-with-shortcut': `
    <button class="flex items-center gap-1 px-3 py-1.5 bg-yellow-500/20 text-yellow-400 rounded-md font-medium text-sm">
      <span>Run</span>
      <span class="text-xs ml-1">↵</span>
    </button>
  `,
  'button-ghost': `
    <button class="px-2 py-1 text-yellow-400 rounded-md text-sm hover:bg-white/15 transition-colors">
      Ghost Button
    </button>
  `,
  'button-icon': `
    <button class="p-1.5 text-yellow-400 rounded-md hover:bg-white/15 transition-colors">
      <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
        <path d="M10 5a1 1 0 011 1v3h3a1 1 0 110 2h-3v3a1 1 0 11-2 0v-3H6a1 1 0 110-2h3V6a1 1 0 011-1z"/>
      </svg>
    </button>
  `,
  'button-disabled': `
    <button class="px-3 py-1.5 bg-yellow-500/20 text-yellow-400/50 rounded-md font-medium text-sm cursor-not-allowed opacity-50" disabled>
      Disabled
    </button>
  `,
  'button-group': `
    <div class="flex items-center gap-2">
      <button class="px-3 py-1.5 bg-yellow-500/20 text-yellow-400 rounded-md font-medium text-sm">Primary</button>
      <button class="px-2 py-1 text-yellow-400 rounded-md text-sm hover:bg-white/15">Ghost</button>
      <button class="p-1.5 text-yellow-400 rounded-md hover:bg-white/15">
        <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
          <path d="M10 6a2 2 0 110-4 2 2 0 010 4zM10 12a2 2 0 110-4 2 2 0 010 4zM10 18a2 2 0 110-4 2 2 0 010 4z"/>
        </svg>
      </button>
    </div>
  `,
};

// Toast Components - matching src/components/toast.rs styles
const TOASTS = {
  'toast-success': `
    <div class="w-96 bg-gray-800/95 border-l-4 border-green-500 rounded-lg shadow-lg overflow-hidden">
      <div class="flex items-start gap-3 px-4 py-3">
        <div class="flex items-center justify-center w-6 h-6 text-lg text-green-500 font-bold">✓</div>
        <div class="flex-1">
          <p class="text-sm text-white font-medium">Success! Operation completed.</p>
        </div>
        <button class="w-5 h-5 flex items-center justify-center text-gray-400 hover:text-white rounded">×</button>
      </div>
    </div>
  `,
  'toast-error': `
    <div class="w-96 bg-gray-800/95 border-l-4 border-red-500 rounded-lg shadow-lg overflow-hidden">
      <div class="flex items-start gap-3 px-4 py-3">
        <div class="flex items-center justify-center w-6 h-6 text-lg text-red-500 font-bold">✕</div>
        <div class="flex-1 flex flex-col gap-2">
          <p class="text-sm text-white font-medium">Error: Something went wrong.</p>
          <div class="flex gap-2">
            <button class="px-2 py-1 bg-gray-700/80 text-yellow-400 text-xs font-medium rounded hover:bg-gray-600">Copy Error</button>
          </div>
        </div>
        <button class="w-5 h-5 flex items-center justify-center text-gray-400 hover:text-white rounded">×</button>
      </div>
    </div>
  `,
  'toast-warning': `
    <div class="w-96 bg-gray-800/95 border-l-4 border-yellow-500 rounded-lg shadow-lg overflow-hidden">
      <div class="flex items-start gap-3 px-4 py-3">
        <div class="flex items-center justify-center w-6 h-6 text-lg text-yellow-500 font-bold">⚠</div>
        <div class="flex-1">
          <p class="text-sm text-white font-medium">Warning: Check your settings.</p>
        </div>
        <button class="w-5 h-5 flex items-center justify-center text-gray-400 hover:text-white rounded">×</button>
      </div>
    </div>
  `,
  'toast-info': `
    <div class="w-96 bg-gray-800/95 border-l-4 border-blue-500 rounded-lg shadow-lg overflow-hidden">
      <div class="flex items-start gap-3 px-4 py-3">
        <div class="flex items-center justify-center w-6 h-6 text-lg text-blue-500 font-bold">ℹ</div>
        <div class="flex-1">
          <p class="text-sm text-white font-medium">Info: New version available.</p>
        </div>
        <button class="w-5 h-5 flex items-center justify-center text-gray-400 hover:text-white rounded">×</button>
      </div>
    </div>
  `,
  'toast-with-details': `
    <div class="w-96 bg-gray-800/95 border-l-4 border-red-500 rounded-lg shadow-lg overflow-hidden">
      <div class="flex items-start gap-3 px-4 py-3">
        <div class="flex items-center justify-center w-6 h-6 text-lg text-red-500 font-bold">✕</div>
        <div class="flex-1 flex flex-col gap-2">
          <p class="text-sm text-white font-medium">TypeError: Cannot read property</p>
          <span class="text-xs text-yellow-400 cursor-pointer hover:underline">View details</span>
        </div>
        <button class="w-5 h-5 flex items-center justify-center text-gray-400 hover:text-white rounded">×</button>
      </div>
      <div class="w-full px-4 py-3 bg-black/20 border-t border-red-500/40">
        <pre class="text-xs text-white font-mono overflow-hidden">at MyClass.method (file.ts:42)
at process.run (lib.ts:15)</pre>
      </div>
    </div>
  `,
};

// Form Input Components - matching src/components/form_fields.rs styles
const FORM_INPUTS = {
  'input-text': `
    <div class="flex items-center gap-3 w-full">
      <label class="w-28 text-sm text-gray-400 font-medium">Username</label>
      <div class="flex-1 flex items-center h-9 px-3 bg-gray-700/50 border border-gray-600 rounded-md">
        <span class="text-lg text-white">john_doe</span>
      </div>
    </div>
  `,
  'input-text-focused': `
    <div class="flex items-center gap-3 w-full">
      <label class="w-28 text-sm text-gray-400 font-medium">Username</label>
      <div class="flex-1 flex items-center h-9 px-3 bg-gray-900 border border-yellow-400 rounded-md">
        <span class="text-lg text-white">john</span>
        <div class="w-0.5 h-5 bg-cyan-400 animate-pulse"></div>
      </div>
    </div>
  `,
  'input-text-placeholder': `
    <div class="flex items-center gap-3 w-full">
      <label class="w-28 text-sm text-gray-400 font-medium">Username</label>
      <div class="flex-1 flex items-center h-9 px-3 bg-gray-700/50 border border-gray-600 rounded-md">
        <span class="text-lg text-gray-500">Enter username</span>
      </div>
    </div>
  `,
  'input-password': `
    <div class="flex items-center gap-3 w-full">
      <label class="w-28 text-sm text-gray-400 font-medium">Password</label>
      <div class="flex-1 flex items-center h-9 px-3 bg-gray-700/50 border border-gray-600 rounded-md">
        <span class="text-lg text-white">••••••••</span>
      </div>
    </div>
  `,
  'input-textarea': `
    <div class="flex items-start gap-3 w-full">
      <label class="w-28 pt-2 text-sm text-gray-400 font-medium">Description</label>
      <div class="flex-1 flex flex-col h-24 px-3 py-2 bg-gray-700/50 border border-gray-600 rounded-md overflow-hidden">
        <span class="text-sm text-white">Multi-line text input
that supports multiple lines
of content.</span>
      </div>
    </div>
  `,
  'input-checkbox-unchecked': `
    <div class="flex items-center gap-3 w-full">
      <div class="w-28"></div>
      <div class="flex items-center gap-2 cursor-pointer">
        <div class="flex items-center justify-center w-5 h-5 bg-gray-700/50 border border-gray-600 rounded"></div>
        <span class="text-sm text-white">Remember me</span>
      </div>
    </div>
  `,
  'input-checkbox-checked': `
    <div class="flex items-center gap-3 w-full">
      <div class="w-28"></div>
      <div class="flex items-center gap-2 cursor-pointer">
        <div class="flex items-center justify-center w-5 h-5 bg-yellow-400 border border-yellow-400 rounded">
          <span class="text-sm text-gray-900 font-bold">✓</span>
        </div>
        <span class="text-sm text-white">Remember me</span>
      </div>
    </div>
  `,
};

// List Item Components - matching src/list_item.rs styles (48px height)
const LIST_ITEMS = {
  'list-item-normal': `
    <div class="flex items-center gap-3 w-full h-12 px-3 bg-transparent hover:bg-white/5 cursor-pointer">
      <span class="text-xl">📜</span>
      <div class="flex-1 flex flex-col justify-center min-w-0">
        <span class="text-sm text-white font-medium truncate">Hello World Script</span>
        <span class="text-xs text-gray-400 truncate">A simple greeting script</span>
      </div>
    </div>
  `,
  'list-item-selected': `
    <div class="flex items-center gap-3 w-full h-12 px-3 bg-yellow-500/15 cursor-pointer">
      <span class="text-xl">⚡</span>
      <div class="flex-1 flex flex-col justify-center min-w-0">
        <span class="text-sm text-white font-medium truncate">Selected Script</span>
        <span class="text-xs text-gray-300 truncate">Currently highlighted item</span>
      </div>
      <span class="text-xs text-gray-400 font-mono">↵</span>
    </div>
  `,
  'list-item-with-shortcut': `
    <div class="flex items-center gap-3 w-full h-12 px-3 bg-transparent hover:bg-white/5 cursor-pointer">
      <span class="text-xl">🔧</span>
      <div class="flex-1 flex flex-col justify-center min-w-0">
        <span class="text-sm text-white font-medium truncate">Settings</span>
        <span class="text-xs text-gray-400 truncate">Configure Script Kit</span>
      </div>
      <span class="px-1.5 py-0.5 text-xs text-yellow-400 bg-yellow-500/10 rounded">⌘,</span>
    </div>
  `,
  'list-item-with-icon': `
    <div class="flex items-center gap-3 w-full h-12 px-3 bg-transparent hover:bg-white/5 cursor-pointer">
      <div class="w-6 h-6 flex items-center justify-center text-gray-400">
        <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
          <path d="M4 4a2 2 0 012-2h4.586A2 2 0 0112 2.586L15.414 6A2 2 0 0116 7.414V16a2 2 0 01-2 2H6a2 2 0 01-2-2V4z"/>
        </svg>
      </div>
      <div class="flex-1 flex flex-col justify-center min-w-0">
        <span class="text-sm text-white font-medium truncate">File Operations</span>
        <span class="text-xs text-gray-400 truncate">Read, write, and manage files</span>
      </div>
    </div>
  `,
  'list-item-group': `
    <div class="flex flex-col w-full">
      <div class="pt-4 pb-1 px-3">
        <span class="text-xs text-gray-500 font-semibold uppercase tracking-wide">RECENT</span>
      </div>
      <div class="flex items-center gap-3 w-full h-12 px-3 bg-yellow-500/15">
        <span class="text-xl">📜</span>
        <div class="flex-1 flex flex-col justify-center">
          <span class="text-sm text-white font-medium">First Script</span>
          <span class="text-xs text-gray-300">Used 5 minutes ago</span>
        </div>
      </div>
      <div class="flex items-center gap-3 w-full h-12 px-3 hover:bg-white/5">
        <span class="text-xl">⚡</span>
        <div class="flex-1 flex flex-col justify-center">
          <span class="text-sm text-white font-medium">Second Script</span>
          <span class="text-xs text-gray-400">Used 1 hour ago</span>
        </div>
      </div>
    </div>
  `,
};

// Scrollbar Component - matching src/components/scrollbar.rs styles
const SCROLLBAR = {
  'scrollbar-track': `
    <div class="relative w-64 h-48 bg-gray-800 rounded-lg overflow-hidden">
      <div class="absolute inset-0 p-3 text-sm text-gray-400">
        Scrollable content area...
      </div>
      <div class="absolute top-0 bottom-0 right-0.5 w-1.5 flex flex-col">
        <div class="flex-grow" style="flex-basis: 10%"></div>
        <div class="bg-gray-400/40 hover:bg-gray-300/60 rounded-full" style="flex-basis: 30%; min-height: 20px"></div>
        <div class="flex-grow" style="flex-basis: 60%"></div>
      </div>
    </div>
  `,
  'scrollbar-top': `
    <div class="relative w-64 h-48 bg-gray-800 rounded-lg overflow-hidden">
      <div class="absolute inset-0 p-3 text-sm text-gray-400">
        Content at top...
      </div>
      <div class="absolute top-0 bottom-0 right-0.5 w-1.5 flex flex-col">
        <div class="bg-gray-400/40 rounded-full" style="flex-basis: 30%; min-height: 20px"></div>
        <div class="flex-grow"></div>
      </div>
    </div>
  `,
  'scrollbar-bottom': `
    <div class="relative w-64 h-48 bg-gray-800 rounded-lg overflow-hidden">
      <div class="absolute inset-0 p-3 text-sm text-gray-400">
        Content at bottom...
      </div>
      <div class="absolute top-0 bottom-0 right-0.5 w-1.5 flex flex-col">
        <div class="flex-grow"></div>
        <div class="bg-gray-400/40 rounded-full" style="flex-basis: 30%; min-height: 20px"></div>
      </div>
    </div>
  `,
};

// Combined Layout Components
const LAYOUTS = {
  'layout-prompt-header': `
    <div class="flex items-center gap-2 w-full h-10 px-3 bg-gray-800/80 border-b border-gray-700">
      <span class="text-xl">⚡</span>
      <span class="text-sm text-white font-medium">Select Script</span>
      <div class="flex-1"></div>
      <button class="p-1 text-gray-400 hover:text-white rounded hover:bg-white/10">
        <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
          <path d="M6 10a2 2 0 11-4 0 2 2 0 014 0zM12 10a2 2 0 11-4 0 2 2 0 014 0zM16 12a2 2 0 100-4 2 2 0 000 4z"/>
        </svg>
      </button>
    </div>
  `,
  'layout-search-input': `
    <div class="flex items-center w-full h-12 px-4 bg-gray-800 border-b border-gray-700">
      <svg class="w-5 h-5 text-gray-500 mr-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
      </svg>
      <input type="text" placeholder="Search scripts..." class="flex-1 bg-transparent text-white placeholder-gray-500 text-lg outline-none"/>
    </div>
  `,
  'layout-action-bar': `
    <div class="flex items-center justify-between w-full h-10 px-3 bg-gray-900/80 border-t border-gray-700">
      <div class="flex items-center gap-2">
        <span class="text-xs text-gray-500">3 scripts</span>
      </div>
      <div class="flex items-center gap-2">
        <button class="px-2 py-1 text-xs text-yellow-400 hover:bg-white/10 rounded">Actions</button>
        <button class="px-2 py-1 text-xs text-gray-400 hover:bg-white/10 rounded">Settings</button>
      </div>
    </div>
  `,
};

// ============================================================================
// Capture Components
// ============================================================================

// Container wrapper for consistent styling
const wrapComponent = (html: string, bgClass = 'bg-gray-900') => `
  <div class="p-8 ${bgClass} min-h-screen flex items-center justify-center">
    ${html}
  </div>
`;

// Capture all component categories
console.error('[SHOWCASE] Capturing button components...');
for (const [name, html] of Object.entries(BUTTONS)) {
  await captureComponent(name, html);
}

console.error('[SHOWCASE] Capturing toast components...');
for (const [name, html] of Object.entries(TOASTS)) {
  await captureComponent(name, html);
}

console.error('[SHOWCASE] Capturing form input components...');
for (const [name, html] of Object.entries(FORM_INPUTS)) {
  await captureComponent(name, `<div class="w-96">${html}</div>`);
}

console.error('[SHOWCASE] Capturing list item components...');
for (const [name, html] of Object.entries(LIST_ITEMS)) {
  await captureComponent(name, `<div class="w-96">${html}</div>`);
}

console.error('[SHOWCASE] Capturing scrollbar components...');
for (const [name, html] of Object.entries(SCROLLBAR)) {
  await captureComponent(name, html);
}

console.error('[SHOWCASE] Capturing layout components...');
for (const [name, html] of Object.entries(LAYOUTS)) {
  await captureComponent(name, `<div class="w-[500px]">${html}</div>`);
}

// ============================================================================
// Summary
// ============================================================================

const componentCount = 
  Object.keys(BUTTONS).length + 
  Object.keys(TOASTS).length + 
  Object.keys(FORM_INPUTS).length + 
  Object.keys(LIST_ITEMS).length + 
  Object.keys(SCROLLBAR).length +
  Object.keys(LAYOUTS).length;

console.error(`[SHOWCASE] Complete! Captured ${componentCount} component screenshots.`);
console.error(`[SHOWCASE] Screenshots saved to: ${screenshotDir}`);

// Show summary
await div(md(`# Component Showcase Complete! 🎉

Captured **${componentCount}** component screenshots to:
\`\`\`
${screenshotDir}
\`\`\`

## Categories:
- **Buttons**: ${Object.keys(BUTTONS).length} variants
- **Toasts**: ${Object.keys(TOASTS).length} variants  
- **Form Inputs**: ${Object.keys(FORM_INPUTS).length} variants
- **List Items**: ${Object.keys(LIST_ITEMS).length} variants
- **Scrollbar**: ${Object.keys(SCROLLBAR).length} variants
- **Layouts**: ${Object.keys(LAYOUTS).length} variants

Press **Escape** or click to exit.`));

console.error('[SHOWCASE] component-showcase.ts completed successfully!');
