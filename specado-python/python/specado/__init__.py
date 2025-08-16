"""Specado - Spec-driven LLM integration library.

This module provides Python bindings for the Specado core library,
enabling spec-driven integration with various LLM providers.
"""

from specado.specado import (
    Client, Message, ChatCompletionResponse, Extensions,
    version, __version__,
    # Capability functions
    get_openai_manifest, get_anthropic_manifest,
    compare_capabilities, get_model_capabilities
)

__all__ = [
    "Client", "Message", "ChatCompletionResponse", "Extensions",
    "version", "__version__",
    # Capability functions
    "get_openai_manifest", "get_anthropic_manifest",
    "compare_capabilities", "get_model_capabilities"
]
