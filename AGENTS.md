# Repository Guidelines

## Source of Truth

Treat `specs/` as the architectural source of truth. Prefer root `Justfile` recipes over ad hoc commands.

Keep shared specifications at the workspace root. Member documentation should reference shared contracts rather than duplicate them.

## Communication

Follow Grice's conversational maxims: be informative, truthful,
relevant, and clear. Prefer direct answers and concrete language.
Avoid unnecessary jargon, repetition, and verbosity.

Always present local file and directory paths as inline code,
never as hyperlinks or clickable file references.

Main-agent plan/build modes and mode switches do not affect subagents; never message subagents because of them.

## Architecture

`lichtung-plot` is a standalone scientific renderer. It must not depend on other `lichtung-*` crates. Adapters and bindings depend on its public API, never the reverse. Core-only consumers must not compile adapter or binding dependencies.

The core owns its errors, annotation types, rendering lifecycle, output contracts, and font policy. Do not introduce network access or asset provisioning into the core.

Keep Python binding code, tests, stubs, and examples in `bindings/python/`. Keep Rust development and test support within workspace members.

## API and Modules

Maintain a single current API and representation. When replacing project-owned APIs, migrate repository callers and remove obsolete implementations instead of adding compatibility layers.

Use `foo.rs` and `foo/` module layouts, not `mod.rs`. Expose modules publicly only when they are deliberate extension points.

## Verification

Run focused tests covering changed behavior and affected callers. Expand coverage only when justified by concrete failures or cross-cutting invariants. Do not run full test suites routinely.

Test semantic contracts rather than incidental rendering details. Avoid snapshot or golden updates that merely accept changed output. See `specs/90_DEVELOPMENT.md` for detailed testing conventions.

For code changes, run `cargo clippy --locked --offline --workspace --all-targets -- -D warnings` or `just lint`. Do not substitute `cargo check`. Documentation-only changes do not require tests.

Workspace verification recipes must explicitly cover the intended members. Package verification must exclude development-only members.

## Commits

Use scoped Conventional Commits, with one logical change per commit. Describe enduring behavior or architectural changes, not temporary planning stages.

Commit completed work before starting the next independent phase, unless explicitly instructed otherwise.
