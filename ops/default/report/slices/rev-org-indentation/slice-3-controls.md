# Slice 3: Buffer-local runtime commands

Status: delivered
Delivery set: rev-org-indentation
Plan: [Org Indentation Delivery](../../../plan/rev-org-indentation.md)
Tracker: [Org Indentation Tracker](../../../tracker/rev-org-indentation.md)
Branch: `feat/rev-org-indentation-03-controls`
PR: `#278`
Integrated: `660b97410fd96f6cfb3b664b48dceee7b37ddfc3`

## Purpose

Deliver the OIN02 runtime controls and OIN03-OIN05 buffer lifecycle behavior
through T3.1-T3.3. Joint PTY/resource acceptance and guide reconciliation remain
Slice 4.

## Delivered Behavior

- `org-indent-mode`, `org-indent-enable` and `org-indent-disable` use the shared
  native command catalog, M-x, modal colon and configurable editing keymaps.
  They report the resulting state, admit read-only Org results and reject
  inactive Org contexts. Enable/disable are idempotent; no shortcut is added.
- One optional runtime choice belongs to the canonical buffer. All its views
  share it, while other buffers, pretty-heading choices and wrapping remain
  independent. Commands never write TOML, source, saved state or edit history.
- Source edits, heading moves, undo/redo, folds, resize, rebind and external
  reload retain the choice. Text mode suspends it; Org reactivation restores it.
  Closing discards it, and reopening starts from the frozen startup preference.
- Effective policy changes invalidate existing bounded layout through its
  current identity and retire saved screen-column goals in every affected view.
  Source points and selection anchors retain their canonical identities.
- Command/configuration references, the exhaustive sample comment and Unreleased
  describe the delivered controls and their lifetime.

## Implementation Notes

T3.1-T3.3 are implemented in `e6dc1c0f1b07e046309911867b9782d87b24008e`.
The existing buffer policy keeps
its startup value plus an optional runtime override; Org activation gates the
query. Settings reapplication changes the startup value without erasing the
explicit override. The existing structural observation owner handles enablement;
no timer, dependency, document scan or second cache is introduced.

## Contracts Added or Changed

- Explicit choices equal to startup still retain runtime provenance.
- Read-only is a source-edit boundary and does not prohibit these presentation
  controls. Org context admission remains with the native session dispatcher.
- Cached fragments cannot authorize hits after an effective policy change,
  even though the source stamp remains unchanged.

## Files of Interest

- `src/buffer.rs`, `src/buffer/org.rs`: buffer-owned runtime choice.
- `src/command.rs`, `src/command/org.rs`, `src/session/outline/indentation.rs`:
  catalog identities and shared dispatch.
- `src/view.rs`: screen-column goal invalidation.
- `tests/org_indentation/controls.rs`, `tests/commands/org_catalog.rs`:
  native command entry, policy and lifecycle acceptance.

## Validation

The following implementation checks passed locally:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test org_indentation --test commands --test configuration --test external_changes -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test modal contexts:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

The required native batch passed 356 tests: commands 164, configuration 155,
external changes 21 and indentation 16. The modal contexts filter passed 20 tests
(97 unrelated modal tests filtered out). Documentation passed the 596-command
catalog and 85 reachable pages with 554 local links/anchors.

Observed successful command times were format 2.77s, Clippy 1.03s, the native
batch 320.03s, modal contexts 13.98s, documentation 1.13s and diff check 0.02s.
The command suite accounted for 209.02s and configuration for 91.90s. These
required suites remain minutes-scale; no deadline or command was changed.

All six required commands passed through Looper on the exact clean final HEAD
`660b97410fd96f6cfb3b664b48dceee7b37ddfc3`. The canonical PR contract was
verified, and PR #278 is merged into dev at that same commit. The combined
delivery gate remains controller-owned.

The first required native batch passed 163 command tests and failed the old
catalog cardinality assertion (231 names versus 234 after these three additions).
The exact assertion now expects 234; the focused visibility case passed after
that correction. The failed batch took 258.82s, including 52.34s compilation and
206.44s command-suite runtime. No acceptance assertion was removed or skipped.

Additional focused checks passed:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test org_indentation controls:: -- --test-threads=1` (seven session tests).
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib session::outline::indentation::tests:: -- --test-threads=1` (one native owner test).
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands org_visibility::visibility_keys -- --test-threads=1` (the corrected catalog case).
 The native readonly
result-owner filter passed one test under both profiles, including explicit
choices equal to startup, settings reapplication and stale fragment-hit rejection.
Its first build took 40.50s and the case took 0.68s.

## Risks and Follow-ups

The controller-owned joint gate has not run. Slice 4 retains the representative
PTY/resource driver, Org learning path, strict Spore build and specification/model
reconciliation. This slice does not claim full delivery-set acceptance.
