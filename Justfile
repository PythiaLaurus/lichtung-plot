set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

# Format Rust sources.
fmt:
    cargo fmt --all

# Check Rust formatting without modifying files.
fmt-check:
    cargo fmt --all -- --check

# Compile and lint every workspace member and target with warnings denied.
lint:
    cargo clippy --locked --offline --workspace --all-targets -- -D warnings

# Run every member's library, integration, and example targets (excluding doctests).
test:
    cargo test --locked --offline --workspace --all-targets

# Run public examples and compile-fail API boundaries.
test-doc:
    cargo test --locked --offline --workspace --doc

# Build API documentation with warnings denied.
docs:
    RUSTDOCFLAGS="-D warnings" cargo doc --locked --offline --workspace --no-deps

# Verify every workspace package from Cargo's unpacked staging directories.
package:
    cargo package --locked --offline --workspace --allow-dirty

# Run the complete read-only repository verification.
verify: fmt-check lint test test-doc docs package
