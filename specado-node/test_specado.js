#!/usr/bin/env node
/**
 * Comprehensive test suite for Specado Node.js bindings.
 * 
 * This module tests the complete FFI API including chat completions,
 * message handling, client configuration, and routing functionality.
 */

const { existsSync } = require('fs');
const { join } = require('path');
const assert = require('assert');

// Try to find the native module in various locations
const possiblePaths = [
    './specado.node',
    './specado.darwin-x64.node',
    './specado.darwin-arm64.node',
    './specado.linux-x64-gnu.node',
    './specado.linux-x64-musl.node',
    './target/debug/specado.node',
    './target/release/specado.node',
    '../target/debug/libspecado_node.dylib',
    '../target/release/libspecado_node.dylib',
    '../target/debug/libspecado_node.so',
    '../target/release/libspecado_node.so',
];

let specado = null;
let modulePath = null;

// Try to load the module from various locations
for (const path of possiblePaths) {
    const fullPath = join(__dirname, path);
    if (existsSync(fullPath)) {
        try {
            specado = require(fullPath);
            modulePath = fullPath;
            break;
        } catch (e) {
            // Continue trying other paths
        }
    }
}

// If not found in expected locations, try to require it directly (for npm-installed case)
if (!specado) {
    try {
        specado = require('./index.js');
    } catch (e) {
        console.error('Warning: Could not load specado module.');
        console.error('Make sure to build the Node module first with: npm run build');
        console.error('Searched paths:', possiblePaths);
        process.exit(1);
    }
}

console.log('🧪 Testing Specado Node.js bindings...');
console.log('Loaded specado module from:', modulePath || 'npm package');
console.log('');

/**
 * Test basic library functions
 */
function testBasicFunctions() {
    console.log('✅ Test 1: Basic library functions');
    
    // Test version
    const version = specado.version();
    assert.strictEqual(typeof version, 'string', 'version should return a string');
    assert(version.length > 0, 'version should not be empty');
    console.log('  - version():', version);
    
    // Test helloWorld
    const hello = specado.helloWorld();
    assert.strictEqual(hello, 'Hello from Specado Core!', 'helloWorld should return correct message');
    console.log('  - helloWorld():', hello);
}

/**
 * Test Message creation
 */
function testMessage() {
    console.log('\n✅ Test 2: Message creation');
    
    const msg = specado.createMessage('user', 'Hello from Node.js!');
    assert.strictEqual(msg.role, 'user', 'Message role should be set correctly');
    assert.strictEqual(msg.content, 'Hello from Node.js!', 'Message content should be set correctly');
    console.log('  - Created message:', { role: msg.role, content: msg.content });
}

/**
 * Test Client creation with default config
 */
function testClientDefault() {
    console.log('\n✅ Test 3: Client creation (default config)');
    
    const client = new specado.Client();
    assert(client, 'Client should be created');
    
    console.log('  - Client methods available:', Object.getOwnPropertyNames(Object.getPrototypeOf(client)));
    console.log('  - Client type:', typeof client);
    
    const keys = client.configKeys();
    assert(Array.isArray(keys), 'configKeys should return an array');
    console.log('  - Config keys:', keys);
    
    const primary = client.getConfig('primary_provider');
    const fallback = client.getConfig('fallback_provider');
    console.log('  - Primary provider:', primary);
    console.log('  - Fallback provider:', fallback);
    
    assert.strictEqual(primary, 'openai', 'Default primary should be openai');
    assert.strictEqual(fallback, 'anthropic', 'Default fallback should be anthropic');
}

/**
 * Test Client creation with custom config
 */
function testClientCustom() {
    console.log('\n✅ Test 4: Client creation (custom config)');
    
    const client = new specado.Client({
        primary_provider: 'anthropic',
        fallback_provider: 'openai'
    });
    
    const primary = client.getConfig('primary_provider');
    const fallback = client.getConfig('fallback_provider');
    console.log('  - Primary provider:', primary);
    console.log('  - Fallback provider:', fallback);
    
    assert.strictEqual(primary, 'anthropic', 'Custom primary should be anthropic');
    assert.strictEqual(fallback, 'openai', 'Custom fallback should be openai');
}

/**
 * Test Chat API access
 */
function testChatApi() {
    console.log('\n✅ Test 5: Chat API access');
    
    const client = new specado.Client();
    const chat = client.getChat();
    assert(chat, 'Chat API should be accessible');
    
    const completions = chat.getCompletions();
    assert(completions, 'Completions API should be accessible');
    console.log('  - Accessed chat.completions API successfully');
}

/**
 * Test async chat completion
 */
async function testChatCompletion() {
    console.log('\n✅ Test 6: Async chat completion');
    
    const client = new specado.Client();
    const chat = client.getChat();
    const completions = chat.getCompletions();
    
    const messages = [
        specado.createMessage('user', 'Hello, how are you?')
    ];
    
    const response = await completions.create('gpt-4', messages, 0.7, 100);
    
    // Validate response structure
    assert(response.id, 'Response should have an ID');
    assert(response.object === 'chat.completion', 'Response object should be chat.completion');
    assert(response.model === 'gpt-4', 'Response model should match request');
    assert(Array.isArray(response.choices), 'Response should have choices array');
    assert(response.choices.length > 0, 'Response should have at least one choice');
    assert(response.extensions, 'Response should have extensions');
    
    console.log('  - Response ID:', response.id);
    console.log('  - Response model:', response.model);
    console.log('  - Extensions object:', response.extensions);
    console.log('  - Extensions keys:', Object.keys(response.extensions || {}));
    console.log('  - Provider used:', response.extensions.providerUsed);
    console.log('  - Fallback triggered:', response.extensions.fallbackTriggered);
    console.log('  - Attempts:', response.extensions.attempts);
    console.log('  - Message role:', response.choices[0].message.role);
    console.log('  - Message content preview:', response.choices[0].message.content.substring(0, 50) + '...');
    
    // Validate extensions with correct camelCase property names
    assert(response.extensions, 'Extensions should exist');
    assert(typeof response.extensions.providerUsed === 'string', 'Provider used should be a string');
    assert(typeof response.extensions.fallbackTriggered === 'boolean', 'Fallback triggered should be boolean');
    assert(typeof response.extensions.attempts === 'number', 'Attempts should be a number');
    console.log('  - Extensions validation: OK');
    
    // Validate choice
    const choice = response.choices[0];
    assert(choice.index === 0, 'Choice index should be 0');
    assert(choice.message.role === 'assistant', 'Response message should be from assistant');
    assert(typeof choice.message.content === 'string', 'Response content should be a string');
}

/**
 * Run all tests
 */
async function runTests() {
    try {
        testBasicFunctions();
        testMessage();
        testClientDefault();
        testClientCustom();
        testChatApi();
        await testChatCompletion();
        
        console.log('\n🎉 All tests passed! Node.js bindings are working correctly.');
        console.log('✅ MVP blocker issue #30 has been resolved.');
        process.exit(0);
    } catch (error) {
        console.error('\n❌ Test failed:', error.message);
        console.error(error.stack);
        process.exit(1);
    }
}

// Run the tests
runTests();