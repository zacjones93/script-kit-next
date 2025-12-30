// Name: SDK Test - Actions API
// Description: Tests setActions() and actionTriggered message handling

import '../../scripts/kit-sdk';

interface TestResult {
  test: string;
  status: 'running' | 'pass' | 'fail' | 'skip';
  timestamp: string;
  result?: unknown;
  error?: string;
  reason?: string;
  duration_ms?: number;
}

function logTest(name: string, status: TestResult['status'], extra?: Partial<TestResult>) {
  const result: TestResult = {
    test: name,
    status,
    timestamp: new Date().toISOString(),
    ...extra
  };
  console.log(JSON.stringify(result));
}

// Capture messages sent to stdout
const sentMessages: unknown[] = [];
const originalStdoutWrite = (process as any).stdout.write.bind((process as any).stdout);
(process as any).stdout.write = (chunk: any, ...args: any[]) => {
  try {
    const parsed = JSON.parse(chunk.toString().trim());
    sentMessages.push(parsed);
  } catch {
    // Not JSON, pass through
  }
  return originalStdoutWrite(chunk, ...args);
};

// Declare globals that will be added by the SDK
declare global {
  function setActions(actions: Action[]): Promise<void>;
  var __kitActionsMap: Map<string, Action>;
  function __handleActionTriggered(msg: { type: string; action: string; input: string; state: any }): Promise<void>;
}

interface Action {
  name: string;
  description?: string;
  shortcut?: string;
  value?: string;
  onAction?: (input: string, state: any) => void | Promise<void>;
  visible?: boolean;
  close?: boolean;
}

