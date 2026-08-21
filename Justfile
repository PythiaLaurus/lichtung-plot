set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

# Format Rust sources.
fmt:
    cargo fmt --all

# Check Rust formatting without modifying files.
fmt-check:
    cargo fmt --all -- --check

# Check the default SVG-only build.
check:
    cargo check --locked --offline --all-targets

# Check SVG and PNG support together.
check-png:
    cargo check --locked --offline --all-targets --features png

# Lint the default SVG-only build.
lint:
    cargo clippy --locked --offline --all-targets -- -D warnings

# Lint SVG and PNG support together.
lint-png:
    cargo clippy --locked --offline --all-targets --features png -- -D warnings

# Run the default SVG-only test suite.
test:
    cargo test --locked --offline --all-targets

# Run the SVG and PNG test suite.
test-png:
    cargo test --locked --offline --all-targets --features png

# Verify repository structure and the standalone dependency boundary.
policy:
    python3 scripts/check_repository.py
    python3 -m unittest discover -s scripts/tests -p 'test_*.py'

# Build API documentation with warnings denied.
docs:
    RUSTDOCFLAGS="-D warnings" cargo doc --locked --offline --all-features --no-deps

# Verify the distributable package from Cargo's unpacked staging directory.
package:
    cargo package --locked --offline --allow-dirty

# Run the complete read-only repository verification.
verify: fmt-check policy check check-png lint lint-png test test-png docs package
