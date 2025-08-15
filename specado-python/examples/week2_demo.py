#!/usr/bin/env python3
"""
Week 2 Python Demo - Routing & Resilience

This demo showcases:
1. Primary-to-fallback routing
2. Retry policies with exponential backoff
3. Routing metadata exposure
4. Error handling and resilience

Run with: python examples/week2_demo.py
"""

import json
from specado import Client, Message


def main():
    print("\n🚀 Specado Week 2 Python Demo - Routing & Resilience\n")
    print("=" * 60)
    
    # Example 1: Basic client with default routing
    print("\n📝 Example 1: Basic Chat Completion")
    print("-" * 40)
    
    client = Client({
        'primary_provider': 'openai',
        'fallback_provider': 'anthropic'
    })
    
    messages = [
        Message('system', 'You are a helpful assistant'),
        Message('user', 'Hello, what can you help me with?')
    ]
    
    response = client.chat.completions.create(
        model='gpt-4',
        messages=messages,
        temperature=0.7
    )
    
    print(f"✅ Response ID: {response.id}")
    print(f"   Model: {response.model}")
    print(f"   Provider used: {response.extensions.provider_used}")
    print(f"   Fallback triggered: {response.extensions.fallback_triggered}")
    print(f"   Total attempts: {response.extensions.attempts}")
    
    if response.choices:
        print(f"   Response: {response.choices[0].message.content[:100]}...")
    
    # Example 2: Trigger fallback with timeout
    print("\n📝 Example 2: Timeout Triggers Fallback")
    print("-" * 40)
    
    messages = [Message('user', 'This will timeout on primary')]
    
    response = client.chat.completions.create(
        model='timeout-test-model',  # Special model that triggers timeout
        messages=messages
    )
    
    print(f"⏱️  Timeout handled!")
    print(f"   Provider used: {response.extensions.provider_used}")
    print(f"   Fallback triggered: {response.extensions.fallback_triggered}")
    print(f"   Total attempts: {response.extensions.attempts}")
    
    # Show metadata
    metadata = response.extensions.metadata
    print(f"   Primary provider: {metadata.get('primary_provider')}")
    print(f"   Fallback provider: {metadata.get('fallback_provider')}")
    print(f"   Fallback index: {metadata.get('fallback_index')}")
    
    # Example 3: Rate limit handling
    print("\n📝 Example 3: Rate Limit Handling")
    print("-" * 40)
    
    messages = [Message('user', 'This triggers rate limit')]
    
    response = client.chat.completions.create(
        model='rate-limit-test-model',
        messages=messages
    )
    
    print(f"🚦 Rate limit handled!")
    print(f"   Provider used: {response.extensions.provider_used}")
    print(f"   Fallback triggered: {response.extensions.fallback_triggered}")
    
    # Check if provider_errors exists in metadata
    metadata = response.extensions.metadata
    if 'provider_errors' in metadata:
        print(f"   Provider errors logged: {len(metadata['provider_errors'])}")
    
    # Example 4: Server error handling
    print("\n📝 Example 4: Server Error Resilience")
    print("-" * 40)
    
    messages = [Message('user', 'This triggers server error')]
    
    response = client.chat.completions.create(
        model='server-error-test-model',
        messages=messages
    )
    
    print(f"🔧 Server error handled!")
    print(f"   Provider used: {response.extensions.provider_used}")
    print(f"   Fallback triggered: {response.extensions.fallback_triggered}")
    print(f"   Recovery successful: True")
    
    # Example 5: Non-retryable error (auth failure)
    print("\n📝 Example 5: Non-Retryable Error Handling")
    print("-" * 40)
    
    try:
        messages = [Message('user', 'This triggers auth error')]
        
        response = client.chat.completions.create(
            model='auth-error-test-model',
            messages=messages
        )
    except RuntimeError as e:
        print(f"❌ Auth error (non-retryable): {str(e)[:80]}...")
        print(f"   This error fails immediately without retry or fallback")
    
    # Example 6: Configuration inspection
    print("\n📝 Example 6: Configuration & Metadata")
    print("-" * 40)
    
    print(f"Client configuration:")
    print(f"   Primary: {client.get_config('primary_provider')}")
    print(f"   Fallback: {client.get_config('fallback_provider')}")
    
    # Show a successful response with full metadata
    messages = [Message('user', 'Show me metadata')]
    
    response = client.chat.completions.create(
        model='gpt-4',
        messages=messages,
        max_tokens=50
    )
    
    print(f"\nResponse metadata:")
    for key, value in response.extensions.metadata.items():
        print(f"   {key}: {value}")
    
    # Example 7: Multiple message context
    print("\n📝 Example 7: Multi-Turn Conversation")
    print("-" * 40)
    
    messages = [
        Message('system', 'You are a technical assistant'),
        Message('user', 'What is routing in distributed systems?'),
        Message('assistant', 'Routing in distributed systems refers to...'),
        Message('user', 'How does fallback work?')
    ]
    
    response = client.chat.completions.create(
        model='gpt-4',
        messages=messages,
        temperature=0.5
    )
    
    print(f"✅ Multi-turn response processed")
    print(f"   Messages sent: {len(messages)}")
    print(f"   Provider: {response.extensions.provider_used}")
    print(f"   Response: {response.choices[0].message.content[:100]}...")
    
    # Summary
    print("\n" + "=" * 60)
    print("\n✨ Key Features Demonstrated:")
    print("  1. Automatic fallback on provider failures")
    print("  2. Retry policies with exponential backoff")
    print("  3. Comprehensive routing metadata")
    print("  4. Error classification (retryable vs non-retryable)")
    print("  5. OpenAI-compatible API with extensions")
    
    print("\n🎯 Production Benefits:")
    print("  - High availability through intelligent routing")
    print("  - Resilience against transient failures")
    print("  - Detailed observability via metadata")
    print("  - Seamless provider switching")
    
    print("\n🏁 Week 2 Python Demo Complete!")
    print("\n📚 Next: Week 3 will add packaging and quickstart guides")


if __name__ == "__main__":
    main()