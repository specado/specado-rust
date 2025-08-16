#!/usr/bin/env node
/**
 * Test script for capability taxonomy in Node.js
 */

const specado = require('./index.js');

async function testCapabilities() {
    console.log('Testing Specado Capability Taxonomy in Node.js\n');
    console.log('='.repeat(50));
    
    try {
        // Test 1: Basic functionality works
        console.log('\n1. Testing basic functionality...');
        const version = specado.version();
        console.log(`   Specado version: ${version}`);
        
        const hello = specado.helloWorld();
        console.log(`   Hello world: ${hello}`);
        
        // Test 2: Get OpenAI manifest
        console.log('\n2. Getting OpenAI manifest...');
        const openaiManifestJson = specado.getOpenaiManifest();
        const openaiManifest = JSON.parse(openaiManifestJson);
        console.log(`   Provider: ${openaiManifest.info.name}`);
        console.log(`   Models: ${Object.keys(openaiManifest.models)}`);
        
        // Test 3: Get Anthropic manifest
        console.log('\n3. Getting Anthropic manifest...');
        const anthropicManifestJson = specado.getAnthropicManifest();
        const anthropicManifest = JSON.parse(anthropicManifestJson);
        console.log(`   Provider: ${anthropicManifest.info.name}`);
        console.log(`   Models: ${Object.keys(anthropicManifest.models)}`);
        
        // Test 4: Get specific model capabilities
        console.log('\n4. Getting model capabilities...');
        const gpt4CapsJson = specado.getModelCapabilities('openai', 'gpt-4-turbo');
        if (gpt4CapsJson) {
            const gpt4Caps = JSON.parse(gpt4CapsJson);
            console.log('   GPT-4 Turbo:');
            console.log(`     - Function calling: ${gpt4Caps.features.function_calling}`);
            console.log(`     - Vision: ${gpt4Caps.features.vision}`);
            console.log(`     - JSON mode: ${gpt4Caps.features.json_mode}`);
            console.log(`     - Max context: ${gpt4Caps.constraints.tokens.max_context_window}`);
        }
        
        const claudeCapsJson = specado.getModelCapabilities('anthropic', 'claude-3-opus');
        if (claudeCapsJson) {
            const claudeCaps = JSON.parse(claudeCapsJson);
            console.log('   Claude 3 Opus:');
            console.log(`     - Tool use: ${claudeCaps.features.tool_use}`);
            console.log(`     - Vision: ${claudeCaps.features.vision}`);
            console.log(`     - JSON mode: ${claudeCaps.features.json_mode}`);
            console.log(`     - Max context: ${claudeCaps.constraints.tokens.max_context_window}`);
        }
        
        // Test 5: Compare capabilities
        console.log('\n5. Comparing capabilities (GPT-4 → Claude)...');
        if (gpt4CapsJson && claudeCapsJson) {
            const comparisonJson = specado.compareCapabilities(gpt4CapsJson, claudeCapsJson);
            const comparison = JSON.parse(comparisonJson);
            console.log(`   Is lossy: ${comparison.lossiness_report.is_lossy}`);
            console.log(`   Severity: ${comparison.lossiness_report.severity}`);
            
            if (comparison.missing_capabilities && comparison.missing_capabilities.length > 0) {
                console.log(`   Missing capabilities: ${comparison.missing_capabilities}`);
            }
            
            if (comparison.lossiness_report.recommendations && comparison.lossiness_report.recommendations.length > 0) {
                console.log('   Recommendations:');
                for (const rec of comparison.lossiness_report.recommendations) {
                    console.log(`     - ${rec}`);
                }
            }
        }
        
        // Test 6: Verify serialization with custom capability
        console.log('\n6. Testing serialization with custom capability...');
        const customCapability = {
            version: "1.0.0",
            modalities: {
                input: ["Text"],
                output: ["Text"],
                configs: {
                    image: null,
                    audio: null,
                    video: null,
                    document: null
                }
            },
            features: {
                function_calling: true,
                json_mode: false,
                streaming: true,
                logprobs: false,
                multiple_responses: false,
                stop_sequences: true,
                seed_support: false,
                tool_use: false,
                vision: false
            },
            parameters: {
                temperature: {
                    supported: true,
                    min: 0.0,
                    max: 1.0,
                    default: 0.7
                },
                top_p: {
                    supported: false,
                    min: null,
                    max: null,
                    default: null
                },
                top_k: {
                    supported: false,
                    min: null,
                    max: null,
                    default: null
                },
                max_tokens: {
                    supported: true,
                    min: 1,
                    max: 2048,
                    default: 512
                },
                frequency_penalty: {
                    supported: false,
                    min: null,
                    max: null,
                    default: null
                },
                presence_penalty: {
                    supported: false,
                    min: null,
                    max: null,
                    default: null
                },
                repetition_penalty: {
                    supported: false,
                    min: null,
                    max: null,
                    default: null
                }
            },
            roles: {
                system: true,
                user: true,
                assistant: true,
                function: false,
                tool: false,
                custom_roles: []
            },
            constraints: {
                tokens: {
                    max_context_window: 4096,
                    max_input_tokens: null,
                    max_output_tokens: 2048,
                    max_tokens_per_message: null,
                    encoding: null
                },
                rate_limits: {
                    requests_per_minute: 100,
                    requests_per_hour: null,
                    requests_per_day: null,
                    tokens_per_minute: 50000,
                    tokens_per_hour: null,
                    tokens_per_day: null,
                    max_concurrent_requests: 10
                },
                messages: {
                    max_messages_per_conversation: null,
                    max_message_length: null,
                    min_messages_required: 1,
                    max_system_message_length: null,
                    allow_empty_messages: false,
                    allow_consecutive_same_role: false
                },
                timeouts: {
                    max_request_timeout_seconds: 60,
                    default_timeout_seconds: 30,
                    stream_timeout_seconds: 60,
                    connection_timeout_seconds: 10
                }
            },
            extensions: {
                custom: {},
                experimental: []
            }
        };
        
        // Compare custom with GPT-4
        const customCapabilityJson = JSON.stringify(customCapability);
        const comparison2Json = specado.compareCapabilities(customCapabilityJson, gpt4CapsJson);
        const comparison2 = JSON.parse(comparison2Json);
        console.log('   Custom → GPT-4 comparison:');
        console.log(`     - Is lossy: ${comparison2.lossiness_report.is_lossy}`);
        console.log(`     - Details: ${comparison2.lossiness_report.details.length} issues found`);
        
        console.log('\n✅ All capability tests passed!');
        
    } catch (error) {
        console.error('❌ Error:', error);
        console.error('Stack trace:', error.stack);
    }
}

testCapabilities();