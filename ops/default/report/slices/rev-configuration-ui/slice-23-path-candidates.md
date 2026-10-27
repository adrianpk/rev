# Slice 23: Observe bounded filesystem candidates

Status: delivered
Delivery set: rev-configuration-ui
Plan: [Configuration and Interaction Delivery](../../../plan/rev-configuration-ui.md)
Tracker: [Configuration and Interaction Tracker](../../../tracker/rev-configuration-ui.md)
Branch: `feat/rev-config-ui-23-path-candidates`
PR: [#72](https://forge.adrianpk.com/adrianpk/rev/pulls/72)

## Purpose

Provide one bounded, cached filesystem component observation with literal path
queries, safe display labels and independently retained opening identities.

## Delivered Behavior

- The file completion provider explicitly observes one absolute directory,
  including hidden names. Directories precede leaves; names sort by exact bytes.
  Child directories and symlink destinations are not enumerated.
- Each observation processes at most 4096 entries and retains at most 512 KiB of
  raw candidate-name bytes. A name that cannot fit ends observation. Reaching an
  exact cap conservatively reports an incomplete list without another entry read.
- Prefix matching and label/type access use cached memory. New files and removed
  names become visible only on explicit observation or refresh. Failed refresh
  preserves the previous snapshot, its directory identity and the IO error kind.
- IO-free queries use explicit absolute base/home roots. Relative and absolute
  paths, spaces, Unicode, `./`, `../`, trailing separators and leading `~/` retain
  literal semantics. Repeated slashes after `~/` stay rooted in home. Variables,
  shell-looking text, glob characters and other tilde forms do not expand.
- Paths remain independent of presentation. Directory markers and escaped
  controls affect only labels. Non-UTF-8 names remain visibly unsupported and
  cannot provide a lossy opening path. Ordinary Unicode identities remain exact.
- Provider construction and filtering do not visit buffers, publish files,
  recurse, create a watcher or perform background work.

## Implementation Notes

- T23.1: `e08a241e09e1bdce280e4619a4839f42ee3f2dc8` adds the bounded file-owned
  provider, cached literal filtering, candidate identities and ordering tests.
- T23.2: `abb2f921ab3d4e553057ade991ff1347fd31fb34` adds literal query parsing
  with explicit roots and Unicode, control-safe and unsupported-name regressions.
- T23.3: `5904932a76fe484dbd3d5d1d3906de6cf9649dca` adds real filesystem
  limit/error/symlink/refresh coverage and documents
  the public provider API in the existing reference index.
- Provider retains a single component. Callers request a new observation for a
  directory transition; redraw and filter operations do not refresh it implicitly.
- Incomplete observations sort only their bounded enumeration subset. They do
  not claim to contain the globally first names from an unbounded sorted listing.
- Entry type inspection does not follow child symlinks. Explicit directory paths
  retain OS resolution of symlink components and `..`; lexical normalization
  cannot silently redirect the observation to another parent.
- Raw names are bounded before candidate construction. Safe labels and paths
  add bounded per-entry storage; no file payloads are loaded. Boundary fixtures
  use roughly 14,400 empty files and complete in the focused suite in 0.20 seconds.

## Contracts Added or Changed

- UI03: reusable literal directory/component queries and retained path identities.
- UI04: bounded single-directory observations, explicit incomplete diagnostics,
  cached filtering, safe labels, unsupported-name rejection and no child traversal.
- Baseline R10-R13: the provider leaves the existing canonical file ingress,
  buffer identity, saved-state and publication owners unchanged.
- `rev::file::completion::{Query, Provider, Candidate}` is the concrete public
  library API. Query construction has no IO or environment dependency.

## Files of Interest

- `src/file/completion.rs`: bounded snapshot, sorting, matching and refresh.
- `src/file/completion/candidate.rs`: lossless paths, type evidence and safe labels.
- `src/file/completion/query.rs`: literal component parsing and explicit roots.
- `tests/path_completion.rs`, `path_completion/identity.rs`,
  `path_completion/observation.rs`: 13 native filesystem and identity regressions.
- `docs/reference/path-completion.md`, `docs/reference/index.md`: API reference.

## Validation

Successful development checks:

- `cargo test --test path_completion`: two initial ordering/cache tests passed.
- `cargo test --test path_completion identity::`: four query/identity tests passed.
- `cargo test --test path_completion identity::literal_queries`: repeated-slash home expansion passed.
- `cargo test --test path_completion observation::`: seven boundary tests passed, including real permission failures and multibyte-name byte limits.
- `cargo clippy --lib --test path_completion -- -D warnings`: passed.
- `cargo run --quiet --example source-check -- --source src/file/completion.rs --source src/file/completion/candidate.rs --root tests/path_completion.rs`: passed.
- `cargo run --quiet --example source-check -- --source src/file/completion/query.rs --source tests/path_completion/identity.rs`: passed.
- `cargo run --quiet --example source-check -- --source tests/path_completion/observation.rs`: passed; nesting at most two.

Controller evidence `ZCOJKRCUPP3WQMVGH3Y3TZ2LU2` passed all five ordered checks
at clean head `39cb2a1d1049409ec31ba8fccb7fb86a68cc2904`, including 13 native
filesystem tests with zero skips. PR #72 integrated into dev at that same head.
Acta merge event: `01M5Z2VGBE4P1KW5MZ2BAHSB3R`. Ordered commands:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test path_completion`
- `python3 scripts/check-source.py`
- `git diff --check`

## Risks and Follow-ups

Find-file initialization, vertical panel integration and interactive path
navigation remain assigned to Slice 24; selected/new-file acceptance remains
assigned to Slice 25. This provider layer does not change startup key behavior.
