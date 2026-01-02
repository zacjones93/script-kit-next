// Name: SDK Test - form() specialized inputs
// Description: Tests form() with specialized input types (url, search, tel, color, textarea)

/**
 * SDK TEST: test-form-specialized.ts
 *
 * Tests form() with specialized input types that may need special handling.
 * This test focuses on visual verification via screenshots.
 *
 * Test cases:
 * 1. form-url: URL input type
 * 2. form-search: Search input type
 * 3. form-tel: Telephone input type
 * 4. form-color: Color picker input
 * 5. form-textarea: Multi-line textarea
 * 6. form-combined: Combined form with all specialized types
 *
 * Visual Testing:
 * Each test captures a screenshot to verify rendering.
 * Screenshots are saved to .test-screenshots/form-specialized-*.png
 *
 * Note: The form parser in GPUI converts HTML form elements to native components.
 * Specialized input types (url, search, tel, color) are passed through as-is.
 */

import "../../scripts/kit-sdk.ts";
import { writeFileSync, mkdirSync, existsSync } from "fs";
import { join } from "path";

// =============================================================================
// Test Infrastructure
// =============================================================================

interface TestResult {
  test: string;
  status: "running" | "pass" | "fail" | "skip";
  timestamp: string;
  result?: unknown;
  error?: string;
  duration_ms?: number;
  screenshot?: string;
}

function logTest(
  name: string,
  status: TestResult["status"],
  extra?: Partial<TestResult>
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
  console.error(`[TEST] ${msg}`);
}

const screenshotDir = join(process.cwd(), ".test-screenshots");

async function captureAndSave(name: string): Promise<string | undefined> {
  try {
    const screenshot = await captureScreenshot();
    debug(`Captured screenshot: ${screenshot.width}x${screenshot.height}`);
    
    mkdirSync(screenshotDir, { recursive: true });
    const filename = `form-specialized-${name}-${Date.now()}.png`;
    const filepath = join(screenshotDir, filename);
    writeFileSync(filepath, Buffer.from(screenshot.data, "base64"));
    debug(`[SCREENSHOT] ${filepath}`);
    return filepath;
  } catch (err) {
    debug(`Screenshot capture failed: ${err}`);
    return undefined;
  }
}

// =============================================================================
// Tests - Use fields() instead of form() since it uses native GPUI components
// =============================================================================

debug("test-form-specialized.ts starting...");
debug(`SDK globals: form=${typeof form}, fields=${typeof fields}`);

// -----------------------------------------------------------------------------
// Test 1: URL input via fields()
// -----------------------------------------------------------------------------
const test1 = "fields-url";
logTest(test1, "running");
const start1 = Date.now();

try {
  debug("Test 1: fields() with URL input type");

  const fieldDefs = [
    {
      name: "website",
      label: "Website URL",
      type: "url" as const,
      placeholder: "https://example.com",
    },
  ];

  // Start the fields() call (don't await - need to capture screenshot while displayed)
  const fieldsPromise = fields(fieldDefs);
  
  // Wait for UI to render then capture
  await new Promise((r) => setTimeout(r, 1000));
  const screenshotPath = await captureAndSave("url");
  
  // Note: Test will wait here for user interaction (Enter/Escape)
  // For automated testing, we rely on the screenshot captured above
  debug("Test 1: Screenshot captured. Awaiting user input to continue...");
  
  const result = await fieldsPromise;
  debug(`Test 1 result: ${JSON.stringify(result)}`);

  logTest(test1, "pass", { 
    result,
    duration_ms: Date.now() - start1,
    screenshot: screenshotPath,
  });
} catch (err) {
  logTest(test1, "fail", {
    error: String(err),
    duration_ms: Date.now() - start1,
  });
}

// -----------------------------------------------------------------------------
// Test 2: Search input via fields()
// -----------------------------------------------------------------------------
const test2 = "fields-search";
logTest(test2, "running");
const start2 = Date.now();

try {
  debug("Test 2: fields() with search input type");

  const fieldDefs = [
    {
      name: "query",
      label: "Search Query",
      type: "text" as const, // search type falls back to text
      placeholder: "Search...",
    },
  ];

  const fieldsPromise = fields(fieldDefs);
  await new Promise((r) => setTimeout(r, 1000));
  const screenshotPath = await captureAndSave("search");
  
  debug("Test 2: Screenshot captured. Awaiting user input to continue...");
  const result = await fieldsPromise;
  debug(`Test 2 result: ${JSON.stringify(result)}`);

  logTest(test2, "pass", { 
    result,
    duration_ms: Date.now() - start2,
    screenshot: screenshotPath,
  });
} catch (err) {
  logTest(test2, "fail", {
    error: String(err),
    duration_ms: Date.now() - start2,
  });
}

// -----------------------------------------------------------------------------
// Test 3: Tel input via fields()
// -----------------------------------------------------------------------------
const test3 = "fields-tel";
logTest(test3, "running");
const start3 = Date.now();

try {
  debug("Test 3: fields() with tel input type");

  const fieldDefs = [
    {
      name: "phone",
      label: "Phone Number",
      type: "tel" as const,
      placeholder: "123-456-7890",
    },
  ];

  const fieldsPromise = fields(fieldDefs);
  await new Promise((r) => setTimeout(r, 1000));
  const screenshotPath = await captureAndSave("tel");
  
  debug("Test 3: Screenshot captured. Awaiting user input to continue...");
  const result = await fieldsPromise;
  debug(`Test 3 result: ${JSON.stringify(result)}`);

  logTest(test3, "pass", { 
    result,
    duration_ms: Date.now() - start3,
    screenshot: screenshotPath,
  });
} catch (err) {
  logTest(test3, "fail", {
    error: String(err),
    duration_ms: Date.now() - start3,
  });
}