async function runTests() {
  // =============================================================================
  // Test 1: setActions sends correct message format
  // =============================================================================
  const test1Name = 'setActions-message-format';
  logTest(test1Name, 'running');
  const start1 = Date.now();

  try {
    // Clear previous messages
    sentMessages.length = 0;
    
    // Define test actions with various properties
    const testActions: Action[] = [
      {
        name: 'copy',
        description: 'Copy to clipboard',
        shortcut: 'cmd+c',
        value: 'copy-value',
        onAction: (input: string, state: any) => {
          console.error('[TEST] copy action called');
        },
        visible: true,
        close: true,
      },
      {
        name: 'paste',
        shortcut: 'cmd+v',
        // No onAction - should use value fallback
        value: 'paste-value',
      },
      {
        name: 'search',
        description: 'Search for something',
        onAction: async (input: string, state: any) => {
          console.error('[TEST] search action called with:', input);
        },
        visible: false,
      },
    ];
    
    // Call setActions
    await (globalThis as any).setActions(testActions);
    
    // Find the setActions message
    const setActionsMsg = sentMessages.find((m: any) => m.type === 'setActions') as any;
    
    if (!setActionsMsg) {
      logTest(test1Name, 'fail', { 
        error: 'setActions message not found in sent messages',
        result: sentMessages,
        duration_ms: Date.now() - start1 
      });
    } else {
      // Verify message format
      const actions = setActionsMsg.actions;
      
      // Check that functions are NOT included (stripped)
      const hasAnyFunction = actions.some((a: any) => typeof a.onAction === 'function');
      if (hasAnyFunction) {
        logTest(test1Name, 'fail', { 
          error: 'onAction function should be stripped from message',
          duration_ms: Date.now() - start1 
        });
      } else {
        // Check hasAction boolean is set correctly
        const copyAction = actions.find((a: any) => a.name === 'copy');
        const pasteAction = actions.find((a: any) => a.name === 'paste');
        const searchAction = actions.find((a: any) => a.name === 'search');
        
        if (copyAction?.hasAction !== true) {
          logTest(test1Name, 'fail', { 
            error: 'copy action should have hasAction=true',
            result: copyAction,
            duration_ms: Date.now() - start1 
          });
        } else if (pasteAction?.hasAction !== false) {
          logTest(test1Name, 'fail', { 
            error: 'paste action should have hasAction=false',
            result: pasteAction,
            duration_ms: Date.now() - start1 
          });
        } else if (searchAction?.hasAction !== true) {
          logTest(test1Name, 'fail', { 
            error: 'search action should have hasAction=true',
            result: searchAction,
            duration_ms: Date.now() - start1 
          });
        } else {
          // Verify other properties are preserved
          if (copyAction.shortcut !== 'cmd+c' || 
              copyAction.description !== 'Copy to clipboard' ||
              copyAction.visible !== true ||
              copyAction.close !== true) {
            logTest(test1Name, 'fail', { 
              error: 'Action properties not preserved correctly',
              result: copyAction,
              duration_ms: Date.now() - start1 
            });
          } else {
            logTest(test1Name, 'pass', { 
              result: { actionsCount: actions.length, hasActionValues: actions.map((a: any) => a.hasAction) },
              duration_ms: Date.now() - start1 
            });
          }
        }
      }
    }
  } catch (err) {
    logTest(test1Name, 'fail', { error: String(err), duration_ms: Date.now() - start1 });
  }

  // =============================================================================
  // Test 2: Actions are stored in global map
  // =============================================================================
  const test2Name = 'actions-stored-in-map';
  logTest(test2Name, 'running');
  const start2 = Date.now();

  try {
    // Access the global actions map
    const actionsMap = (globalThis as any).__kitActionsMap as Map<string, any>;
    
    if (!actionsMap) {
      logTest(test2Name, 'fail', { 
        error: '__kitActionsMap not found on global',
        duration_ms: Date.now() - start2 
      });
    } else if (!(actionsMap instanceof Map)) {
      logTest(test2Name, 'fail', { 
        error: '__kitActionsMap is not a Map',
        result: typeof actionsMap,
        duration_ms: Date.now() - start2 
      });
    } else {
      // Check that our test actions were stored
      const copyAction = actionsMap.get('copy');
      const pasteAction = actionsMap.get('paste');
      const searchAction = actionsMap.get('search');
      
      if (!copyAction || !pasteAction || !searchAction) {
        logTest(test2Name, 'fail', { 
          error: 'Not all actions found in map',
          result: { 
            hasCopy: !!copyAction, 
            hasPaste: !!pasteAction, 
            hasSearch: !!searchAction,
            mapSize: actionsMap.size 
          },
          duration_ms: Date.now() - start2 
        });
      } else if (typeof copyAction.onAction !== 'function') {
        logTest(test2Name, 'fail', { 
          error: 'onAction function not preserved in map',
          result: typeof copyAction.onAction,
          duration_ms: Date.now() - start2 
        });
      } else {
        logTest(test2Name, 'pass', { 
          result: { mapSize: actionsMap.size, hasOnAction: typeof copyAction.onAction },
          duration_ms: Date.now() - start2 
        });
      }
    }
  } catch (err) {
    logTest(test2Name, 'fail', { error: String(err), duration_ms: Date.now() - start2 });
  }

  // =============================================================================
  // Test 3: actionTriggered calls onAction handler
  // =============================================================================
  const test3Name = 'action-triggered-calls-handler';
  logTest(test3Name, 'running');
  const start3 = Date.now();

  try {
    let handlerCalled = false;
    let handlerInput = '';
    let handlerState: any = null;
    
    // Set up a new action with a trackable handler
    await (globalThis as any).setActions([
      {
        name: 'test-handler',
        onAction: (input: string, state: any) => {
          handlerCalled = true;
          handlerInput = input;
          handlerState = state;
        },
      },
    ]);
    
    // Simulate receiving an actionTriggered message
    // We need to trigger the stdin handler with a properly formatted message
    const testMessage = {
      type: 'actionTriggered',
      action: 'test-handler',
      input: 'test-input-value',
      state: { selectedIndex: 5, filter: 'foo' },
    };
    
    // Manually call the internal handler (simulating stdin message)
    // The SDK processes stdin messages and should handle actionTriggered
    if ((globalThis as any).__handleActionTriggered) {
      await (globalThis as any).__handleActionTriggered(testMessage);
      
      if (!handlerCalled) {
        logTest(test3Name, 'fail', { 
          error: 'Handler was not called',
          duration_ms: Date.now() - start3 
        });
      } else if (handlerInput !== 'test-input-value') {
        logTest(test3Name, 'fail', { 
          error: `Handler received wrong input: ${handlerInput}`,
          duration_ms: Date.now() - start3 
        });
      } else if (handlerState?.selectedIndex !== 5) {
        logTest(test3Name, 'fail', { 
          error: 'Handler received wrong state',
          result: handlerState,
          duration_ms: Date.now() - start3 
        });
      } else {
        logTest(test3Name, 'pass', { 
          result: { handlerCalled, handlerInput, handlerState },
          duration_ms: Date.now() - start3 
        });
      }
    } else {
      // Fallback: check if we can find another way to test
      // For now, skip this test if the internal handler isn't exposed
      logTest(test3Name, 'skip', { 
        reason: '__handleActionTriggered not exposed - will test via stdin in integration test',
        duration_ms: Date.now() - start3 
      });
    }
  } catch (err) {
    logTest(test3Name, 'fail', { error: String(err), duration_ms: Date.now() - start3 });
  }

  // =============================================================================
  // Test 4: actionTriggered with no handler calls submit with value
  // =============================================================================
  const test4Name = 'action-triggered-fallback-submit';
  logTest(test4Name, 'running');
  const start4 = Date.now();

  try {
    // Clear previous messages
    sentMessages.length = 0;
    
    // Set up an action without onAction handler, but with value
    await (globalThis as any).setActions([
      {
        name: 'no-handler-action',
        value: 'fallback-value',
      },
    ]);
    
    // Simulate receiving actionTriggered for this action
    const testMessage = {
      type: 'actionTriggered',
      action: 'no-handler-action',
      input: 'some-input',
      state: {},
    };
    
    // Try to call the internal handler
    if ((globalThis as any).__handleActionTriggered) {
      await (globalThis as any).__handleActionTriggered(testMessage);
      
      // Check if submit was called with the fallback value
      const submitMsg = sentMessages.find((m: any) => m.type === 'forceSubmit') as any;
      
      if (!submitMsg) {
        logTest(test4Name, 'fail', { 
          error: 'submit not called when action has no handler',
          result: sentMessages,
          duration_ms: Date.now() - start4 
        });
      } else if (submitMsg.value !== 'fallback-value') {
        logTest(test4Name, 'fail', { 
          error: `submit called with wrong value: ${submitMsg.value}`,
          duration_ms: Date.now() - start4 
        });
      } else {
        logTest(test4Name, 'pass', { 
          result: { submittedValue: submitMsg.value },
          duration_ms: Date.now() - start4 
        });
      }
    } else {
      logTest(test4Name, 'skip', { 
        reason: '__handleActionTriggered not exposed',
        duration_ms: Date.now() - start4 
      });
    }
  } catch (err) {
    logTest(test4Name, 'fail', { error: String(err), duration_ms: Date.now() - start4 });
  }

  // =============================================================================
  // Test 5: setActions clears previous actions
  // =============================================================================
  const test5Name = 'setActions-clears-previous';
  logTest(test5Name, 'running');
  const start5 = Date.now();

  try {
    // First, set some actions
    await (globalThis as any).setActions([
      { name: 'action-a' },
      { name: 'action-b' },
    ]);
    
    const actionsMap = (globalThis as any).__kitActionsMap as Map<string, any>;
    const sizeAfterFirst = actionsMap.size;
    
    // Now set different actions
    await (globalThis as any).setActions([
      { name: 'action-c' },
    ]);
    
    const sizeAfterSecond = actionsMap.size;
    const hasActionA = actionsMap.has('action-a');
    const hasActionC = actionsMap.has('action-c');
    
    if (sizeAfterFirst !== 2) {
      logTest(test5Name, 'fail', { 
        error: `First setActions should have 2 actions, got ${sizeAfterFirst}`,
        duration_ms: Date.now() - start5 
      });
    } else if (sizeAfterSecond !== 1) {
      logTest(test5Name, 'fail', { 
        error: `Second setActions should have 1 action, got ${sizeAfterSecond}`,
        duration_ms: Date.now() - start5 
      });
    } else if (hasActionA) {
      logTest(test5Name, 'fail', { 
        error: 'action-a should have been cleared',
        duration_ms: Date.now() - start5 
      });
    } else if (!hasActionC) {
      logTest(test5Name, 'fail', { 
        error: 'action-c should be present',
        duration_ms: Date.now() - start5 
      });
    } else {
      logTest(test5Name, 'pass', { 
        result: { sizeAfterFirst, sizeAfterSecond, hasActionA, hasActionC },
        duration_ms: Date.now() - start5 
      });
    }
  } catch (err) {
    logTest(test5Name, 'fail', { error: String(err), duration_ms: Date.now() - start5 });
  }

  // Exit cleanly
  console.error('[TEST] Actions SDK tests complete');
  (process as any).exit(0);
}

runTests();
