# Bevy Retro Shaders Task Runner

# Show this help message
default:
    @just --list

# Build the WebAssembly demo using Trunk (2D)
[group('web')]
web-build-2d:
    cd web && trunk build index.html

# Build the WebAssembly demo using Trunk (3D)
[group('web')]
web-build-3d:
    cd web && trunk build 3d.html

# Serve the 2D web demo locally with hot reloading
[group('web')]
web-serve:
    cd web && trunk serve

# Serve the 3D web demo locally with hot reloading
[group('web')]
web-serve-3d:
    cd web && trunk serve 3d.html --port 8081

# Run the 2D CRT interactive example (Native)
[group('examples')]
run-2d:
    cargo run --example crt_example --features "jpeg,hot_reload"

# Run the 3D interactive example with PBR (Native)
[group('examples')]
run-3d:
    cargo run --example crt_3d_example

# Build all targets (Native examples + Web demo)
[group('common')]
build-all:
    cargo build
    cd web && trunk build index.html
    cd web && trunk build 3d.html

# Clean all build artifacts and distribution folders
[group('common')]
clean:
    cargo clean
    rm -rf web/dist

# Format Rust code
[group('quality')]
fmt:
    cargo fmt --all

# Lint Rust code
[group('quality')]
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Check for outdated dependencies
[group('quality')]
outdated:
    cargo outdated --depth 1

# Check all quality groups
[group('quality')]
check: fmt lint test doc audit outdated

# Run tests
[group('tests')]
test:
    cargo test --all-targets --all-features

# Generate documentation
[group('doc')]
doc:
    cargo doc --no-deps --all-features

# Run security audit
[group('security')]
audit:
    cargo audit
