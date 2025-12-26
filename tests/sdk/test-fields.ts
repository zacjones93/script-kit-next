// Name: SDK Test - fields() and form()
// Description: Tests fields() and form() multi-input prompts

/**
 * SDK TEST: test-fields.ts
 *
 * Tests the fields() and form() functions for multi-input forms.
 *
 * Test cases:
 * 1. fields-simple: Simple string field labels
 * 2. fields-detailed: Detailed field definitions with types
 * 3. form-custom: Custom HTML form
 *
 * Expected behavior:
 * - fields() returns array of values matching number of fields
 * - form() returns object with form data
 */

import "../../scripts/kit-sdk.ts";

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

// =============================================================================
// Tests
// =============================================================================

debug("test-fields.ts starting...");
debug(`SDK globals: fields=${typeof fields}, form=${typeof form}`);

// -----------------------------------------------------------------------------
// Test 1: Simple string fields
// -----------------------------------------------------------------------------
const test1 = "fields-simple";
logTest(test1, "running");
const start1 = Date.now();

try {
	debug("Test 1: fields() with simple string labels");

	const result = await fields(["Name", "Email", "Phone"]);

	debug(`Test 1 result: ${JSON.stringify(result)}`);

	// Assertion: result should be an array with 3 elements
	if (!Array.isArray(result)) {
		logTest(test1, "fail", {
			error: `Expected array, got ${typeof result}`,
			result,
			duration_ms: Date.now() - start1,
		});
	} else if (result.length !== 3) {
		logTest(test1, "fail", {
			error: `Expected 3 values for 3 fields, got ${result.length}`,
			result,
			duration_ms: Date.now() - start1,
		});
	} else {
		logTest(test1, "pass", { result, duration_ms: Date.now() - start1 });
	}
} catch (err) {
	logTest(test1, "fail", {
		error: String(err),
		duration_ms: Date.now() - start1,
	});
}

// -----------------------------------------------------------------------------
// Test 2: Detailed field definitions
// -----------------------------------------------------------------------------
const test2 = "fields-detailed";
logTest(test2, "running");
const start2 = Date.now();

try {
	debug("Test 2: fields() with detailed field definitions");

	const fieldDefs: (
		| string
		| {
				name: string;
				label: string;
				type?:
					| "text"
					| "password"
					| "email"
					| "number"
					| "date"
					| "time"
					| "url"
					| "tel"
					| "color";
				placeholder?: string;
				value?: string;
		  }
	)[] = [
		{ name: "username", label: "Username", placeholder: "Enter your username" },
		{
			name: "password",
			label: "Password",
			type: "password",
			placeholder: "Enter password",
		},
		{
			name: "email",
			label: "Email Address",
			type: "email",
			value: "user@example.com",
		},
		{ name: "age", label: "Age", type: "number" },
		{ name: "birthday", label: "Birthday", type: "date" },
		{
			name: "website",
			label: "Website",
			type: "url",
			placeholder: "https://example.com",
		},
	];

	const result = await fields(fieldDefs);

	debug(`Test 2 result: ${JSON.stringify(result)}`);

	// Assertion: result should be an array with 6 elements
	if (!Array.isArray(result)) {
		logTest(test2, "fail", {
			error: `Expected array, got ${typeof result}`,
			result,
			duration_ms: Date.now() - start2,
		});
	} else if (result.length !== fieldDefs.length) {
		logTest(test2, "fail", {
			error: `Expected ${fieldDefs.length} values for ${fieldDefs.length} fields, got ${result.length}`,
			result,
			duration_ms: Date.now() - start2,
		});
	} else {
		logTest(test2, "pass", { result, duration_ms: Date.now() - start2 });
	}
} catch (err) {
	logTest(test2, "fail", {
		error: String(err),
		duration_ms: Date.now() - start2,
	});
}

// -----------------------------------------------------------------------------
// Test 3: Custom HTML form
// -----------------------------------------------------------------------------
const test3 = "form-custom";
logTest(test3, "running");
const start3 = Date.now();

try {
	debug("Test 3: form() with custom HTML");

	const result = await form(`
    <form>
      <label for="firstName">First Name:</label>
      <input type="text" id="firstName" name="firstName" />
      
      <label for="lastName">Last Name:</label>
      <input type="text" id="lastName" name="lastName" />
      
      <label for="favoriteColor">Favorite Color:</label>
      <input type="color" id="favoriteColor" name="favoriteColor" />
    </form>
  `);

	debug(`Test 3 result: ${JSON.stringify(result)}`);

	// Assertion: result should be an object (form data)
	if (result === null || result === undefined) {
		logTest(test3, "fail", {
			error: `Expected form data object, got ${result}`,
			result,
			duration_ms: Date.now() - start3,
		});
	} else if (typeof result !== "object") {
		logTest(test3, "fail", {
			error: `Expected object, got ${typeof result}`,
			result,
			duration_ms: Date.now() - start3,
		});
	} else {
		logTest(test3, "pass", { result, duration_ms: Date.now() - start3 });
	}
} catch (err) {
	logTest(test3, "fail", {
		error: String(err),
		duration_ms: Date.now() - start3,
	});
}

// -----------------------------------------------------------------------------
// Show Summary
// -----------------------------------------------------------------------------
debug("test-fields.ts completed!");

await div(
	md(`# fields() and form() Tests Complete

All form input tests have been executed.

## Test Cases Run
1. **fields-simple**: Simple string field labels (expects array of 3)
2. **fields-detailed**: Detailed field definitions (expects array of 6)
3. **form-custom**: Custom HTML form (expects object)

---

*Check the JSONL output for detailed results*

Press Escape or click to exit.`),
);

debug("test-fields.ts exiting...");
