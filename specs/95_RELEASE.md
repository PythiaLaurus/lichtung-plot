# Release Process And 0.1.0 Closeout

## Scope And Authorization

Release preparation, artifact verification and external publication are separate
operations. This process does not authorize a push, remote tag, TestPyPI upload,
registry upload, GitHub Release or documentation deployment. Each external action
requires explicit approval of its destination, source commit and artifact set.
Never place credentials in source, logs, release notes or chat.

The public workspace must build, package and validate without the internal
repository. Internal acceptance can provide separately recorded evidence, not
required package inputs or a hidden public-CI dependency. Keep the standalone core,
font policy, scientific assets and single current API unchanged during closeout.

## Release Identity And Deliverables

Use one coordinated product version and one immutable source commit:

| Deliverable | Registry identity | 0.1.0 target |
| --- | --- | --- |
| Rust renderer | crates.io `lichtung-plot` | `0.1.0` |
| ndarray adapter | crates.io `lichtung-plot-ndarray` | `0.1.0` |
| Polars adapter | crates.io `lichtung-plot-polars` | `0.1.0` |
| Python distributions | PyPI `lichtung-plot`, import `lichtung_plot` | `0.1.0` |
| Public source/release notes | Git tag / GitHub Release | `v0.1.0` |

The Python member's Cargo package is implementation support, not a fourth
crates.io upload. It and the Book command retain `publish = false`; the Book
command's development version is not part of the coordinated product version.
An ordinary `0.1.0` release is not a SemVer prerelease merely because it is below
1.0. Bug fixes use a new patch version; incompatible pre-1.0 API changes require
an explicitly documented minor-version decision.

Every product manifest and built artifact must agree on the version. Rust
inter-member dependencies retain both a local path and a registry version.
Source provenance, versions and package membership are verified from actual
archives, not inferred from filenames or an old installed wheel.

## Supported Platforms And Runtime Requirements

The proposed first binary targets are Linux x86_64, Windows x86_64 and macOS
arm64. This is a candidate matrix, not a support certificate or permission to
omit other targets silently. Approve the final matrix and minimum OS/libc
versions before release; each advertised target requires its own real evidence.

Ordinary CPython starts at 3.10 with `abi3-py310`. A stable-ABI tag can reduce the
number of wheels, but does not prove support for every later Python, PyPy,
free-threaded interpreters, CPU or OS. Test the supported minimum and selected
newer CPython versions with admitted provider versions. Native list/tuple use
must remain independent of optional NumPy, pandas and Polars installations.

Linux wheels require verified manylinux compatibility, an explicit libc floor,
CPU baseline and external-library audit. A local `linux_x86_64` wheel is not a
PyPI-portability certificate. Do not relabel a binary or skip the audit to make
it appear compatible. Use approved, pinned build images/tools; this process
does not automatically download them.

Wheels contain the native extension; installing a matching wheel does not
require installing the Rust crates or a Rust compiler. Source builds require
the documented Rust/Python build tools and dependencies. Rendering requires
installed fonts covering the requested text; neither installation nor rendering
downloads or provisions fonts. Record optional-provider constraints, including
the eager Polars/runtime pair, separately from Python and platform compatibility.

## Closeout Work And Exit Criteria

Commit each completed logical phase before starting the next one. Preserve
dependency pins unless a concrete release blocker requires an approved change.
Do not grow the public test suite or restore retired reflection/matrix workflows
merely to create a release checklist.

### 1. Repository And Account Preparation

- Confirm the public repository URL, package names, version availability,
  ownership and maintainer access. A pending publisher does not reserve a name.
- Synchronize version metadata, repository/documentation/issue URLs, package
  READMEs and changelog. Keep release notes conditional until publication occurs.
- Include the exact MIT and Apache-2.0 texts in every applicable distribution;
  audit required third-party notices. SPDX metadata alone is not the texts.
- Check that archives contain required source, typing, examples and local
  fixtures, but no internal paths, credentials, caches or full external corpora.
- Prepare crates.io authentication with a verified account and a short-lived,
  least-privilege token for the initial manual publication. PyPI requires an
  account with two-factor authentication; prefer Trusted Publishing.

### 2. Candidate Source Verification

- Freeze a clean candidate commit and record toolchain, lockfile hash, selected
  interpreter/providers and local font prerequisites.
- Run the root source gate defined in `specs/90_DEVELOPMENT.md` once for the final
  release candidate. It covers all intended members, not only the default core.
- Verify the three distributable Rust packages in a fresh package target.
  Exclude development-only members. Do not use `--no-verify`, hand-built registry
  substitutes or production API changes to repair packaging.
- Record each stage's actual result. A failed aggregate command is not successful
  merely because a later focused retry repaired its final stage. After a local
  repair, rerun affected stages rather than repeating successful whole suites.

### 3. Python Source And Binary Verification

