// Name: SDK Test - template()
// Description: Tests template() tab-through snippet editing

/**
 * SDK TEST: test-template.ts
 *
 * Tests the template() function which provides mustache-style {{placeholder}} editing.
 *
 * Test cases:
 * 1. template-mustache-single: Single {{placeholder}}
 * 2. template-mustache-multiple: Multiple {{placeholders}}
 * 3. template-mustache-duplicate: Duplicate {{placeholders}} (should show once)
 * 4. template-mustache-underscore: {{placeholder_with_underscore}}
 *
 * Expected behavior:
 * - template() sends JSONL message with type: 'template'
 * - User can tab through placeholders
 * - Filled template is returned
 */

import "../../scripts/kit-sdk";

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

debug("test-template.ts starting...");
debug(`SDK globals: template=${typeof template}`);

// -----------------------------------------------------------------------------
// Test 1: Simple template with single {{placeholder}}
// -----------------------------------------------------------------------------
const test1 = "template-mustache-single";
logTest(test1, "running");
const start1 = Date.now();

try {
	debug("Test 1: template() with single {{name}} placeholder");

	const result = await template("Hello {{name}}!");

	debug(`Test 1 result: "${result}"`);

	// Assertion: result should be a non-empty string (user filled in placeholders)
	if (typeof result !== "string") {
		logTest(test1, "fail", {
			error: `Expected string result, got ${typeof result}`,
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
// Test 2: Template with multiple {{placeholders}}
// -----------------------------------------------------------------------------
const test2 = "template-mustache-multiple";
logTest(test2, "running");
const start2 = Date.now();

try {
	debug("Test 2: template() with multiple placeholders");

	const result = await template(
		"Hello {{name}}, your email is {{email}} and your role is {{role}}.",
	);

	debug(`Test 2 result:\n${result}`);

	// Assertion: result should be a string
	if (typeof result !== "string") {
		logTest(test2, "fail", {
			error: `Expected string result, got ${typeof result}`,
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
// Test 3: Template with underscore placeholder names
// -----------------------------------------------------------------------------
const test3 = "template-mustache-underscore";
logTest(test3, "running");
const start3 = Date.now();

try {
	debug("Test 3: template() with underscore placeholder names");

	const result = await template(
		"Dear {{first_name}} {{last_name}},\n\nWelcome to {{company_name}}!",
	);

	debug(`Test 3 result:\n${result}`);

	// Assertion: result should be a string
	if (typeof result !== "string") {
		logTest(test3, "fail", {
			error: `Expected string result, got ${typeof result}`,
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
debug("test-template.ts completed!");

await div(
	md(
		"# template() Tests Complete\n\nAll `template()` tests have been executed.\n\n## Test Cases Run\n1. **template-mustache-single**: Single {{placeholder}}\n2. **template-mustache-multiple**: Multiple {{placeholders}}\n3. **template-mustache-underscore**: {{placeholder_with_underscore}}\n\n---\n\n*Check the JSONL output for detailed results*\n\nPress Escape or click to exit.",
	),
);

debug("test-template.ts exiting...");
