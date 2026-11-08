# Slice 6: Explicit persistent library

Status: reviewing
Delivery set: rev-keyboard-macros
Plan: [Keyboard Macros Delivery](../../../plan/rev-keyboard-macros.md)
Tracker: [Keyboard Macros Tracker](../../../tracker/rev-keyboard-macros.md)
Branch: `feat/rev-keyboard-macros-06-library`
PR: pending
Integrated: pending
Implementation: pending

## Purpose

Deliver T6.1-T6.3: explicit private schema-1 persistence, inert startup/reload and
TOML export with checked record concurrency and preservation of session values.

## Delivered Behavior

- `kmacro-save` selects a named snapshot and confirms publication. Any supplied
  prefix includes its admitted source binding. Unprefixed replacement announces
  removal of a previously saved binding; the current session binding remains.
- Startup and `kmacro-load` read the fixed library without execution or directory
  creation. Structurally invalid stores remain untouched. After complete
  validation, conflicting names are quarantined independently and conflicting
  bindings remain inactive while their named definitions remain available.
- Reload preserves unsaved bodies, descriptions, working counters and bindings.
  `kmacro-delete` confirms removal of exactly one saved identity/revision while
  retaining independent session/ring snapshots and other concern-owned stores.
- `insert-kbd-macro` inserts a valid single-record TOML document through normal
  source editing and undo. It performs no library write. Ordinary naming,
  binding, replay, ring changes and exit also perform no implicit save.

## Implementation Notes

The concrete library owner resolves only STATE/macros/library.toml and its
library.toml.lock. Native descriptor-based NOFOLLOW directory traversal, private
0700/0600 admission, nonblocking locking and the existing Temporary/Output
publication primitives protect staging and atomic replacement. Each write rereads
and validates the bounded fresh aggregate, checks the selected saved identity and
record revision, and mutates one record. Independent concurrent writes survive;
stale same-record and same-name additions fail, including identical payloads.
Revisions are checked positive TOML integers and never wrap.

Replacement is the native commit point. Failures before it preserve old bytes and
clean owned staging. Later synchronization/observation warnings return the exact
committed candidate, whose saved identity/revision the session retains. A later
write checks that same record before advancing; uncertain publication cannot
allocate a duplicate addition or authorize another session's stale revision.

The typed codec admits every current step, raw prefix form, role-qualified prompt
value, relative completion precondition, counter operation and modal entry state.
It preserves the captured initial counter format independently of the working
format. Encoded input is bounded before parsing, collection visitors enforce
record/step/argument/edit limits, and definition/aggregate validation enforces the
approved byte, count, format and decoded-memory bounds. Saved baselines share
immutable bodies and participate in the existing 48 MiB retention admission.

## Contracts Added or Changed

- Four unbound native commands and their exact catalog descriptions.
- Explicit macro-library attachment for isolated embeddings, plus read-only
  ordinary executable startup admission under the existing XDG root snapshot.
- Schema-1 aggregate and record revisions, opaque saved identities, typed input
  fields, saved baselines and conflict-aware name/binding admission.
- User-facing state reference, private backup guidance and the existing
  Unreleased macro outcome. No dependency or configuration leaf is added.

## Files of Interest

- `src/macros/library/codec.rs`: strict typed TOML projection and admission.
- `src/macros/library/store.rs`: private files, fresh-record checks and native
  atomic publication; `store_tests.rs` covers faults and revision exhaustion.
- `src/session/macros/library.rs`: native selection/consent, inert reload/export,
  session preservation and independent name/binding quarantine.
- `tests/macros/library.rs`: restart in both profiles, explicit-only writes,
  bindings, concurrent records, invalid state, deletion and sibling isolation.
- `docs/reference/macro-library.md`: exact schema, limits and ownership reference.

## Validation

The required development selection passed. Final clean-HEAD evidence is recorded
separately by the controller validator after implementation and PR references.
These checks do not establish the integrated delivery-set gate or PTY acceptance.

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check` — pass.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev -- -D warnings` — pass.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test macros -- library names --test-threads=1` — pass, 24 selected library/name cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands -- prefix_arguments palette ranking context_isolation transient_isolation prompt_reset workspace_reset --test-threads=1` — pass, 52 selected command cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check` — pass, 654 commands, 293 configuration entries, 93 pages and 642 links/anchors.
- `git diff --check` — pass.

Additional development checks:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib -- macros::library session::macros::tests queries_clock regions_admission --test-threads=1` — pass, 20 native admission, budget and storage cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test macros -- --test-threads=1` — pass, all 80 macro integration cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib -- macros::library --test-threads=1` — pass, 4 codec/publication cases after final schema admission changes.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test macros -- library::rejected_state --test-threads=1` — pass, invalid-store startup/reload/save preservation across 7 filesystem/input variants.

## Risks and Follow-ups

Definition editing and the macro menu remain Slice 7. Full integrated, PTY,
large-buffer and complete user-journey acceptance remains Slice 8. A committed
write can report uncertain durability after directory synchronization or final
observation fails; its exact candidate remains retained and revision checked.
