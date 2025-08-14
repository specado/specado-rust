#!/usr/bin/env python3
"""
Test module for Specado Python bindings.

This module tests the FFI bindings between Python and the Rust core library.
"""

import sys
import os
import unittest

# Add the target directory to the path so we can import the compiled module
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'target', 'debug'))
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'target', 'release'))

try:
    import specado
except ImportError as e:
    print(f"Warning: Could not import specado module: {e}", file=sys.stderr)
    print("Make sure to build the Python module first with: maturin develop", file=sys.stderr)
    # Allow tests to be imported even if module isn't built yet
    specado = None


class TestSpecadoBindings(unittest.TestCase):
    """Test cases for Specado Python bindings."""
    
    def setUp(self):
        """Set up test fixtures."""
        if specado is None:
            self.skipTest("Specado module not available. Build with: maturin develop")
    
    def test_hello_world(self):
        """Test the hello_world function returns expected message."""
        result = specado.hello_world()
        self.assertEqual(result, "Hello from Specado Core!")
        self.assertIsInstance(result, str)
    
    def test_version(self):
        """Test the version function returns a valid version string."""
        result = specado.version()
        self.assertIsInstance(result, str)
        self.assertTrue(len(result) > 0)
        # Version should be in semver format
        parts = result.split('.')
        self.assertGreaterEqual(len(parts), 2)
    
    def test_module_version_attribute(self):
        """Test that the module has a __version__ attribute."""
        self.assertTrue(hasattr(specado, '__version__'))
        self.assertEqual(specado.__version__, specado.version())


def main():
    """Main entry point for running tests."""
    unittest.main(verbosity=2)


if __name__ == '__main__':
    main()