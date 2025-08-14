#!/usr/bin/env node
/**
 * Test module for Specado Node.js bindings.
 * 
 * This module tests the FFI bindings between Node.js and the Rust core library.
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

console.log('Loaded specado module from:', modulePath || 'npm package');

/**
 * Test the hello_world function
 */
function testHelloWorld() {
    console.log('Testing hello_world()...');
    const result = specado.helloWorld();
    assert.strictEqual(result, 'Hello from Specado Core!', 'hello_world should return correct message');
    assert.strictEqual(typeof result, 'string', 'hello_world should return a string');
    console.log('✓ hello_world() test passed');
}

/**
 * Test the version function
 */
function testVersion() {
    console.log('Testing version()...');
    const result = specado.version();
    assert.strictEqual(typeof result, 'string', 'version should return a string');
    assert(result.length > 0, 'version should not be empty');
    
    // Version should be in semver format
    const parts = result.split('.');
    assert(parts.length >= 2, 'version should be in semver format');
    console.log(`✓ version() test passed (version: ${result})`);
}

/**
 * Test the async hello_world function if available
 */
async function testHelloWorldAsync() {
    if (typeof specado.helloWorldAsync === 'function') {
        console.log('Testing helloWorldAsync()...');
        const result = await specado.helloWorldAsync();
        assert.strictEqual(result, 'Hello from Specado Core!', 'helloWorldAsync should return correct message');
        assert.strictEqual(typeof result, 'string', 'helloWorldAsync should return a string');
        console.log('✓ helloWorldAsync() test passed');
    } else {
        console.log('⚠ helloWorldAsync() not available');
    }
}

/**
 * Run all tests
 */
async function runTests() {
    console.log('Running Specado Node.js binding tests...\n');
    
    try {
        testHelloWorld();
        testVersion();
        await testHelloWorldAsync();
        
        console.log('\n✅ All tests passed!');
        process.exit(0);
    } catch (error) {
        console.error('\n❌ Test failed:', error.message);
        console.error(error.stack);
        process.exit(1);
    }
}

// Run the tests
runTests();