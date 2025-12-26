// Name: SDK Test - select()
// Description: Tests select() multi-select prompt

/**
 * SDK TEST: test-select.ts
 *
 * Tests the select() function for multi-select prompts.
 *
 * Test cases:
 * 1. select-basic: Basic multi-select with string choices
 * 2. select-mixed: Mixed string and object choices
 *
 * Expected behavior:
 * - select() returns array of selected values
 * - Supports both string and object choice formats
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

debug("test-select.ts starting...");
debug(`SDK globals: select=${typeof select}`);

// -----------------------------------------------------------------------------
// Test 1: Basic multi-select
// -----------------------------------------------------------------------------
const test1 = "select-basic";
logTest(test1, "running");
const start1 = Date.now();

try {
	debug("Test 1: select() with string choices");

	const result = await select("Choose your favorite fruits", [
		"Apple",
		"Banana",
		"Cherry",
		"Date",
		"Elderberry",
	]);

	debug(`Test 1 result: ${JSON.stringify(result)}`);

	// Assertion: result should be an array (can be empty if user selects nothing)
	if (!Array.isArray(result)) {
		logTest(test1, "fail", {
			error: `Expected array of selected items, got ${typeof result}`,
			result,
			duration_ms: Date.now() - start1,
		});
	} else {
		// Each selected item should be a string (from our string choices)
		const allStrings = result.every((item) => typeof item === "string");
		if (!allStrings && result.length > 0) {
			logTest(test1, "fail", {
				error: `Expected all selected items to be strings`,
				result,
				duration_ms: Date.now() - start1,
			});
		} else {
			logTest(test1, "pass", { result, duration_ms: Date.now() - start1 });
		}
	}
} catch (err) {
	logTest(test1, "fail", {
		error: String(err),
		duration_ms: Date.now() - start1,
	});
}

// -----------------------------------------------------------------------------
// Test 2: Mixed string and object choices
// -----------------------------------------------------------------------------
const test2 = "select-mixed";
logTest(test2, "running");
const start2 = Date.now();

try {
	debug("Test 2: select() with mixed choices");

	const result = await select("Choose items", [
		"Simple String",
		{
			name: "Object Choice",
			value: "obj-value",
			description: "Has a description",
		},
		{
			name: "Another Object",
			value: "numeric-choice",
			description: "Value is a string",
		},
		"Another String",
	]);

	debug(`Test 2 result: ${JSON.stringify(result)}`);

	// Assertion: result should be an array
	if (!Array.isArray(result)) {
		logTest(test2, "fail", {
			error: `Expected array of selected items, got ${typeof result}`,
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
// Show Summary
// -----------------------------------------------------------------------------
debug("test-select.ts completed!");

await div(
	md(`# select() Tests Complete

All multi-select tests have been executed.

## Test Cases Run
1. **select-basic**: String choices (expects array)
2. **select-mixed**: Mixed string/object choices (expects array)

---

*Check the JSONL output for detailed results*

Press Escape or click to exit.`),
);

debug("test-select.ts exiting...");
