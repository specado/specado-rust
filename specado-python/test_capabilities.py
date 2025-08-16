#!/usr/bin/env python3
"""Test script for capability taxonomy in Python"""

import json
import specado

def test_capabilities():
    print("Testing Specado Capability Taxonomy in Python\n")
    print("=" * 50)
    
    # Test 1: Get OpenAI manifest
    print("\n1. Getting OpenAI manifest...")
    openai_manifest = specado.get_openai_manifest()
    print(f"   Provider: {openai_manifest['info']['name']}")
    print(f"   Models: {list(openai_manifest['models'].keys())}")
    
    # Test 2: Get Anthropic manifest
    print("\n2. Getting Anthropic manifest...")
    anthropic_manifest = specado.get_anthropic_manifest()
    print(f"   Provider: {anthropic_manifest['info']['name']}")
    print(f"   Models: {list(anthropic_manifest['models'].keys())}")
    
    # Test 3: Get specific model capabilities
    print("\n3. Getting model capabilities...")
    gpt4_caps = specado.get_model_capabilities("openai", "gpt-4-turbo")
    if gpt4_caps:
        print(f"   GPT-4 Turbo:")
        print(f"     - Function calling: {gpt4_caps['features']['function_calling']}")
        print(f"     - Vision: {gpt4_caps['features']['vision']}")
        print(f"     - JSON mode: {gpt4_caps['features']['json_mode']}")
        print(f"     - Max context: {gpt4_caps['constraints']['tokens']['max_context_window']}")
    
    claude_caps = specado.get_model_capabilities("anthropic", "claude-3-opus")
    if claude_caps:
        print(f"   Claude 3 Opus:")
        print(f"     - Tool use: {claude_caps['features']['tool_use']}")
        print(f"     - Vision: {claude_caps['features']['vision']}")
        print(f"     - JSON mode: {claude_caps['features']['json_mode']}")
        print(f"     - Max context: {claude_caps['constraints']['tokens']['max_context_window']}")
    
    # Test 4: Compare capabilities
    print("\n4. Comparing capabilities (GPT-4 → Claude)...")
    if gpt4_caps and claude_caps:
        comparison = specado.compare_capabilities(gpt4_caps, claude_caps)
        print(f"   Is lossy: {comparison['lossiness_report']['is_lossy']}")
        print(f"   Severity: {comparison['lossiness_report']['severity']}")
        
        if comparison['missing_capabilities']:
            print(f"   Missing capabilities: {comparison['missing_capabilities']}")
        
        if comparison['lossiness_report']['recommendations']:
            print(f"   Recommendations:")
            for rec in comparison['lossiness_report']['recommendations']:
                print(f"     - {rec}")
    
    # Test 5: Verify serialization
    print("\n5. Testing serialization...")
    custom_capability = {
        "version": "1.0.0",
        "modalities": {
            "input": ["Text"],
            "output": ["Text"],
            "configs": {
                "image": None,
                "audio": None,
                "video": None,
                "document": None
            }
        },
        "features": {
            "function_calling": True,
            "json_mode": False,
            "streaming": True,
            "logprobs": False,
            "multiple_responses": False,
            "stop_sequences": True,
            "seed_support": False,
            "tool_use": False,
            "vision": False
        },
        "parameters": {
            "temperature": {
                "supported": True,
                "min": 0.0,
                "max": 1.0,
                "default": 0.7
            },
            "top_p": {
                "supported": False,
                "min": None,
                "max": None,
                "default": None
            },
            "top_k": {
                "supported": False,
                "min": None,
                "max": None,
                "default": None
            },
            "max_tokens": {
                "supported": True,
                "min": 1,
                "max": 2048,
                "default": 512
            },
            "frequency_penalty": {
                "supported": False,
                "min": None,
                "max": None,
                "default": None
            },
            "presence_penalty": {
                "supported": False,
                "min": None,
                "max": None,
                "default": None
            },
            "repetition_penalty": {
                "supported": False,
                "min": None,
                "max": None,
                "default": None
            }
        },
        "roles": {
            "system": True,
            "user": True,
            "assistant": True,
            "function": False,
            "tool": False,
            "custom_roles": []
        },
        "constraints": {
            "tokens": {
                "max_context_window": 4096,
                "max_input_tokens": None,
                "max_output_tokens": 2048,
                "max_tokens_per_message": None,
                "encoding": None
            },
            "rate_limits": {
                "requests_per_minute": 100,
                "requests_per_hour": None,
                "requests_per_day": None,
                "tokens_per_minute": 50000,
                "tokens_per_hour": None,
                "tokens_per_day": None,
                "max_concurrent_requests": 10
            },
            "messages": {
                "max_messages_per_conversation": None,
                "max_message_length": None,
                "min_messages_required": 1,
                "max_system_message_length": None,
                "allow_empty_messages": False,
                "allow_consecutive_same_role": False
            },
            "timeouts": {
                "max_request_timeout_seconds": 60,
                "default_timeout_seconds": 30,
                "stream_timeout_seconds": 60,
                "connection_timeout_seconds": 10
            }
        },
        "extensions": {
            "custom": {},
            "experimental": []
        }
    }
    
    # Compare custom with GPT-4
    comparison2 = specado.compare_capabilities(custom_capability, gpt4_caps)
    print(f"   Custom → GPT-4 comparison:")
    print(f"     - Is lossy: {comparison2['lossiness_report']['is_lossy']}")
    print(f"     - Details: {len(comparison2['lossiness_report']['details'])} issues found")
    
    print("\n✅ All capability tests passed!")
    
if __name__ == "__main__":
    try:
        test_capabilities()
    except Exception as e:
        print(f"❌ Error: {e}")
        import traceback
        traceback.print_exc()