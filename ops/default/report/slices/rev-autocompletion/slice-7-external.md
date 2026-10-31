# Slice 7: Explicit external grammar admission

Status: reviewing
Delivery set: rev-autocompletion
Plan: [Native Autocompletion Delivery](../../../plan/rev-autocompletion.md)
Tracker: [Native Autocompletion Tracker](../../../tracker/rev-autocompletion.md)
Branch: `feat/rev-autocompletion-07-external`
PR: pending

## Purpose

Deliver CMP07, CMP09 and the approved R31 delta through explicit native asset
registration, retaining R35 and the existing checked completion boundary.

## Delivered Behavior

Startup TOML accepts a complete `language.<id>.external` registration containing
parser, symbol, highlights and completions. Existing associations select the
identity, including languages outside the bundled catalog. Relative paths resolve
under the resolved XDG data root's `rev/grammars`; absolute paths are accepted.
Parent escapes, escaping symlinks and nonregular assets fail admission.

First use queues one temporary admission worker. It reads the selected files,
loads the explicit native symbol, validates the parser ABI and compiles both
queries before publishing an immutable generation. At most 32 registrations and
1 MiB per nonempty UTF-8 query are admitted. One outcome is retained per
registration/session. No repeated failed load, startup directory discovery or
asset watcher is introduced. The loop polls only while a selected outcome needs
publication, then returns to its ordinary idle wait.

Admitted external syntax uses External status, including declared-language Org
source drafts. Its explicit declaration/scope query feeds the same completion
worker, source pins, checked literal edit and undo contract as bundled languages.
A new grammar generation retires observations from the fallback even if text and
path are unchanged. Failure retains bundled highlighting/completion, or plain
editing for an unbundled identity.

## Implementation Notes

The pinned safe `tree-sitter-loader = 0.27.0` API loads only the explicitly named
library/symbol. First-party roots retain forbid(unsafe_code); no custom FFI,
compiler invocation, downloader, CLI, Wasm or project discovery enters runtime.
The upstream API intentionally retains loaded mappings for process lifetime,
including a successful load followed by query rejection. Install another versioned
asset directory, update registration and restart; never overwrite mapped code.

The approved default-features=false build exposed an upstream 0.27.0 compilation
defect: LoaderError references TagsError and QueryError when their imports are
configured out. Keeping that exact pin and default-features=false, the manifest
explicitly enables only tree-sitter-tags. Cargo's feature graph confirms the tags
feature and one engine 0.27.1; highlight/Wasm defaults remain disabled. The
application still calls only Loader::load_language. The lock adds 12 packages,
including test-only Python; no previous locked package version changes.
Integrated executable size/resource measurement remains assigned to Slice 8.

Validated highlight/completion queries are shared immutable values; each buffer
retains its sole parser/tree. External completion uses a finite profile of seven
declaration kinds, explicit ancestor scopes and optional comment/string regions.
Unknown captures, predicates and property operations reject admission. Built-in
text predicates remain available. The existing 2 MiB source cutoff, 32-buffer/
8 MiB snapshot budget, 4096/16384 symbol limits, 8 MiB payload, 512-byte labels,
50 ms cancellable work and checked edit limit remain in effect.

## Contracts Added or Changed

The complete sample and exhaustive reference describe the four registration
leaves, atomic registration override, path/symbol/query bounds, finite captures,
native-code trust and replacement by restart. No default external registration is
activated. R31's named baseline restriction now points to its approved successor;
historical baseline decisions/gate evidence and R35 are preserved.

## Files of Interest

- `src/syntax/assets.rs`, `assets/admission.rs`, `assets/queue.rs`, `assets/context.rs`: one-time admission, safe load, profile validation and literal contexts.
- `src/config/external.rs`, `config/resolve.rs`, `config/merge.rs`: startup registration validation, resolved paths, immutable registry and provenance.
- `src/buffer/syntax.rs`, `src/session/assets.rs`, `src/completion/job.rs`: generation publication, active-only polling and shared completion ownership.
- `tests/support/language_assets.rs`, `tests/language_assets/`, `tests/completion/external.rs`: real precompiled fixtures and native workflows.
- `docs/reference/syntax.md`, `defaults.md`, `config.toml`: complete user-facing asset contract.

## Validation

T7.1 is commit `386dc82aaca1c3f8c5a4b87c539164237db71caa`.
T7.2 is commit `48f8ba4eee593e3157470a153df1e953c91c7c3b`.
Development checks reproduced the upstream feature defect before the explicit
feature correction. Cargo check, library Clippy, real-library integration and
fallback-generation diagnostics then passed. R35's complete requirement text was
compared against the predecessor and remains unchanged.

Focused controller receipt `EFW6JDQ6OJZPLSUTZFXO24KHKU` passed all eight checks at
clean candidate `48f8ba4eee593e3157470a153df1e953c91c7c3b`: six admission cases,
two external completion workflows, one generation invalidation test and 12 shared
completion library tests. Actual Rust/Python native libraries are compiled from
locked crate sources before editor construction using the owned test fixture.
The incompatible fixture changes only the generated parser ABI constant.
Coverage includes replacement highlighting, unbundled Python completion and
one-step undo, cached failures, missing/wrong-symbol/ABI/query assets, query
profile/UTF-8/size rejection, relative paths and retention after unlinking assets.
The fixture owners remove their temporary files after checks finish.

Source-quality checks cover the new/changed runtime owners listed below. The
unchanged document validator and event dispatcher retain their existing baseline
complexity and are outside that focused command. Documentation passed with 326
commands, 202 configuration entries, 60 reachable pages and 350 local links/anchors.

1. `cargo test --locked --test language_assets`: passed.
2. `cargo test --locked --test completion external::`: passed.
3. `cargo test --locked --lib syntax::assets`: passed.
4. `cargo test --locked --lib completion`: passed.
5. `cargo clippy --locked --lib -- -D warnings`: passed.
6. `cargo run --locked --quiet --example source-check -- --file src/config/external.rs --file src/session/assets.rs --file src/syntax/assets.rs --file src/syntax/assets/admission.rs --file src/syntax/assets/queue.rs --file src/syntax/assets/context.rs --file src/syntax/assets/tests.rs --file src/syntax.rs --file src/syntax/completion.rs --file src/syntax/scope.rs --file src/buffer/syntax.rs --file src/buffer/policy.rs --file src/config.rs --file src/config/resolve.rs --file src/config/merge.rs --file src/completion.rs --file src/completion/job.rs`: passed.
7. `cargo run --locked --quiet --example documentation-check`: passed.
8. `git diff --check`: passed.

The canonical PR will link the full immutable report-introduction commit. After
its PR-reference follow-up, the controller records fresh final clean-HEAD
validation in this exact configured order:

1. `cargo fmt --all -- --check`
2. `cargo test --locked --test completion --test language_assets --test syntax --test configuration --test files`
3. `git diff --check`

## Risks and Follow-ups

Native libraries are explicitly user trusted; ABI/query admission cannot contain
arbitrary native-code faults. Mappings remain for process lifetime. The external
profile provides syntax declarations and explicit scopes, with no receiver/type
inference or language-specific structural adapter. Integrated resource/PTY
acceptance and final guidance retain their approved Slice 8 boundary.
