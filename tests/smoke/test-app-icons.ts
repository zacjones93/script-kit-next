// Name: Test App Icons Display
// Description: Verify macOS app icons are displayed in the list

import '../../scripts/kit-sdk';

console.error('[SMOKE] Testing app icon display...');

// This test just shows the main list which includes apps with icons
// A user should see real app icons (not just emoji) for installed apps

await arg("Search for apps to see their icons", [
    {
        name: "Safari",
        value: "safari",
        description: "Should show Safari icon (not 📱)",
    },
    {
        name: "Finder", 
        value: "finder",
        description: "Should show Finder icon (not 📱)",
    },
    {
        name: "Calculator",
        value: "calculator",
        description: "Should show Calculator icon (not 📱)",
    },
    {
        name: "Type an app name to search",
        value: "search",
        description: "Apps from /Applications should have real icons",
    }
]);

console.error('[SMOKE] App icon test complete');
