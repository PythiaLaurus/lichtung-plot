set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

# Format Rust sources.
fmt:
    cargo fmt --all

# Check Rust formatting without modifying files.
fmt-check:
    cargo fmt --all -- --check

# Check the minimal SVG-only build.
check:
    cargo check --locked --offline --all-targets --no-default-features

# Check the default SVG and PNG build.
check-png:
    cargo check --locked --offline --all-targets

# Lint the minimal SVG-only build.
lint:
    cargo clippy --locked --offline --all-targets --no-default-features -- -D warnings

# Lint the default SVG and PNG build.
lint-png:
    cargo clippy --locked --offline --all-targets -- -D warnings

# Run the minimal SVG-only test suite.
test:
    cargo test --locked --offline --all-targets --no-default-features

# Run the default SVG and PNG test suite.
test-png:
    cargo test --locked --offline --all-targets

# Build API documentation with warnings denied.
docs:
    RUSTDOCFLAGS="-D warnings" cargo doc --locked --offline --all-features --no-deps

# Verify the distributable package from Cargo's unpacked staging directory.
package:
    cargo package --locked --offline --allow-dirty

# Run the complete read-only repository verification.
verify: fmt-check check check-png lint lint-png test test-png docs package
