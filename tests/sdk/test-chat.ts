// Name: SDK Test - chat()
// Description: Tests chat() conversational UI prompt

/**
 * SDK TEST: test-chat.ts
 *
 * Tests the conversational chat UI where messages can be added programmatically.
 *
 * Test cases:
 * 1. chat-basic: Basic chat with messages
 * 2. chat-setinput: Chat with pre-filled input
 * 3. chat-simple: Simple chat without options
 *
 * Requires GPUI support for:
 * - 'chat' message type to open chat UI
 * - 'chatAction' message type for addMessage, setInput, submit actions
 * - Submit response with user's final input
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

// Helper to pause for async visualization
const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

// =============================================================================
// Tests
// =============================================================================

debug("test-chat.ts starting...");
debug(`SDK globals: chat=${typeof chat}`);

// -----------------------------------------------------------------------------
// Test 1: Basic chat with messages
// -----------------------------------------------------------------------------
const test1 = "chat-basic";
logTest(test1, "running");
const start1 = Date.now();

try {
	debug("Test 1: chat() with messages");

	const result = await chat({
		onInit: async () => {
			// Add initial messages
			chat.addMessage({
				text: "Welcome! I'm your assistant.",
				position: "left",
			});
			await wait(500);
			chat.addMessage({ text: "How can I help you today?", position: "left" });
		},
		onSubmit: async (input) => {
			debug(`User submitted: ${input}`);
			// Could add response message here
			chat.addMessage({ text: `You said: ${input}`, position: "left" });
		},
	});

	debug(`Test 1 result: "${result}"`);

	// Assertion: result should be a string (user's input)
	if (typeof result !== "string") {
		logTest(test1, "fail", {
			error: `Expected string response, got ${typeof result}`,
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
// Test 2: Chat with pre-filled input
// -----------------------------------------------------------------------------
const test2 = "chat-setinput";
logTest(test2, "running");
const start2 = Date.now();

try {
	debug("Test 2: chat() with setInput");

	const result = await chat({
		onInit: async () => {
			chat.addMessage({ text: "Type your name:", position: "left" });
			chat.setInput("John Doe"); // Pre-fill the input
		},
	});

	debug(`Test 2 result: "${result}"`);

	// Assertion: result should be a string
	if (typeof result !== "string") {
		logTest(test2, "fail", {
			error: `Expected string response, got ${typeof result}`,
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
// Test 3: Simple chat without options
// -----------------------------------------------------------------------------
const test3 = "chat-simple";
logTest(test3, "running");
const start3 = Date.now();

try {
	debug("Test 3: chat() without options");

	// Can also call without options
	const result = await chat();

	debug(`Test 3 result: "${result}"`);

	// Assertion: result should be a string (even empty is valid)
	if (typeof result !== "string") {
		logTest(test3, "fail", {
			error: `Expected string response, got ${typeof result}`,
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
debug("test-chat.ts completed!");

await div(
	md(`# chat() Tests Complete

All chat prompt tests have been executed.

## Test Cases Run
1. **chat-basic**: Basic chat with messages (expects string)
2. **chat-setinput**: Chat with pre-filled input (expects string)
3. **chat-simple**: Simple chat without options (expects string)

---

*Check the JSONL output for detailed results*

Press Escape or click to exit.`),
);

debug("test-chat.ts exiting...");
