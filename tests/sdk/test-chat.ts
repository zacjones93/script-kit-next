/**
 * Test: Chat Prompt (TIER 4A)
 * 
 * Tests the conversational chat UI where messages can be added programmatically.
 * 
 * Requires GPUI support for:
 * - 'chat' message type to open chat UI
 * - 'chatAction' message type for addMessage, setInput, submit actions
 * - Submit response with user's final input
 */

import '../kit-sdk';

// Helper to pause for async visualization
const wait = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

async function testBasicChat() {
  console.log('Test 1: Basic chat with messages');
  
  const result = await chat({
    onInit: async () => {
      // Add initial messages
      chat.addMessage({ text: "Welcome! I'm your assistant.", position: 'left' });
      await wait(500);
      chat.addMessage({ text: "How can I help you today?", position: 'left' });
    },
    onSubmit: async (input) => {
      console.log(`User submitted: ${input}`);
      // Could add response message here
      chat.addMessage({ text: `You said: ${input}`, position: 'left' });
    },
  });
  
  console.log(`Final chat result: ${result}`);
}

async function testChatWithSetInput() {
  console.log('Test 2: Chat with pre-filled input');
  
  const result = await chat({
    onInit: async () => {
      chat.addMessage({ text: "Type your name:", position: 'left' });
      chat.setInput("John Doe"); // Pre-fill the input
    },
  });
  
  console.log(`Name entered: ${result}`);
}

async function testSimpleChat() {
  console.log('Test 3: Simple chat without options');
  
  // Can also call without options
  const result = await chat();
  console.log(`Simple chat result: ${result}`);
}

async function testConversation() {
  console.log('Test 4: Multi-turn conversation');
  
  // First turn
  let response = await chat({
    onInit: async () => {
      chat.addMessage({ text: "What's your favorite color?", position: 'left' });
    },
  });
  
  // Second turn with context
  response = await chat({
    onInit: async () => {
      chat.addMessage({ text: `You said "${response}". Nice choice!`, position: 'left' });
      chat.addMessage({ text: "What's your favorite number?", position: 'left' });
    },
  });
  
  console.log(`Final answer: ${response}`);
}

// Run tests
(async () => {
  console.log('=== Chat Prompt Tests (TIER 4A) ===\n');
  console.log('NOTE: These tests require GPUI chat support.\n');
  
  await testBasicChat();
  await testChatWithSetInput();
  await testSimpleChat();
  await testConversation();
  
  console.log('\n=== All chat tests complete ===');
})();
