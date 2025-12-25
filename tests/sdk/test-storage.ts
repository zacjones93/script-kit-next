/**
 * TIER 5B Test: Storage, Path, and Utility Functions
 * 
 * This test demonstrates:
 * - Path utilities (home, kenvPath, kitPath, tmpPath)
 * - File utilities (isFile, isDir, isBin)
 * - Database and store (db, store, memoryMap)
 * - Browser/App utilities (browse, editFile, run, inspect)
 * 
 * Run with: bun scripts/kit-sdk.ts tests/sdk/test-storage.ts
 */

import '../../scripts/kit-sdk';

// =============================================================================
// Path Utilities Tests
// =============================================================================

async function testPathUtilities() {
  console.log('=== Testing Path Utilities ===\n');
  
  // Test home()
  const homePath = home();
  console.log(`home(): ${homePath}`);
  
  const downloadsPath = home('Downloads', 'file.txt');
  console.log(`home('Downloads', 'file.txt'): ${downloadsPath}`);
  
  // Test kenvPath()
  const kenvRoot = kenvPath();
  console.log(`kenvPath(): ${kenvRoot}`);
  
  const scriptsPath = kenvPath('scripts', 'hello.ts');
  console.log(`kenvPath('scripts', 'hello.ts'): ${scriptsPath}`);
  
  // Test kitPath()
  const kitRoot = kitPath();
  console.log(`kitPath(): ${kitRoot}`);
  
  const dbPath = kitPath('db', 'scripts.json');
  console.log(`kitPath('db', 'scripts.json'): ${dbPath}`);
  
  // Test tmpPath()
  const tmpRoot = tmpPath();
  console.log(`tmpPath(): ${tmpRoot}`);
  
  const tmpFile = tmpPath('output.txt');
  console.log(`tmpPath('output.txt'): ${tmpFile}`);
  
  console.log('');
}

// =============================================================================
// File Utilities Tests
// =============================================================================

async function testFileUtilities() {
  console.log('=== Testing File Utilities ===\n');
  
  // Test isFile() - check if this test file exists
  const thisFile = import.meta.path || process.argv[1];
  const isThisFile = await isFile(thisFile);
  console.log(`isFile('${thisFile}'): ${isThisFile}`);
  
  // Test isFile() - check non-existent file
  const nonExistent = '/this/does/not/exist.txt';
  const isNonExistent = await isFile(nonExistent);
  console.log(`isFile('${nonExistent}'): ${isNonExistent}`);
  
  // Test isDir() - check home directory
  const homeDir = home();
  const isHomeDir = await isDir(homeDir);
  console.log(`isDir('${homeDir}'): ${isHomeDir}`);
  
  // Test isDir() - check a file (should be false)
  const isFileDirTest = await isDir(thisFile);
  console.log(`isDir('${thisFile}'): ${isFileDirTest}`);
  
  // Test isBin() - check /bin/ls
  const binLs = '/bin/ls';
  const isBinLs = await isBin(binLs);
  console.log(`isBin('${binLs}'): ${isBinLs}`);
  
  // Test isBin() - check this file (should be false)
  const isBinThisFile = await isBin(thisFile);
  console.log(`isBin('${thisFile}'): ${isBinThisFile}`);
  
  console.log('');
}

// =============================================================================
// Memory Map Tests (in-process, no messages)
// =============================================================================

async function testMemoryMap() {
  console.log('=== Testing Memory Map ===\n');
  
  // Set some values
  memoryMap.set('counter', 42);
  memoryMap.set('user', { name: 'John', age: 30 });
  memoryMap.set('tags', ['javascript', 'typescript', 'bun']);
  
  // Get values
  console.log(`memoryMap.get('counter'): ${memoryMap.get('counter')}`);
  console.log(`memoryMap.get('user'): ${JSON.stringify(memoryMap.get('user'))}`);
  console.log(`memoryMap.get('tags'): ${JSON.stringify(memoryMap.get('tags'))}`);
  
  // Get non-existent key
  console.log(`memoryMap.get('nonexistent'): ${memoryMap.get('nonexistent')}`);
  
  // Delete a key
  const deleted = memoryMap.delete('counter');
  console.log(`memoryMap.delete('counter'): ${deleted}`);
  console.log(`memoryMap.get('counter') after delete: ${memoryMap.get('counter')}`);
  
  // Delete non-existent key
  const deletedNonExistent = memoryMap.delete('nonexistent');
  console.log(`memoryMap.delete('nonexistent'): ${deletedNonExistent}`);
  
  // Clear all
  memoryMap.clear();
  console.log(`After clear, memoryMap.get('user'): ${memoryMap.get('user')}`);
  
  console.log('');
}

// =============================================================================
// Interactive Tests (require GPUI)
// =============================================================================

async function testDatabaseInteractive() {
  console.log('=== Testing Database (requires GPUI) ===\n');
  
  // Create/load a database
  const database = await db({ 
    count: 0, 
    items: [] 
  });
  
  console.log('Initial database data:', JSON.stringify(database.data, null, 2));
  
  // Modify the data
  if (typeof database.data === 'object' && database.data !== null) {
    const data = database.data as { count: number; items: string[] };
    data.count += 1;
    data.items.push(`Item ${Date.now()}`);
  }
  
  // Write changes
  await database.write();
  console.log('Database saved!');
}

async function testStoreInteractive() {
  console.log('=== Testing Store (requires GPUI) ===\n');
  
  // Set a value
  await store.set('lastRun', new Date().toISOString());
  console.log('Set lastRun to current time');
  
  // Get a value
  const lastRun = await store.get('lastRun');
  console.log(`store.get('lastRun'): ${lastRun}`);
  
  // Get non-existent key
  const nonExistent = await store.get('nonexistent');
  console.log(`store.get('nonexistent'): ${nonExistent}`);
}

async function testBrowserAppInteractive() {
  console.log('=== Testing Browser/App Utilities (requires GPUI) ===\n');
  
  // Open URL in browser
  await browse('https://scriptkit.com');
  console.log('Opened https://scriptkit.com in browser');
  
  // Open file in editor
  // await editFile(home('.zshrc'));
  // console.log('Opened ~/.zshrc in editor');
  
  // Inspect data
  await inspect({
    test: 'This is a test object',
    nested: {
      value: 42,
      array: [1, 2, 3]
    }
  });
  console.log('Sent data to inspect');
}

// =============================================================================
// Main
// =============================================================================

async function main() {
  console.log('TIER 5B: Storage and Path Utilities Test\n');
  console.log('=========================================\n');
  
  // These tests run without GPUI (pure JS)
  await testPathUtilities();
  await testFileUtilities();
  await testMemoryMap();
  
  // Check if we're running in GPUI context
  const args = process.argv.slice(2);
  if (args.includes('--interactive')) {
    console.log('Running interactive tests (requires GPUI)...\n');
    await testDatabaseInteractive();
    await testStoreInteractive();
    await testBrowserAppInteractive();
  } else {
    console.log('Skipping interactive tests (pass --interactive to run them)\n');
    console.log('Interactive tests require GPUI connection for:');
    console.log('  - db() - JSON file database');
    console.log('  - store.get/set() - Key-value store');
    console.log('  - browse() - Open URL in browser');
    console.log('  - editFile() - Open file in editor');
    console.log('  - run() - Run another script');
    console.log('  - inspect() - Pretty-print data');
  }
  
  console.log('\nAll non-interactive tests completed!');
}

main().catch(console.error);
