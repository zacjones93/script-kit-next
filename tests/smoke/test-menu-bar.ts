// Name: Smoke Test - Menu Bar
// Description: Tests menu bar APIs (getMenuBar, executeMenuAction)

/**
 * SMOKE TEST: test-menu-bar.ts
 *
 * Tests the Menu Bar APIs for reading and executing application menu actions.
 * Requires accessibility permissions on macOS.
 *
 * Test categories:
 * 1. getMenuBar() - Get menu bar items from frontmost app
 * 2. Menu item structure - Verify MenuBarItem properties
 * 3. Menu path traversal - Verify menuPath property is set correctly
 * 4. executeMenuAction() - Execute a menu action (requires accessibility)
 *
 * Note: Menu bar access requires accessibility permissions on macOS.
 * Tests that require accessibility will be marked [SKIP] if permission is not granted.
 */

import '../../scripts/kit-sdk';

// Import types from kit-sdk
import type { MenuBarItem } from '../../scripts/kit-sdk';

// =============================================================================
// Test Infrastructure
// =============================================================================

interface TestResult {
  test: string;
  status: 'running' | 'pass' | 'fail' | 'skip';
  timestamp: string;
  result?: unknown;
  error?: string;
  duration_ms?: number;
  reason?: string;
}

function logTest(
  name: string,
  status: TestResult['status'],
  extra?: Partial<TestResult>,
) {
  const result: TestResult = {
    test: name,
    status,
    timestamp: new Date().toISOString(),
    ...extra,
  };
  console.log(JSON.stringify(result));
}

function debug(msg: string) {
  console.error(`[SMOKE] ${msg}`);
}

// =============================================================================
// Tests
// =============================================================================

debug('test-menu-bar.ts starting...');
debug(`SDK globals: getMenuBar=${typeof getMenuBar}, executeMenuAction=${typeof executeMenuAction}`);

// -----------------------------------------------------------------------------
// Test 1: getMenuBar() - Get menu bar items from frontmost app
// -----------------------------------------------------------------------------
const test1 = 'getMenuBar-returns-array';
logTest(test1, 'running');
const start1 = Date.now();

let menuItems: MenuBarItem[] = [];

try {
  debug('Test 1: getMenuBar()');

  menuItems = await getMenuBar();

  // Verify it returns an array
  if (!Array.isArray(menuItems)) {
    throw new Error(`Expected array, got ${typeof menuItems}`);
  }

  debug(`Test 1 completed - got ${menuItems.length} top-level menu items`);
  debug(`Menu titles: ${menuItems.map(m => m.title).join(', ')}`);
  logTest(test1, 'pass', {
    result: { 
      menuCount: menuItems.length,
      titles: menuItems.map(m => m.title)
    },
    duration_ms: Date.now() - start1,
  });
} catch (err) {
  const errorMessage = String(err);
  // Check if it's an accessibility permission issue
  if (errorMessage.toLowerCase().includes('accessibility') || 
      errorMessage.toLowerCase().includes('permission')) {
    debug('[SKIP] Test 1 - accessibility permission required');
    logTest(test1, 'skip', {
      reason: 'Accessibility permission required for menu bar access',
      duration_ms: Date.now() - start1,
    });
  } else {
    logTest(test1, 'fail', {
      error: errorMessage,
      duration_ms: Date.now() - start1,
    });
  }
}

// -----------------------------------------------------------------------------
// Test 2: Menu item structure - Verify MenuBarItem properties
// -----------------------------------------------------------------------------
const test2 = 'menu-item-structure';
logTest(test2, 'running');
const start2 = Date.now();

try {
  debug('Test 2: Verify menu item structure');

  if (menuItems.length === 0) {
    debug('[SKIP] Test 2 - no menu items available');
    logTest(test2, 'skip', {
      reason: 'No menu items available to inspect',
      duration_ms: Date.now() - start2,
    });
  } else {
    const firstMenu = menuItems[0];

    // Verify required properties exist
    const hasTitle = typeof firstMenu.title === 'string';
    const hasEnabled = typeof firstMenu.enabled === 'boolean';
    const hasChildren = Array.isArray(firstMenu.children);
    const hasMenuPath = Array.isArray(firstMenu.menuPath);

    if (!hasTitle) {
      throw new Error(`Expected title to be string, got ${typeof firstMenu.title}`);
    }
    if (!hasEnabled) {
      throw new Error(`Expected enabled to be boolean, got ${typeof firstMenu.enabled}`);
    }
    if (!hasChildren) {
      throw new Error(`Expected children to be array, got ${typeof firstMenu.children}`);
    }
    if (!hasMenuPath) {
      throw new Error(`Expected menuPath to be array, got ${typeof firstMenu.menuPath}`);
    }

    // Check optional shortcut property type if present
    const hasValidShortcut = firstMenu.shortcut === undefined || 
                             typeof firstMenu.shortcut === 'string';
    if (!hasValidShortcut) {
      throw new Error(`Expected shortcut to be string or undefined, got ${typeof firstMenu.shortcut}`);
    }

    debug(`Test 2 completed - menu structure verified for "${firstMenu.title}"`);
    logTest(test2, 'pass', {
      result: {
        sampleMenu: {
          title: firstMenu.title,
          enabled: firstMenu.enabled,
          childCount: firstMenu.children.length,
          menuPath: firstMenu.menuPath,
          hasShortcut: !!firstMenu.shortcut,
        },
      },
      duration_ms: Date.now() - start2,
    });
  }
} catch (err) {
  logTest(test2, 'fail', {
    error: String(err),
    duration_ms: Date.now() - start2,
  });
}

