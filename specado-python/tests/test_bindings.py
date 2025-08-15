"""
Tests for Specado Python bindings Week 2 functionality
"""
import pytest
import specado
from specado import Client, Message


def test_client_creation():
    """Test creating a client with default configuration"""
    client = Client()
    assert client is not None
    assert hasattr(client, 'chat')
    assert hasattr(client.chat, 'completions')


def test_client_with_config():
    """Test creating a client with custom configuration"""
    config = {
        'primary_provider': 'openai',
        'fallback_provider': 'anthropic'
    }
    client = Client(config)
    assert client.get_config('primary_provider') == 'openai'
    assert client.get_config('fallback_provider') == 'anthropic'


def test_message_creation():
    """Test creating message objects"""
    msg = Message('user', 'Hello, world!')
    assert msg.role == 'user'
    assert msg.content == 'Hello, world!'
    
    # Test message representation
    repr_str = repr(msg)
    assert 'Message' in repr_str
    assert 'user' in repr_str


def test_chat_completion_basic():
    """Test basic chat completion request"""
    client = Client()
    
    messages = [
        Message('system', 'You are a helpful assistant'),
        Message('user', 'Hello')
    ]
    
    response = client.chat.completions.create(
        model='gpt-4',
        messages=messages
    )
    
    assert response is not None
    assert hasattr(response, 'id')
    assert hasattr(response, 'model')
    assert hasattr(response, 'choices')
    assert hasattr(response, 'extensions')
    
    # Check extensions
    assert hasattr(response.extensions, 'provider_used')
    assert hasattr(response.extensions, 'fallback_triggered')
    assert hasattr(response.extensions, 'attempts')
    assert hasattr(response.extensions, 'metadata')


def test_chat_completion_with_params():
    """Test chat completion with temperature and max_tokens"""
    client = Client()
    
    messages = [
        Message('user', 'Tell me a story')
    ]
    
    response = client.chat.completions.create(
        model='gpt-4',
        messages=messages,
        temperature=0.7,
        max_tokens=100
    )
    
    assert response is not None
    assert response.model == 'gpt-4'
    assert len(response.choices) > 0
    
    # Check first choice
    choice = response.choices[0]
    assert hasattr(choice, 'message')
    assert hasattr(choice, 'finish_reason')
    assert choice.message.role == 'assistant'


def test_fallback_triggered():
    """Test that fallback is triggered on primary failure"""
    client = Client({
        'primary_provider': 'openai',
        'fallback_provider': 'anthropic'
    })
    
    # Use special model that triggers timeout in demo implementation
    messages = [Message('user', 'Test fallback')]
    
    response = client.chat.completions.create(
        model='timeout-test-model',
        messages=messages
    )
    
    assert response is not None
    assert response.extensions.fallback_triggered == True
    assert response.extensions.provider_used == 'anthropic'
    assert response.extensions.attempts > 1


def test_rate_limit_handling():
    """Test rate limit error handling"""
    client = Client()
    
    messages = [Message('user', 'Test rate limit')]
    
    response = client.chat.completions.create(
        model='rate-limit-test-model',
        messages=messages
    )
    
    # Should fall back to anthropic
    assert response.extensions.fallback_triggered == True
    assert response.extensions.provider_used == 'anthropic'


def test_metadata_tracking():
    """Test that routing metadata is properly tracked"""
    client = Client()
    
    messages = [Message('user', 'Test metadata')]
    
    response = client.chat.completions.create(
        model='gpt-4',
        messages=messages
    )
    
    # Check metadata
    metadata = response.extensions.metadata
    assert 'primary_provider' in metadata
    assert 'fallback_used' in metadata
    assert metadata['fallback_used'] == False
    
    # Primary should succeed
    assert response.extensions.provider_used == 'openai'
    assert response.extensions.fallback_triggered == False


def test_auth_error_non_retryable():
    """Test that auth errors are not retried"""
    client = Client()
    
    messages = [Message('user', 'Test auth error')]
    
    with pytest.raises(RuntimeError) as exc_info:
        client.chat.completions.create(
            model='auth-error-test-model',
            messages=messages
        )
    
    assert 'Routing failed' in str(exc_info.value)


def test_version():
    """Test version function"""
    version = specado.version()
    assert version is not None
    assert isinstance(version, str)
    assert len(version) > 0


def test_module_version():
    """Test module __version__ attribute"""
    assert hasattr(specado, '__version__')
    assert specado.__version__ is not None
    assert isinstance(specado.__version__, str)


def test_multiple_messages():
    """Test chat completion with multiple messages"""
    client = Client()
    
    messages = [
        Message('system', 'You are a helpful assistant'),
        Message('user', 'Hello'),
        Message('assistant', 'Hi there!'),
        Message('user', 'How are you?')
    ]
    
    response = client.chat.completions.create(
        model='gpt-4',
        messages=messages
    )
    
    assert response is not None
    assert len(response.choices) > 0
    assert response.extensions.attempts >= 1


def test_client_repr():
    """Test client string representation"""
    config = {
        'primary_provider': 'openai',
        'fallback_provider': 'anthropic'
    }
    client = Client(config)
    repr_str = repr(client)
    assert 'Client' in repr_str
    assert 'providers' in repr_str


def test_response_repr():
    """Test response string representation"""
    client = Client()
    
    messages = [Message('user', 'Test')]
    response = client.chat.completions.create(
        model='gpt-4',
        messages=messages
    )
    
    repr_str = repr(response)
    assert 'ChatCompletionResponse' in repr_str
    assert 'gpt-4' in repr_str
    assert 'openai' in repr_str


def test_server_error_fallback():
    """Test that server errors trigger fallback"""
    client = Client()
    
    messages = [Message('user', 'Test server error')]
    
    response = client.chat.completions.create(
        model='server-error-test-model',
        messages=messages
    )
    
    # Should fall back to anthropic
    assert response.extensions.fallback_triggered == True
    assert response.extensions.provider_used == 'anthropic'
    assert response.extensions.attempts > 1