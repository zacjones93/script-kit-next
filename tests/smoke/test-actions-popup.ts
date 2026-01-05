import '../../scripts/kit-sdk';

// Test script to verify actions popup behavior
// This should trigger the SDK actions path which may be using inline overlay

const result = await arg({
  placeholder: "Pick a fruit",
  actions: [
    { name: "Action 1", shortcut: "cmd+1" },
    { name: "Action 2", shortcut: "cmd+2" },
  ]
}, ["Apple", "Banana", "Cherry"]);

console.log("Selected:", result);
