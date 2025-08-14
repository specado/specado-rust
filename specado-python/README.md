# Specado Python Bindings

Python bindings for the Specado spec-driven LLM integration library.

## Installation

```bash
pip install specado
```

## Usage

```python
import specado

# Get a hello world message
message = specado.hello_world()
print(message)  # "Hello from Specado Core!"

# Check version
version = specado.version()
print(f"Specado version: {version}")
```

## Development

This package is built using [maturin](https://github.com/PyO3/maturin) and [PyO3](https://pyo3.rs/).

To build the package locally:

```bash
maturin develop
```

To run tests:

```bash
python test_specado.py
```

## License

Apache-2.0