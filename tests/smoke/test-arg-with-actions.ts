// Name: Test arg() with actions
// Description: Tests the actions parameter for arg()

import '../../scripts/kit-sdk';

console.error('[SMOKE] test-arg-with-actions starting...');

// Test 1: Basic actions with arg() 3rd argument
const choices = ['Apple', 'Banana', 'Cherry'];

const actions = [
  {
    name: 'Copy Value',
    shortcut: 'cmd+c',
    onAction: (input: string) => {
      console.error('[SMOKE] Copy action triggered with input:', input);
    },
  },
  {
    name: 'Preview',
    shortcut: 'cmd+p',
    onAction: (input: string) => {
      console.error('[SMOKE] Preview action triggered');
    },
  },
  {
    name: 'Open in Editor',
    description: 'Opens the selected item in default editor',
    shortcut: 'cmd+e',
    onAction: (input: string) => {
      console.error('[SMOKE] Open in Editor action triggered');
    },
  },
];

console.error('[SMOKE] Showing arg prompt with 3 choices and 3 actions...');
console.error('[SMOKE] Expected: Actions button visible in header, Cmd+K shows actions panel');

const result = await arg('Pick a fruit (press Cmd+K for actions):', choices, actions);

console.error('[SMOKE] Result:', result);
console.error('[SMOKE] test-arg-with-actions completed successfully');

process.exit(0);