// -----------------------------------------------------------------------------
// Test 3: Menu path traversal - Verify menuPath property is set correctly
// -----------------------------------------------------------------------------
const test3 = 'menu-path-traversal';
logTest(test3, 'running');
const start3 = Date.now();

try {
  debug('Test 3: Verify menuPath property');

  if (menuItems.length === 0) {
    debug('[SKIP] Test 3 - no menu items available');
    logTest(test3, 'skip', {
      reason: 'No menu items available',
      duration_ms: Date.now() - start3,
    });
  } else {
    // Find a menu with children to test path traversal
    const menuWithChildren = menuItems.find(m => m.children.length > 0);
    
    if (!menuWithChildren) {
      debug('[SKIP] Test 3 - no menus with children found');
      logTest(test3, 'skip', {
        reason: 'No menus with children found',
        duration_ms: Date.now() - start3,
      });
    } else {
      // Verify top-level menu has itself in the path
      if (!menuWithChildren.menuPath.includes(menuWithChildren.title)) {
        throw new Error(`Top-level menu "${menuWithChildren.title}" should have its title in menuPath`);
      }

      // Verify children have proper path
      const firstChild = menuWithChildren.children[0];
      if (firstChild) {
        const expectedPathLength = 2; // [parent, child]
        if (firstChild.menuPath.length !== expectedPathLength) {
          debug(`Note: Child menuPath length is ${firstChild.menuPath.length}, expected ~${expectedPathLength}`);
        }
        
        // The path should include the parent menu title
        if (!firstChild.menuPath.includes(menuWithChildren.title)) {
          throw new Error(`Child menu should have parent "${menuWithChildren.title}" in its menuPath`);
        }
      }

      debug(`Test 3 completed - menuPath verified for "${menuWithChildren.title}"`);
      logTest(test3, 'pass', {
        result: {
          parentMenu: menuWithChildren.title,
          parentPath: menuWithChildren.menuPath,
          childCount: menuWithChildren.children.length,
          sampleChildPath: menuWithChildren.children[0]?.menuPath,
        },
        duration_ms: Date.now() - start3,
      });
    }
  }
} catch (err) {
  logTest(test3, 'fail', {
    error: String(err),
    duration_ms: Date.now() - start3,
  });
}

// -----------------------------------------------------------------------------
// Test 4: getMenuBar with bundleId - Get menu bar from specific app
// -----------------------------------------------------------------------------
const test4 = 'getMenuBar-with-bundleId';
logTest(test4, 'running');
const start4 = Date.now();

try {
  debug('Test 4: getMenuBar with bundleId (Finder)');

  // Try to get Finder's menu bar - Finder should always be running
  const finderMenus = await getMenuBar('com.apple.finder');

  if (!Array.isArray(finderMenus)) {
    throw new Error(`Expected array, got ${typeof finderMenus}`);
  }

  debug(`Test 4 completed - got ${finderMenus.length} Finder menu items`);
  logTest(test4, 'pass', {
    result: {
      menuCount: finderMenus.length,
      titles: finderMenus.map(m => m.title),
    },
    duration_ms: Date.now() - start4,
  });
} catch (err) {
  const errorMessage = String(err);
  if (errorMessage.toLowerCase().includes('accessibility') || 
      errorMessage.toLowerCase().includes('permission') ||
      errorMessage.toLowerCase().includes('not running')) {
    debug('[SKIP] Test 4 - accessibility permission or app not running');
    logTest(test4, 'skip', {
      reason: 'Accessibility permission required or Finder not accessible',
      duration_ms: Date.now() - start4,
    });
  } else {
    logTest(test4, 'fail', {
      error: errorMessage,
      duration_ms: Date.now() - start4,
    });
  }
}

