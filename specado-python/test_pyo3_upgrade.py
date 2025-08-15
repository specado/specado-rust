#!/usr/bin/env python3
"""
Test script to verify PyO3 v0.25.1 upgrade works correctly
"""

# Note: This would be run after building with maturin develop

def test_basic_import():
    """Test that the module can be imported"""
    try:
        import specado
        print("✅ Module import successful")
        return True
    except ImportError as e:
        print(f"❌ Module import failed: {e}")
        return False

def test_version():
    """Test version function"""
    try:
        import specado
        version = specado.version()
        print(f"✅ Version: {version}")
        assert isinstance(version, str)
        assert len(version) > 0
        return True
    except Exception as e:
        print(f"❌ Version test failed: {e}")
        return False

def test_client_creation():
    """Test Client creation"""
    try:
        from specado import Client, Message
        
        # Create client with default config
        client = Client()
        print("✅ Client created with default config")
        
        # Create client with custom config
        config = {
            'primary_provider': 'openai',
            'fallback_provider': 'anthropic'
        }
        client = Client(config)
        print("✅ Client created with custom config")
        
        # Check config access
        primary = client.get_config('primary_provider')
        assert primary == 'openai'
        print(f"✅ Config access works: primary={primary}")
        
        # Check config_keys method
        keys = client.config_keys()
        assert 'primary_provider' in keys
        print(f"✅ Config keys: {keys}")
        
        return True
    except Exception as e:
        print(f"❌ Client creation test failed: {e}")
        return False

def test_message_creation():
    """Test Message creation"""
    try:
        from specado import Message
        
        msg = Message('user', 'Hello, world!')
        assert msg.role == 'user'
        assert msg.content == 'Hello, world!'
        print("✅ Message creation works")
        
        # Test repr
        repr_str = repr(msg)
        assert 'Message' in repr_str
        assert 'user' in repr_str
        print(f"✅ Message repr: {repr_str}")
        
        # Test str
        str_repr = str(msg)
        assert 'user' in str_repr
        assert 'Hello, world!' in str_repr
        print(f"✅ Message str: {str_repr}")
        
        return True
    except Exception as e:
        print(f"❌ Message creation test failed: {e}")
        return False

def test_chat_completion():
    """Test chat completion (mock)"""
    try:
        from specado import Client, Message
        
        client = Client()
        messages = [
            Message('system', 'You are a helpful assistant'),
            Message('user', 'Hello')
        ]
        
        # This would make a real call in production
        # For testing, it uses mock responses
        response = client.chat.completions.create(
            model='gpt-4',
            messages=messages,
            temperature=0.7,
            max_tokens=100
        )
        
        # Check response structure
        assert hasattr(response, 'id')
        assert hasattr(response, 'model')
        assert hasattr(response, 'choices')
        assert hasattr(response, 'extensions')
        print("✅ Chat completion response structure valid")
        
        # Check extensions
        ext = response.extensions
        assert hasattr(ext, 'provider_used')
        assert hasattr(ext, 'fallback_triggered')
        assert hasattr(ext, 'attempts')
        print(f"✅ Extensions: provider={ext.provider_used}, fallback={ext.fallback_triggered}, attempts={ext.attempts}")
        
        # Check metadata
        metadata = ext.metadata
        assert isinstance(metadata, dict)
        print(f"✅ Metadata type: {type(metadata)}")
        
        # Check repr methods
        ext_repr = repr(ext)
        assert 'Extensions' in ext_repr
        print(f"✅ Extensions repr: {ext_repr}")
        
        resp_repr = repr(response)
        assert 'ChatCompletionResponse' in resp_repr
        print(f"✅ Response repr: {resp_repr}")
        
        resp_str = str(response)
        assert 'ChatCompletion' in resp_str
        print(f"✅ Response str: {resp_str}")
        
        return True
    except Exception as e:
        print(f"❌ Chat completion test failed: {e}")
        import traceback
        traceback.print_exc()
        return False

def main():
    print("🧪 Testing PyO3 v0.25.1 Upgrade\n")
    print("=" * 50)
    
    tests = [
        ("Module Import", test_basic_import),
        ("Version Function", test_version),
        ("Client Creation", test_client_creation),
        ("Message Creation", test_message_creation),
        ("Chat Completion", test_chat_completion),
    ]
    
    passed = 0
    failed = 0
    
    for name, test_func in tests:
        print(f"\n📝 Testing: {name}")
        print("-" * 30)
        if test_func():
            passed += 1
        else:
            failed += 1
    
    print("\n" + "=" * 50)
    print(f"\n📊 Results: {passed} passed, {failed} failed")
    
    if failed == 0:
        print("✅ All tests passed! PyO3 v0.25.1 upgrade successful!")
    else:
        print(f"❌ {failed} tests failed. Please review the errors above.")
    
    return failed == 0

if __name__ == "__main__":
    import sys
    sys.exit(0 if main() else 1)