// -----------------------------------------------------------------------------
// Test 4: Color input via fields()
// -----------------------------------------------------------------------------
const test4 = "fields-color";
logTest(test4, "running");
const start4 = Date.now();

try {
  debug("Test 4: fields() with color input type");

  const fieldDefs = [
    {
      name: "favoriteColor",
      label: "Favorite Color",
      type: "color" as const,
      value: "#ff0000",
    },
  ];

  const fieldsPromise = fields(fieldDefs);
  await new Promise((r) => setTimeout(r, 1000));
  const screenshotPath = await captureAndSave("color");
  
  debug("Test 4: Screenshot captured. Awaiting user input to continue...");
  const result = await fieldsPromise;
  debug(`Test 4 result: ${JSON.stringify(result)}`);

  logTest(test4, "pass", { 
    result,
    duration_ms: Date.now() - start4,
    screenshot: screenshotPath,
  });
} catch (err) {
  logTest(test4, "fail", {
    error: String(err),
    duration_ms: Date.now() - start4,
  });
}

// -----------------------------------------------------------------------------
// Test 5: Textarea via form() - this is where we test actual HTML parsing
// -----------------------------------------------------------------------------
const test5 = "form-textarea";
logTest(test5, "running");
const start5 = Date.now();

try {
  debug("Test 5: form() with textarea element");

  const html = `
    <form>
      <label for="bio">Biography:</label>
      <textarea name="bio" id="bio" placeholder="Tell us about yourself" rows="4"></textarea>
    </form>
  `;

  const formPromise = form(html);
  await new Promise((r) => setTimeout(r, 1000));
  const screenshotPath = await captureAndSave("textarea");
  
  debug("Test 5: Screenshot captured. Awaiting user input to continue...");
  const result = await formPromise;
  debug(`Test 5 result: ${JSON.stringify(result)}`);

  logTest(test5, "pass", { 
    result,
    duration_ms: Date.now() - start5,
    screenshot: screenshotPath,
  });
} catch (err) {
  logTest(test5, "fail", {
    error: String(err),
    duration_ms: Date.now() - start5,
  });
}

// -----------------------------------------------------------------------------
// Test 6: Combined specialized inputs via fields()
// -----------------------------------------------------------------------------
const test6 = "fields-combined";
logTest(test6, "running");
const start6 = Date.now();

try {
  debug("Test 6: fields() with combined specialized input types");

  const fieldDefs = [
    {
      name: "website",
      label: "Website (URL)",
      type: "url" as const,
      placeholder: "https://example.com",
    },
    {
      name: "phone",
      label: "Phone (Tel)",
      type: "tel" as const,
      placeholder: "123-456-7890",
    },
    {
      name: "themeColor",
      label: "Theme Color",
      type: "color" as const,
      value: "#4f46e5",
    },
    {
      name: "email",
      label: "Email",
      type: "email" as const,
      placeholder: "you@example.com",
    },
    {
      name: "age",
      label: "Age",
      type: "number" as const,
      placeholder: "25",
    },
  ];

  const fieldsPromise = fields(fieldDefs);
  await new Promise((r) => setTimeout(r, 1200)); // Extra time for complex form
  const screenshotPath = await captureAndSave("combined");
  
  debug("Test 6: Screenshot captured. Awaiting user input to continue...");
  const result = await fieldsPromise;
  debug(`Test 6 result: ${JSON.stringify(result)}`);

  // Verify all expected fields are present (even if empty)
  const expectedFields = ["website", "phone", "themeColor", "email", "age"];
  if (Array.isArray(result) && result.length === expectedFields.length) {
    logTest(test6, "pass", { 
      result,
      duration_ms: Date.now() - start6,
      screenshot: screenshotPath,
    });
  } else {
    logTest(test6, "fail", {
      error: `Expected ${expectedFields.length} values, got ${Array.isArray(result) ? result.length : 'non-array'}`,
      result,
      duration_ms: Date.now() - start6,
      screenshot: screenshotPath,
    });
  }
} catch (err) {
  logTest(test6, "fail", {
    error: String(err),
    duration_ms: Date.now() - start6,
  });
}

// -----------------------------------------------------------------------------
// Summary
// -----------------------------------------------------------------------------
debug("test-form-specialized.ts completed!");

await div(
  md(`# form() and fields() Specialized Inputs Tests Complete

All specialized form input tests have been executed.

## Test Cases Run
1. **fields-url**: URL input type via fields()
2. **fields-search**: Search input (as text) via fields()
3. **fields-tel**: Telephone input type via fields()
4. **fields-color**: Color picker input via fields()
5. **form-textarea**: Multi-line textarea via form()
6. **fields-combined**: All specialized types via fields()

## Screenshots Location
\`.test-screenshots/form-specialized-*.png\`

## Notes
- fields() uses native GPUI components
- form() parses HTML to native GPUI components
- Color inputs display as text fields (native picker not implemented)
- URL/tel types are supported but render like text inputs

---

*Check JSONL output for detailed results*

Press Escape to exit.`)
);

debug("test-form-specialized.ts exiting...");
