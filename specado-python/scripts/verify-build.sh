#!/bin/bash
set -e

# Specado Python Build Verification Script
# Tests wheel building, installation, and basic functionality

echo "🔧 Specado Python Build Verification"
echo "===================================="

# Get the directory of this script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
TEMP_ENV_DIR="/tmp/specado-verify-$$"

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

cleanup() {
    if [ -d "$TEMP_ENV_DIR" ]; then
        log_info "Cleaning up temporary environment..."
        rm -rf "$TEMP_ENV_DIR"
    fi
}

# Cleanup on exit
trap cleanup EXIT

# Check dependencies
check_dependencies() {
    log_info "Checking dependencies..."
    
    if ! command -v python3 &> /dev/null; then
        log_error "python3 is required but not installed"
        exit 1
    fi
    
    if ! command -v maturin &> /dev/null; then
        log_error "maturin is required but not installed"
        echo "Install with: pip install maturin"
        exit 1
    fi
    
    PYTHON_VERSION=$(python3 --version | cut -d' ' -f2 | cut -d'.' -f1-2)
    log_info "Python version: $PYTHON_VERSION"
    
    MATURIN_VERSION=$(maturin --version | cut -d' ' -f2)
    log_info "Maturin version: $MATURIN_VERSION"
}

# Build wheel
build_wheel() {
    log_info "Building wheel..."
    cd "$PROJECT_DIR"
    
    # Clean previous builds
    rm -rf dist/ target/wheels/
    
    # Build the wheel
    if ! maturin build --release; then
        log_error "Failed to build wheel"
        exit 1
    fi
    
    # Check if wheel was created
    if [ ! -d "dist" ] || [ -z "$(ls -A dist/)" ]; then
        log_error "No wheel file found in dist/"
        exit 1
    fi
    
    WHEEL_FILE=$(ls dist/*.whl | head -1)
    log_info "Built wheel: $(basename "$WHEEL_FILE")"
    
    # Basic wheel inspection
    if command -v unzip &> /dev/null; then
        log_info "Wheel contents:"
        unzip -l "$WHEEL_FILE" | head -20
    fi
}

# Test installation in clean environment
test_installation() {
    log_info "Testing installation in clean environment..."
    
    # Create temporary virtual environment
    python3 -m venv "$TEMP_ENV_DIR"
    source "$TEMP_ENV_DIR/bin/activate"
    
    # Upgrade pip
    pip install --upgrade pip
    
    # Install the wheel
    WHEEL_FILE=$(ls "$PROJECT_DIR/dist"/*.whl | head -1)
    if ! pip install "$WHEEL_FILE"; then
        log_error "Failed to install wheel"
        exit 1
    fi
    
    log_info "Installation successful"
    
    # List installed packages
    log_info "Installed packages:"
    pip list | grep -E "(specado|pyo3)"
}

# Test basic functionality
test_functionality() {
    log_info "Testing basic functionality..."
    
    # Ensure we're in the virtual environment
    source "$TEMP_ENV_DIR/bin/activate"
    
    # Test basic import and functionality
    python3 << 'EOF'
import sys
print(f"Python path: {sys.path[0]}")

# Test basic import
try:
    import specado
    print("✅ Successfully imported specado")
except ImportError as e:
    print(f"❌ Failed to import specado: {e}")
    sys.exit(1)

# Test version
try:
    version = specado.version()
    print(f"✅ Specado version: {version}")
except Exception as e:
    print(f"❌ Failed to get version: {e}")
    sys.exit(1)

# Test Client creation
try:
    from specado import Client, Message
    client = Client()
    print("✅ Successfully created Client")
except Exception as e:
    print(f"❌ Failed to create Client: {e}")
    sys.exit(1)

# Test Message creation
try:
    msg = Message("user", "test message")
    print(f"✅ Successfully created Message: {msg.role} - {msg.content}")
except Exception as e:
    print(f"❌ Failed to create Message: {e}")
    sys.exit(1)

# Test basic API structure
try:
    # Test that client has chat.completions
    assert hasattr(client, 'chat'), "Client missing 'chat' attribute"
    assert hasattr(client.chat, 'completions'), "Chat missing 'completions' attribute"
    assert hasattr(client.chat.completions, 'create'), "Completions missing 'create' method"
    print("✅ API structure validation passed")
except Exception as e:
    print(f"❌ API structure validation failed: {e}")
    sys.exit(1)

print("🎉 All functionality tests passed!")
EOF

    if [ $? -ne 0 ]; then
        log_error "Functionality tests failed"
        exit 1
    fi
}

# Test wheel metadata
test_metadata() {
    log_info "Testing wheel metadata..."
    
    WHEEL_FILE=$(ls "$PROJECT_DIR/dist"/*.whl | head -1)
    
    # Extract metadata
    if command -v python3 &> /dev/null; then
        python3 << EOF
import zipfile
import sys

wheel_path = "$WHEEL_FILE"
try:
    with zipfile.ZipFile(wheel_path, 'r') as wheel:
        # Find metadata file
        metadata_files = [f for f in wheel.namelist() if f.endswith('METADATA')]
        if not metadata_files:
            print("❌ No METADATA file found in wheel")
            sys.exit(1)
        
        metadata_content = wheel.read(metadata_files[0]).decode('utf-8')
        print("✅ Wheel metadata found")
        
        # Check key metadata fields
        required_fields = ['Name:', 'Version:', 'Author:', 'License:']
        for field in required_fields:
            if field in metadata_content:
                print(f"✅ {field} present")
            else:
                print(f"❌ {field} missing")
        
        # Show first 20 lines of metadata
        lines = metadata_content.split('\n')[:20]
        print("\nMetadata preview:")
        print('\n'.join(lines))

except Exception as e:
    print(f"❌ Failed to read wheel metadata: {e}")
    sys.exit(1)
EOF
    fi
}

# Test with twine if available
test_twine_check() {
    if command -v twine &> /dev/null; then
        log_info "Running twine check..."
        WHEEL_FILE=$(ls "$PROJECT_DIR/dist"/*.whl | head -1)
        
        if twine check "$WHEEL_FILE"; then
            log_info "✅ Twine check passed"
        else
            log_warn "⚠️ Twine check failed - wheel may not be PyPI ready"
        fi
    else
        log_warn "twine not available - skipping PyPI readiness check"
        echo "Install with: pip install twine"
    fi
}

# Main execution
main() {
    log_info "Starting build verification process..."
    
    check_dependencies
    build_wheel
    test_metadata
    test_installation
    test_functionality
    test_twine_check
    
    log_info "🎉 Build verification completed successfully!"
    log_info "Your wheel is ready for distribution:"
    ls -la "$PROJECT_DIR/dist/"
}

# Run main function
main