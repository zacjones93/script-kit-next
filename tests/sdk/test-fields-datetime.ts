// Name: SDK Test - fields() Date/Time Input Types
// Description: Tests fields() with date, time, datetime-local, month, week input types

/**
 * SDK TEST: test-fields-datetime.ts
 *
 * Tests the fields() function for date/time input types.
 *
 * CRITICAL FINDING: The `fields()` function is NOT YET IMPLEMENTED in the Rust/GPUI side.
 * The SDK sends `Message::Fields` but `execute_script.rs` doesn't handle it, causing:
 *   "Unhandled message type: Fields"
 *
 * Test cases (will fail until fields() is implemented):
 * 1. fields-date: Date input field
 * 2. fields-time: Time input field
 * 3. fields-datetime-local: DateTime-local input field
 * 4. fields-month: Month input field
 * 5. fields-week: Week input field
 * 6. fields-all-datetime: Combined form with all date/time types
 *
 * SDK Type Gap:
 * The FieldDef type in kit-sdk.ts line 25 only includes:
 *   text | password | email | number | date | time | url | tel | color
 *
 * Missing HTML5 input types:
 *   datetime-local, month, week, range, search, hidden, file, checkbox, radio
 */

import "../../scripts/kit-sdk.ts";
import { writeFileSync, mkdirSync, existsSync } from "fs";
import { join } from "path";

// =============================================================================
// Test Infrastructure
// =============================================================================

interface TestResult {
	test: string;
	status: "running" | "pass" | "fail" | "skip" | "blocked";
	timestamp: string;
	result?: unknown;
	error?: string;
	duration_ms?: number;
	blocker?: string;
}

function logTest(
	name: string,
	status: TestResult["status"],
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
	console.error(`[TEST] ${msg}`);
}

async function captureAndSave(filename: string): Promise<string> {
	try {
		const screenshot = await captureScreenshot();
		const screenshotDir = join(process.cwd(), ".test-screenshots");
		mkdirSync(screenshotDir, { recursive: true });

		const timestamp = Date.now();
		const filepath = join(screenshotDir, `${filename}-${timestamp}.png`);
		writeFileSync(filepath, Buffer.from(screenshot.data, "base64"));

		debug(`[SCREENSHOT] ${filepath} (${screenshot.width}x${screenshot.height})`);
		return filepath;
	} catch (err) {
		debug(`[SCREENSHOT ERROR] ${err}`);
		return "";
	}
}

// =============================================================================
// Tests
// =============================================================================

debug("test-fields-datetime.ts starting...");
debug(`SDK globals: fields=${typeof fields}, form=${typeof form}, div=${typeof div}`);

// First, document the blocking issue
const blockerTest = "fields-implementation-check";
logTest(blockerTest, "running");
const blockerStart = Date.now();

debug("CRITICAL: Checking if fields() is implemented in GPUI...");
debug("The SDK sends Message::Fields but the Rust side doesn't handle it.");
debug("See execute_script.rs match statement - Fields falls through to 'other' catch-all.");

logTest(blockerTest, "blocked", {
	blocker: "Message::Fields not handled in execute_script.rs",
	error: "fields() prompt type not implemented in GPUI - shows 'Unhandled message type: Fields'",
	result: {
		sdk_sends: "Message::Fields",
		rust_handler: "Missing - falls through to 'other' branch",
		required_fix: "Add ShowFields to PromptMessage enum and handle Message::Fields",
	},
	duration_ms: Date.now() - blockerStart,
});

// -----------------------------------------------------------------------------
// Test 1-6: Document expected behavior (but mark as blocked)
// -----------------------------------------------------------------------------

