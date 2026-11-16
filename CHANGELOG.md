# Changelog

## 0.1.0 — Unreleased

The first coordinated Rust and Python release is in preparation. Registry
uploads, the `v0.1.0` tag and a GitHub Release require separate approval; this
entry does not announce publication.

### Included

- A standalone Rust renderer with 25 scientific plot families, facets, Canvas
  composition, themes, text layout and caller-supplied inferential annotations.
- Frozen rendered scenes with outlined SVG, preserve-text SVG and PNG output.
- Independent ndarray and eager Polars adapters consuming the renderer's public API.
- A typed Python package with native list/tuple, NumPy, pandas and eager Polars
  input routes, reusable Plot/Canvas owners and immutable configuration values.
- Peer English and Chinese guides with runnable synthetic examples.

### Requirements And Limits

- Rust 1.96; ordinary CPython 3.10 or newer with an `abi3-py310` extension.
  PyPy and free-threaded Python are not certified.
- Binary platforms and minimum OS/libc versions remain subject to artifact
  verification and release approval; no complete platform matrix is claimed yet.
- NumPy, pandas and Polars are optional input providers. The Python Polars bridge
  requires `polars==1.41.2` and `polars-runtime-32==1.41.2`, without PyArrow.
- Callers install fonts covering their visible text. The renderer never downloads
  fonts or assets and does not perform application inferential analysis.

Local build results do not certify unverified platforms or authorize publication.
