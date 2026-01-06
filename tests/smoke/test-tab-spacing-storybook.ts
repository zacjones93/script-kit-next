import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Compact storybook for Tab opacity and logo spacing variations
// Using inline styles for reliability
const html = `
<div style="display: flex; flex-direction: column; gap: 8px; padding: 12px; background: #171717; font-size: 12px;">
  <div style="font-size: 14px; font-weight: bold; color: white; margin-bottom: 4px;">Tab Opacity & Logo Spacing</div>
  
  <div style="color: #facc15; font-size: 11px;">Tab Opacity:</div>
  
  <!-- 20% -->
  <div style="display: flex; align-items: center; padding: 6px 12px; background: #262626; border-radius: 6px;">
    <span style="color: #6b7280; width: 50px;">20%:</span>
    <span style="color: white;">Script Kit</span>
    <div style="flex: 1;"></div>
    <span style="color: #facc15;">Ask AI</span>
    <span style="margin-left: 4px; padding: 2px 6px; background: rgba(64,64,64,0.2); border-radius: 4px; color: #9ca3af;">Tab</span>
    <span style="margin-left: 12px; color: #facc15;">Run</span><span style="margin-left: 4px; color: #9ca3af;">↵</span>
    <span style="margin-left: 12px; color: #facc15;">Actions</span><span style="margin-left: 4px; color: #6b7280;">⌘K</span>
    <div style="margin-left: 16px; width: 16px; height: 16px; background: rgba(250,204,21,0.85); border-radius: 4px; display: flex; align-items: center; justify-content: center;"><span style="color: black; font-size: 8px;">⚡</span></div>
  </div>
  
  <!-- 30% -->
  <div style="display: flex; align-items: center; padding: 6px 12px; background: #262626; border-radius: 6px;">
    <span style="color: #6b7280; width: 50px;">30%:</span>
    <span style="color: white;">Script Kit</span>
    <div style="flex: 1;"></div>
    <span style="color: #facc15;">Ask AI</span>
    <span style="margin-left: 4px; padding: 2px 6px; background: rgba(64,64,64,0.3); border-radius: 4px; color: #9ca3af;">Tab</span>
    <span style="margin-left: 12px; color: #facc15;">Run</span><span style="margin-left: 4px; color: #9ca3af;">↵</span>
    <span style="margin-left: 12px; color: #facc15;">Actions</span><span style="margin-left: 4px; color: #6b7280;">⌘K</span>
    <div style="margin-left: 16px; width: 16px; height: 16px; background: rgba(250,204,21,0.85); border-radius: 4px; display: flex; align-items: center; justify-content: center;"><span style="color: black; font-size: 8px;">⚡</span></div>
  </div>
  
  <!-- 40% current -->
  <div style="display: flex; align-items: center; padding: 6px 12px; background: #262626; border-radius: 6px; border: 1px solid rgba(250,204,21,0.3);">
    <span style="color: #facc15; width: 50px;">40%*:</span>
    <span style="color: white;">Script Kit</span>
    <div style="flex: 1;"></div>
    <span style="color: #facc15;">Ask AI</span>
    <span style="margin-left: 4px; padding: 2px 6px; background: rgba(64,64,64,0.4); border-radius: 4px; color: #9ca3af;">Tab</span>
    <span style="margin-left: 12px; color: #facc15;">Run</span><span style="margin-left: 4px; color: #9ca3af;">↵</span>
    <span style="margin-left: 12px; color: #facc15;">Actions</span><span style="margin-left: 4px; color: #6b7280;">⌘K</span>
    <div style="margin-left: 16px; width: 16px; height: 16px; background: rgba(250,204,21,0.85); border-radius: 4px; display: flex; align-items: center; justify-content: center;"><span style="color: black; font-size: 8px;">⚡</span></div>
  </div>
  
  <!-- 50% -->
  <div style="display: flex; align-items: center; padding: 6px 12px; background: #262626; border-radius: 6px;">
    <span style="color: #6b7280; width: 50px;">50%:</span>
    <span style="color: white;">Script Kit</span>
    <div style="flex: 1;"></div>
    <span style="color: #facc15;">Ask AI</span>
    <span style="margin-left: 4px; padding: 2px 6px; background: rgba(64,64,64,0.5); border-radius: 4px; color: #9ca3af;">Tab</span>
    <span style="margin-left: 12px; color: #facc15;">Run</span><span style="margin-left: 4px; color: #9ca3af;">↵</span>
    <span style="margin-left: 12px; color: #facc15;">Actions</span><span style="margin-left: 4px; color: #6b7280;">⌘K</span>
    <div style="margin-left: 16px; width: 16px; height: 16px; background: rgba(250,204,21,0.85); border-radius: 4px; display: flex; align-items: center; justify-content: center;"><span style="color: black; font-size: 8px;">⚡</span></div>
  </div>
  
  <div style="color: #facc15; font-size: 11px; margin-top: 8px;">Logo Margin (with 30% Tab):</div>
  
  <!-- 8px margin -->
  <div style="display: flex; align-items: center; padding: 6px 12px; background: #262626; border-radius: 6px;">
    <span style="color: #6b7280; width: 50px;">8px:</span>
    <span style="color: white;">Script Kit</span>
    <div style="flex: 1;"></div>
    <span style="color: #facc15;">Ask AI</span>
    <span style="margin-left: 4px; padding: 2px 6px; background: rgba(64,64,64,0.3); border-radius: 4px; color: #9ca3af;">Tab</span>
    <span style="margin-left: 12px; color: #facc15;">Run</span><span style="margin-left: 4px; color: #9ca3af;">↵</span>
    <span style="margin-left: 12px; color: #facc15;">Actions</span><span style="margin-left: 4px; color: #6b7280;">⌘K</span>
    <div style="margin-left: 8px; width: 16px; height: 16px; background: rgba(250,204,21,0.85); border-radius: 4px; display: flex; align-items: center; justify-content: center;"><span style="color: black; font-size: 8px;">⚡</span></div>
  </div>
  
  <!-- 10px margin -->
  <div style="display: flex; align-items: center; padding: 6px 12px; background: #262626; border-radius: 6px;">
    <span style="color: #6b7280; width: 50px;">10px:</span>
    <span style="color: white;">Script Kit</span>
    <div style="flex: 1;"></div>
    <span style="color: #facc15;">Ask AI</span>
    <span style="margin-left: 4px; padding: 2px 6px; background: rgba(64,64,64,0.3); border-radius: 4px; color: #9ca3af;">Tab</span>
    <span style="margin-left: 12px; color: #facc15;">Run</span><span style="margin-left: 4px; color: #9ca3af;">↵</span>
    <span style="margin-left: 12px; color: #facc15;">Actions</span><span style="margin-left: 4px; color: #6b7280;">⌘K</span>
    <div style="margin-left: 10px; width: 16px; height: 16px; background: rgba(250,204,21,0.85); border-radius: 4px; display: flex; align-items: center; justify-content: center;"><span style="color: black; font-size: 8px;">⚡</span></div>
  </div>
  
  <!-- 12px margin -->
  <div style="display: flex; align-items: center; padding: 6px 12px; background: #262626; border-radius: 6px;">
    <span style="color: #6b7280; width: 50px;">12px:</span>
    <span style="color: white;">Script Kit</span>
    <div style="flex: 1;"></div>
    <span style="color: #facc15;">Ask AI</span>
    <span style="margin-left: 4px; padding: 2px 6px; background: rgba(64,64,64,0.3); border-radius: 4px; color: #9ca3af;">Tab</span>
    <span style="margin-left: 12px; color: #facc15;">Run</span><span style="margin-left: 4px; color: #9ca3af;">↵</span>
    <span style="margin-left: 12px; color: #facc15;">Actions</span><span style="margin-left: 4px; color: #6b7280;">⌘K</span>
    <div style="margin-left: 12px; width: 16px; height: 16px; background: rgba(250,204,21,0.85); border-radius: 4px; display: flex; align-items: center; justify-content: center;"><span style="color: black; font-size: 8px;">⚡</span></div>
  </div>
  
  <!-- 16px margin current -->
  <div style="display: flex; align-items: center; padding: 6px 12px; background: #262626; border-radius: 6px; border: 1px solid rgba(250,204,21,0.3);">
    <span style="color: #facc15; width: 50px;">16px*:</span>
    <span style="color: white;">Script Kit</span>
    <div style="flex: 1;"></div>
    <span style="color: #facc15;">Ask AI</span>
    <span style="margin-left: 4px; padding: 2px 6px; background: rgba(64,64,64,0.3); border-radius: 4px; color: #9ca3af;">Tab</span>
    <span style="margin-left: 12px; color: #facc15;">Run</span><span style="margin-left: 4px; color: #9ca3af;">↵</span>
    <span style="margin-left: 12px; color: #facc15;">Actions</span><span style="margin-left: 4px; color: #6b7280;">⌘K</span>
    <div style="margin-left: 16px; width: 16px; height: 16px; background: rgba(250,204,21,0.85); border-radius: 4px; display: flex; align-items: center; justify-content: center;"><span style="color: black; font-size: 8px;">⚡</span></div>
  </div>
  
  <div style="color: #6b7280; font-size: 10px; margin-top: 4px;">* = current setting (yellow border)</div>
</div>
`;

div(html);

// Wait for render
await new Promise(r => setTimeout(r, 1000));

// Capture screenshot
const screenshot = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });
const path = join(dir, `tab-spacing-storybook-${Date.now()}.png`);
writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
console.error(`[SCREENSHOT] ${path}`);

process.exit(0);
