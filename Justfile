set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

# Format Rust sources.
fmt:
    cargo fmt --all

# Check Rust formatting without modifying files.
fmt-check:
    cargo fmt --all -- --check

# Check every target.
check:
    cargo check --locked --offline --all-targets

# Lint every target.
lint:
    cargo clippy --locked --offline --all-targets -- -D warnings

# Run the complete test suite.
test:
    cargo test --locked --offline --all-targets

# Build API documentation with warnings denied.
docs:
    RUSTDOCFLAGS="-D warnings" cargo doc --locked --offline --no-deps

# Verify the distributable package from Cargo's unpacked staging directory.
package:
    cargo package --locked --offline --allow-dirty

# Run the complete read-only repository verification.
verify: fmt-check check lint test docs package
