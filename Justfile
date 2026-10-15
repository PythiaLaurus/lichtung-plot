set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

# Format Rust sources.
fmt:
    cargo fmt --all

# Check Rust formatting without modifying files.
fmt-check:
    cargo fmt --all -- --check

# Compile and lint every target with warnings denied.
lint:
    cargo clippy --locked --offline --all-targets -- -D warnings

# Run library, integration, and example targets (excluding doctests).
test:
    cargo test --locked --offline --all-targets

# Run public examples and compile-fail API boundaries.
test-doc:
    cargo test --locked --offline --doc

# Build API documentation with warnings denied.
docs:
    RUSTDOCFLAGS="-D warnings" cargo doc --locked --offline --no-deps

# Verify the distributable package from Cargo's unpacked staging directory.
package:
    cargo package --locked --offline --allow-dirty

# Run the complete read-only repository verification.
verify: fmt-check lint test test-doc docs package