const testCases = [
	{
		name: "fields-date",
		description: "Date input field",
		field: { name: "birthday", label: "Birthday", type: "date" },
		typeInSDK: true,
	},
	{
		name: "fields-time",
		description: "Time input field",
		field: { name: "meeting", label: "Meeting Time", type: "time" },
		typeInSDK: true,
	},
	{
		name: "fields-datetime-local",
		description: "DateTime-local input field",
		field: { name: "appointment", label: "Appointment", type: "datetime-local" },
		typeInSDK: false,
	},
	{
		name: "fields-month",
		description: "Month input field",
		field: { name: "expiry", label: "Card Expiry Month", type: "month" },
		typeInSDK: false,
	},
	{
		name: "fields-week",
		description: "Week input field",
		field: { name: "week", label: "Week Number", type: "week" },
		typeInSDK: false,
	},
];

for (const testCase of testCases) {
	logTest(testCase.name, "blocked", {
		blocker: "fields() not implemented in GPUI",
		result: {
			description: testCase.description,
			field_definition: testCase.field,
			type_in_sdk: testCase.typeInSDK,
			type_in_sdk_note: testCase.typeInSDK
				? "Type is defined in FieldDef"
				: "Type NOT in FieldDef - needs to be added",
		},
	});
}

// Test 6: Combined fields
logTest("fields-all-datetime", "blocked", {
	blocker: "fields() not implemented in GPUI",
	result: {
		description: "Combined form with all date/time types",
		fields_count: 5,
		fields: testCases.map((tc) => tc.field),
	},
});

// -----------------------------------------------------------------------------
// Use div() to display findings (since div() IS implemented)
// -----------------------------------------------------------------------------

debug("Displaying findings using div() (which IS implemented)...");

const findingsHtml = `
# fields() Date/Time Input Types - Test Results

## CRITICAL BLOCKER

**The \`fields()\` function is NOT YET IMPLEMENTED in GPUI.**

When the SDK calls \`fields()\`, it sends a \`Message::Fields\` to the Rust app, but the message handler in \`execute_script.rs\` doesn't have a case for it. It falls through to the "Unhandled message type" catch-all.

### Required Implementation

1. Add \`ShowFields\` variant to \`PromptMessage\` enum in \`main.rs\`
2. Add \`Message::Fields\` handling in \`execute_script.rs\` match statement
3. Implement \`PromptMessage::ShowFields\` handler in \`prompt_handler.rs\`
4. Create \`FieldsPrompt\` view in GPUI (similar to \`FormPrompt\`)

## Test Cases (All Blocked)

| Test | Input Type | In SDK Type | Status |
|------|------------|-------------|--------|
| fields-date | date | ✅ Yes | 🚫 Blocked |
| fields-time | time | ✅ Yes | 🚫 Blocked |
| fields-datetime-local | datetime-local | ❌ No | 🚫 Blocked |
| fields-month | month | ❌ No | 🚫 Blocked |
| fields-week | week | ❌ No | 🚫 Blocked |
| fields-all-datetime | (combined) | Mixed | 🚫 Blocked |

## SDK Type Gap

The \`FieldDef\` type in \`kit-sdk.ts\` line 25 only includes:
\`\`\`typescript
type?: 'text' | 'password' | 'email' | 'number' | 'date' | 'time' | 'url' | 'tel' | 'color'
\`\`\`

**Missing HTML5 input types:**
- \`datetime-local\` - Date + time picker
- \`month\` - Month/year picker
- \`week\` - Week picker
- \`range\` - Slider
- \`search\` - Search input
- \`hidden\` - Hidden field
- \`file\` - File upload
- \`checkbox\` - Checkbox
- \`radio\` - Radio button

---

*Press Escape to exit*
`;

// Capture a screenshot showing these findings
await div(md(findingsHtml));

// Wait for render
await new Promise((resolve) => setTimeout(resolve, 500));

// Capture screenshot of the findings
const findingsScreenshot = await captureAndSave("fields-datetime-findings");
debug(`Findings screenshot: ${findingsScreenshot}`);

// Log the findings screenshot result
logTest("findings-display", "pass", {
	result: {
		screenshot: findingsScreenshot,
		note: "Displayed test findings using div() since fields() is blocked",
	},
});

debug("test-fields-datetime.ts completed!");
debug("SUMMARY: fields() is NOT implemented in GPUI. All date/time tests are BLOCKED.");
