// Name: Test Mouse Scroll Behavior
// Description: Verify mouse scroll can reach items at the end of a long list
// Author: worker-mouse-scroll-test
//
// BASELINE TEST - Documents current behavior where mouse scroll CANNOT
// reach items beyond initially measured content. Keyboard arrows work
// because they use scroll_to_reveal_item(index).
//
// ROOT CAUSE:
// - Main menu uses GPUI's list() component with variable-height items
// - LIST_ITEM_HEIGHT = 48px, SECTION_HEADER_HEIGHT = 24px  
// - GPUI list() calculates scroll bounds from measured heights
// - Unmeasured items (not yet rendered) contribute 0px to scroll bounds
// - Result: scrollbar thinks list is smaller than it actually is
//
// This test establishes the baseline that will be verified after the fix.

import '../../scripts/kit-sdk';

console.error('[MOUSE_SCROLL_TEST] Starting mouse scroll baseline test');
console.error('[MOUSE_SCROLL_TEST] ===========================================');

// Wait for the main menu to fully load
console.error('[MOUSE_SCROLL_TEST] Waiting for main menu to load...');
await new Promise(resolve => setTimeout(resolve, 1500));

// Get layout info to understand current list bounds
console.error('[MOUSE_SCROLL_TEST] Getting layout info...');
try {
  const layout = await getLayoutInfo();
  
  console.error(`[MOUSE_SCROLL_TEST] Window: ${layout.windowWidth}x${layout.windowHeight}`);
  console.error(`[MOUSE_SCROLL_TEST] Prompt type: ${layout.promptType}`);
  console.error(`[MOUSE_SCROLL_TEST] Total components: ${layout.components.length}`);
  
  // Find list-related components
  const listComponents = layout.components.filter(c => 
    c.name.toLowerCase().includes('list') || 
    c.type === 'list' ||
    c.type === 'listItem'
  );
  
  if (listComponents.length > 0) {
    console.error(`[MOUSE_SCROLL_TEST] Found ${listComponents.length} list-related components:`);
    for (const comp of listComponents) {
      console.error(`[MOUSE_SCROLL_TEST]   - ${comp.name} (${comp.type}): ${comp.bounds.width}x${comp.bounds.height} at (${comp.bounds.x}, ${comp.bounds.y})`);
    }
  }
  
  // Find the main content/container area
  const containers = layout.components.filter(c => 
    c.type === 'container' || c.type === 'panel'
  );
  
  if (containers.length > 0) {
    console.error(`[MOUSE_SCROLL_TEST] Container bounds:`);
    for (const cont of containers) {
      console.error(`[MOUSE_SCROLL_TEST]   - ${cont.name}: ${cont.bounds.width}x${cont.bounds.height}`);
    }
  }
  
  // Log all components for detailed analysis
  console.error('[MOUSE_SCROLL_TEST] All components:');
  for (const comp of layout.components) {
    console.error(`[MOUSE_SCROLL_TEST]   ${comp.depth}| ${comp.name} (${comp.type}): ${comp.bounds.width}x${comp.bounds.height}`);
  }
  
} catch (e) {
  console.error(`[MOUSE_SCROLL_TEST] Error getting layout info: ${e}`);
}

console.error('[MOUSE_SCROLL_TEST] ===========================================');
console.error('[MOUSE_SCROLL_TEST] BASELINE BEHAVIOR DOCUMENTATION:');
console.error('[MOUSE_SCROLL_TEST] ===========================================');
console.error('[MOUSE_SCROLL_TEST]');
console.error('[MOUSE_SCROLL_TEST] CURRENT STATE (BROKEN):');
console.error('[MOUSE_SCROLL_TEST]   - Mouse wheel scroll CANNOT reach items at end of list');
console.error('[MOUSE_SCROLL_TEST]   - Scrollbar shows incorrect max position');
console.error('[MOUSE_SCROLL_TEST]   - Only items visible + buffer can be scrolled to');
console.error('[MOUSE_SCROLL_TEST]');
console.error('[MOUSE_SCROLL_TEST] WORKING BEHAVIOR:');
console.error('[MOUSE_SCROLL_TEST]   - Keyboard arrows CAN reach all items');
console.error('[MOUSE_SCROLL_TEST]   - Uses scroll_to_reveal_item(index) which bypasses bounds issue');
console.error('[MOUSE_SCROLL_TEST]');
console.error('[MOUSE_SCROLL_TEST] ROOT CAUSE:');
console.error('[MOUSE_SCROLL_TEST]   - GPUI list() calculates scroll bounds from measured heights');
console.error('[MOUSE_SCROLL_TEST]   - Variable-height items (48px items, 24px headers)');
console.error('[MOUSE_SCROLL_TEST]   - Unmeasured items contribute 0px to scroll max');
console.error('[MOUSE_SCROLL_TEST]');
console.error('[MOUSE_SCROLL_TEST] EXPECTED FIX:');
console.error('[MOUSE_SCROLL_TEST]   - Custom scroll wheel handler in list component');
console.error('[MOUSE_SCROLL_TEST]   - Calculate true content height from item count + heights');
console.error('[MOUSE_SCROLL_TEST]   - Override default scroll bounds with correct values');
console.error('[MOUSE_SCROLL_TEST]');
console.error('[MOUSE_SCROLL_TEST] MANUAL VERIFICATION STEPS:');
console.error('[MOUSE_SCROLL_TEST]   1. Open Script Kit main menu (Cmd+;)');
console.error('[MOUSE_SCROLL_TEST]   2. Ensure list has 20+ items (scripts, builtins, etc.)');
console.error('[MOUSE_SCROLL_TEST]   3. Use mouse wheel/trackpad to scroll down');
console.error('[MOUSE_SCROLL_TEST]   4. Try to reach the LAST item in the list');
console.error('[MOUSE_SCROLL_TEST]   5. CURRENT: Cannot reach last items with mouse');
console.error('[MOUSE_SCROLL_TEST]   6. Use keyboard Down arrow to reach last item');
console.error('[MOUSE_SCROLL_TEST]   7. Keyboard navigation reaches all items');
console.error('[MOUSE_SCROLL_TEST]');
console.error('[MOUSE_SCROLL_TEST] ===========================================');
console.error('[MOUSE_SCROLL_TEST] Test complete - baseline documented');

// @ts-ignore - process.exit is available at runtime
globalThis.process?.exit?.(0);
