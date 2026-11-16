set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

python := env_var_or_default("LICHTUNG_PYTHON", "python3")
maturin := env_var_or_default("LICHTUNG_MATURIN", "maturin")
cargo_about := env_var_or_default("LICHTUNG_CARGO_ABOUT", "cargo-about")

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

# Verify renderer/adapter packages, excluding the workspace-local Book command.
package:
    cargo package --locked --offline -p lichtung-plot -p lichtung-plot-ndarray -p lichtung-plot-polars --allow-dirty

# Exercise native atomic-output contracts, including target-specific handle rules.
test-output:
    cargo test --locked --offline -p lichtung-plot --lib orchestrator::artifacts::tests::

# Build the mixed Python package with already-installed maturin and Python.
python-build:
    env -u PYO3_BUILD_EXTENSION_MODULE -u PYO3_NO_PYTHON -u PYO3_CONFIG_FILE PYO3_PYTHON="{{python}}" "{{maturin}}" build --manifest-path bindings/python/Cargo.toml --locked --offline --release --interpreter "{{python}}"

# Test an installed wheel; isolated mode excludes source-tree package imports.
python-test *args:
    "{{python}}" -I -B -m pytest -q bindings/python/tests {{args}}

# Check installed declarations and public type boundaries, without API reflection.
python-typing:
    "{{python}}" -I -B bindings/python/tests/check_typing.py

# Use the PEP 517 source producer, including its projected-lock validation.
python-sdist out="target/python-sdist":
    LICHTUNG_MATURIN="{{maturin}}" "{{python}}" -I -B bindings/python/_build_backend.py --out "{{out}}"

# Inspect an actual archive against this checkout; never install or rebuild it.
python-artifact archive platform="" output="-":
    "{{python}}" -I -B bindings/python/check_artifact.py "{{archive}}" --platform "{{platform}}" --output "{{output}}"

# Offline maintainer-only notice verification; --write explicitly regenerates.
python-notices *args:
    "{{python}}" -I -B bindings/python/third_party.py --cargo-about "{{cargo_about}}" {{args}}

# Check peer tutorial anchors, image-to-code links and executable source includes.
book-check:
    cargo run --locked --offline -p lichtung-plot-book -- check

# Execute the gallery once and assemble fresh sources/artifacts for both Books.
gallery:
    cargo run --locked --offline -p lichtung-plot-book -- gallery

# Generate the gallery and build both single-page guides with the installed mdBook.
book:
    cargo run --locked --offline -p lichtung-plot-book -- build

# Run Rust/source workspace verification; installed-wheel checks are separate.
verify: fmt-check lint test test-doc docs book-check package