- Produce the sdist through `just python-sdist` or the configured PEP 517 backend.
  Bare `maturin sdist` bypasses this project's projected-lock validation and must
  not be substituted.
- Independently unpack that sdist and build a locked/offline wheel without
  reaching into the original checkout. This is distinct from an in-checkout build.
- Build and audit wheels for every approved binary target, preferably from the
  same verified sdist. Keep build, audit and upload jobs separate.
- Install the new distributions into isolated environments. Execute maintained
  runtime and typing checks plus representative real SVG/PNG consumers. Keep
  installed-package imports separate from source-tree imports.
- Inspect actual wheel/sdist versions, metadata, licenses, typing payload,
  dependency declarations, source closure, platform tags and dynamic libraries.
  Check README/metadata rendering with an explicitly selected installed tool.
- Record filenames, SHA-256, source commit, tools, target/CPU/libc floors and test
  results. Missing providers, ignored corpora or unexecuted targets are not passes.

### 4. Publishing Rehearsal And Approval

An optional TestPyPI rehearsal is a separately approved external upload, not part
of local verification. Install from TestPyPI with explicitly prepared dependencies;
do not use an unrestricted extra index that mixes package sources.

Collect the approved artifacts in a clean release-specific directory. Verify its
exact membership and hashes before upload; do not upload a broad output-directory
wildcard containing old builds. Python publishing uploads those same validated
bytes without rebuilding. Rust publishing uses the frozen source commit and
checks the resulting crate content/provenance. Do not treat a workstation lock
as a guarantee of every downstream dependency resolution.

The readiness review must include all required artifacts and platform evidence,
an approved changelog, registry/account checks and a documented recovery path.
Only then approve remote publication.

## External Publication Sequence

1. Approve and push the candidate source and its annotated `v0.1.0` tag. Verify
   that the tag still identifies the tested commit; never publish moving `main`.
2. Publish the Rust core, confirm registry/index visibility, then publish the
   adapters. Upload only the three explicitly selected crates:

   ```sh
   cargo publish --locked --registry crates-io -p lichtung-plot
   cargo publish --locked --registry crates-io -p lichtung-plot-ndarray
   cargo publish --locked --registry crates-io -p lichtung-plot-polars
   ```

   These are networked publication commands, not offline verification recipes.
3. Upload the approved Python sdist and complete wheel set to PyPI. Prefer a
   GitHub Actions Trusted Publisher restricted to the repository, workflow and
   protected publishing environment, with job-level `id-token: write` and manual
   approval. New PyPI projects can use a pending publisher. Do not use a command
   that rebuilds and immediately uploads an unverified binary.
4. Install each published product from its real registry into fresh external
   consumers, with no checkout path/patch overrides. Check versions and real
   representative output; inspect docs.rs/package-index metadata too.
5. Publish the prepared GitHub Release only after registry checks succeed.
   Attach the validated artifacts, hashes, release notes, compatibility limits
   and registry/documentation links. GitHub's automatic source archive is not
   the project's verified Python sdist.

GitHub Releases, Git tags, crates.io and PyPI are distinct operations, not an
atomic transaction. Draft notes/assets before finalizing an immutable GitHub
Release. Documentation deployment, if wanted, is another explicitly approved
operation, not an implied consequence of `just book`.

## Automation And Recovery

Start with explicit/manual release approval, not automatic uploads on ordinary
pushes or pull requests. Pin Actions and build tools, minimize permissions, and
keep untrusted PR jobs away from publishing credentials. A later crates.io Trusted
Publisher can replace manual tokens after the initial crates are established.
Do not create a second renderer, generic release framework or implicit tool/asset
installer to support automation.

For a transient failure, inspect which uploads succeeded and resume only missing
items from the same frozen source/artifact set. Do not silently skip an existing
file without checking its hash. If code or an uploaded artifact is defective,
stop the announcement, make a new version and consider yanking the defective
registry release. Published crate versions and uploaded PyPI files are not
replaceable; yanking does not erase history. Never move an already public release
tag to hide a different source commit.

## 0.1.0 Readiness Record

This document is the release procedure, not an assertion that 0.1.0 is published.
Keep executed evidence with exact commit/artifact hashes, separate from the
remaining checklist. Before any external publication, confirm:

- [ ] Coordinated versions, metadata and included license texts validated.
- [ ] Clean final source and all three independently unpacked Rust archives pass.
- [ ] New Python sdist builds independently; fresh installed runtime/typing pass.
- [ ] Approved platform matrix, libc/CPU floors and complete audited wheel set pass.
- [ ] Account ownership, names/version availability and publishing approval confirmed.
- [ ] Release notes, immutable source identity and exact artifact/hash list approved.
- [ ] Registry uploads and fresh registry consumers pass before public announcement.

## References

- [Cargo publishing](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [Maturin distribution](https://www.maturin.rs/distribution.html)
- [PyPI Trusted Publishing](https://docs.pypi.org/trusted-publishers/)
- [GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases)