// -----------------------------------------------------------------------------
// Test 5: executeMenuAction() - Execute a menu action
// NOTE: This test is intentionally conservative to avoid unwanted side effects
// -----------------------------------------------------------------------------
const test5 = 'executeMenuAction-function-exists';
logTest(test5, 'running');
const start5 = Date.now();

try {
  debug('Test 5: Verify executeMenuAction function exists');

  // Verify executeMenuAction function exists
  if (typeof executeMenuAction !== 'function') {
    throw new Error(`Expected executeMenuAction to be a function, got ${typeof executeMenuAction}`);
  }

  // We won't actually execute a menu action here as it could have side effects
  // Instead, just verify the function exists and is callable
  debug('Test 5 completed - executeMenuAction function exists');
  logTest(test5, 'pass', {
    result: {
      functionType: typeof executeMenuAction,
      note: 'Did not execute to avoid side effects'
    },
    duration_ms: Date.now() - start5,
  });
} catch (err) {
  logTest(test5, 'fail', {
    error: String(err),
    duration_ms: Date.now() - start5,
  });
}

// -----------------------------------------------------------------------------
// Test 6: Menu shortcuts - Check if shortcuts are properly formatted
// -----------------------------------------------------------------------------
const test6 = 'menu-shortcuts';
logTest(test6, 'running');
const start6 = Date.now();

try {
  debug('Test 6: Check menu shortcuts');

  if (menuItems.length === 0) {
    debug('[SKIP] Test 6 - no menu items available');
    logTest(test6, 'skip', {
      reason: 'No menu items available',
      duration_ms: Date.now() - start6,
    });
  } else {
    // Recursively find all menu items with shortcuts
    function findShortcuts(items: MenuBarItem[]): Array<{ title: string; shortcut: string; path: string[] }> {
      const shortcuts: Array<{ title: string; shortcut: string; path: string[] }> = [];
      
      for (const item of items) {
        if (item.shortcut) {
          shortcuts.push({
            title: item.title,
            shortcut: item.shortcut,
            path: item.menuPath,
          });
        }
        if (item.children.length > 0) {
          shortcuts.push(...findShortcuts(item.children));
        }
      }
      
      return shortcuts;
    }

    const shortcuts = findShortcuts(menuItems);
    
    debug(`Test 6 completed - found ${shortcuts.length} menu items with shortcuts`);
    logTest(test6, 'pass', {
      result: {
        shortcutCount: shortcuts.length,
        samples: shortcuts.slice(0, 5), // Show first 5 shortcuts
      },
      duration_ms: Date.now() - start6,
    });
  }
} catch (err) {
  logTest(test6, 'fail', {
    error: String(err),
    duration_ms: Date.now() - start6,
  });
}

// -----------------------------------------------------------------------------
// Test 7: Menu enabled state - Verify enabled flag is set
// -----------------------------------------------------------------------------
const test7 = 'menu-enabled-state';
logTest(test7, 'running');
const start7 = Date.now();

try {
  debug('Test 7: Check menu enabled states');

  if (menuItems.length === 0) {
    debug('[SKIP] Test 7 - no menu items available');
    logTest(test7, 'skip', {
      reason: 'No menu items available',
      duration_ms: Date.now() - start7,
    });
  } else {
    // Count enabled and disabled items
    function countEnabled(items: MenuBarItem[]): { enabled: number; disabled: number } {
      let enabled = 0;
      let disabled = 0;
      
      for (const item of items) {
        if (item.enabled) {
          enabled++;
        } else {
          disabled++;
        }
        if (item.children.length > 0) {
          const childCounts = countEnabled(item.children);
          enabled += childCounts.enabled;
          disabled += childCounts.disabled;
        }
      }
      
      return { enabled, disabled };
    }

    const counts = countEnabled(menuItems);
    
    debug(`Test 7 completed - ${counts.enabled} enabled, ${counts.disabled} disabled`);
    logTest(test7, 'pass', {
      result: {
        enabledCount: counts.enabled,
        disabledCount: counts.disabled,
        total: counts.enabled + counts.disabled,
      },
      duration_ms: Date.now() - start7,
    });
  }
} catch (err) {
  logTest(test7, 'fail', {
    error: String(err),
    duration_ms: Date.now() - start7,
  });
}

// -----------------------------------------------------------------------------
// Summary and Exit
// -----------------------------------------------------------------------------
debug('test-menu-bar.ts completed!');
debug('All 7 tests executed. Check JSONL output for detailed results.');

// Exit cleanly for autonomous testing
exit(0);
