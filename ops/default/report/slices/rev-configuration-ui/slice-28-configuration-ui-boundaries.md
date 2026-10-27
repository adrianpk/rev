# Slice 28: Harden cross-feature failure boundaries

Status: reviewing
Delivery set: rev-configuration-ui
Plan: [Configuration and Interaction Delivery](../../../plan/rev-configuration-ui.md)
Tracker: [Configuration and Interaction Tracker](../../../tracker/rev-configuration-ui.md)
Branch: `test/rev-config-ui-28-configuration-ui-boundaries`
PR: pending

## Purpose

Verify whole-candidate rejection and filesystem/history transitions across the
existing configuration, completion, rectangle and rename owners.

## Delivered Behavior

- Existing include-cycle, depth, occurrence, per-source and total-byte fixtures
  now check startup rollback as well as loader diagnostics. Nondefault admitted
  prefixes detect partial publication. Root/include/parent permission failures,
  broken sources and malformed late layers retain complete built-ins.
- Layered normalized prefix conflicts run in editor, minibuffer and tree maps
  with both declaration orders. Diagnostics name the responsible source/key;
  unrelated palette, editing and association settings also roll back.
- Overlapping layered associations retain exact/suffix specificity and recorded
  origins after all source files disappear, without rediscovery or reload.
- Cached path selection rechecks invalid UTF-8/NUL replacement contents and
  current file/directory kind. Failures retain dirty shared text, history,
  exact view and editable query; retries accept the chosen path rather than a
  newly earlier directory entry. The candidate row limit is one.
- Six copy/kill/yank combinations transfer widths 4→8, 8→2 and 32→1 through
  three editing panes. Copied spaces and killed literal tabs remain exact.
  Two-row publication remains one complete undo unit with a one-group limit
  and one-byte soft payload target. Rectangle storage leaves the linear ring
  untouched; all shared target points remain valid.
- Explicit target saves preserve BOM/CRLF and final-newline absence. Undo/redo
  before and after save reports the correct dirty state. Source disk bytes and
  source pane identity remain unchanged by destination edits and saves.
- File and directory rename journeys exercise occupied-target refusal, real
  parent permission denial and success with shared views and a retained redo
  future after older history eviction. Only successful file rename changes
  language policy; directory rename retains descendant basename policy. Both
  use the frozen snapshot after the configuration file changes.
- A later save writes only the relocated path, retains BOM/CRLF/Unicode bytes,
  and preserves truthful saved identity across redo, undo and exhausted undo.

## Implementation Notes

- T28.1: `a56ad61f17e6` extends existing configuration fixtures and adds two
  native stale-path tests using the existing configured session fixture.
- T28.2: `c918b3a0945a` adds two parameterized native boundary tests for formatted
  rectangle transfer and configured file/directory rename/history workflows.
- T28.3 records the falsifiable requirement map below. All focused behavior
  checks pass on the existing runtime; no runtime repair is required.
- All changes use native integration targets, existing owned temporary fixtures
  and real filesystem operations. Permission modes are restored before
  assertions and fixture teardown. There are no new dependencies, subprocesses,
  skipped cases, global environment changes or changes to gate definitions.
- Work remains finite: existing include maxima, two invalid-content cases, two
  kind transitions, six transfer combinations, two rename forms and tree loops
  bounded by the observed entry count. The four added test functions cover
  distinct cross-feature interactions rather than repeating the PTY matrix.

## Contracts Added or Changed

| Failure boundary | Requirement it can falsify | Native evidence |
| --- | --- | --- |
| Ordinary/symlink/hard-link cycle leaks a partial candidate | CFG07-CFG08, CFG14 | `configuration::include_bounds::alias_cycles` |
| Exact graph maxima regress or one-past input escapes startup rejection | CFG08, CFG14 | `include_bounds::{depth_limit,occurrence_limit,source_byte_limit,total_byte_limit}` in configuration |
| Missing, nonregular, denied or invalid late sources activate accepted prefixes | CFG07-CFG08, CFG14 | `include_bounds::{broken_sources,denied_sources,rejected_graphs}` and `candidate::whole_candidate_fallback` in configuration |
| Cross-layer alias prefix collision partially replaces any context or unrelated preference | CFG07, CFG12-CFG14 | `configuration::keymaps::merged_conflict` |
| Association overlap loses specificity or consults removed configuration | CFG09, CFG21 | `configuration::associations::layered_patterns` |
| Selected stale contents consume shared edits or retry another sorted row | CFG16, UI03-UI04 | `path_completion::stale::changed_contents` and `failures::vanished_choice` |
| Cached directory marker overrides current target kind | UI03-UI04 | `path_completion::stale::changed_kind` |
| Cross-width capture/yank changes stored rows, splits a history group or loses save truth | CFG10-CFG11, CFG16-CFG17, UI01 | `rectangles::boundaries::formatted_transfer` |
| Failed rename changes policy/history or successful relocation loses redo/save ownership | CFG09-CFG11, CFG16-CFG17, UI01 | `recovery::configured::rename_history` |

## Files of Interest

- `tests/configuration/include_bounds.rs`: loader/startup rejection correlation.
- `tests/configuration/keymaps.rs`: layered context collision rollback.
- `tests/configuration/associations.rs`: retained overlap resolution.
- `tests/path_completion/stale.rs`: changed contents and current-kind acceptance.
- `tests/rectangles/boundaries.rs`: formatted shared cross-width transfer/save.
- `tests/recovery/configured.rs`: real rename failure/success with retained state.

## Validation

Successful focused development checks:

- `cargo test --test configuration include_bounds -- --nocapture`: nine tests passed.
- `cargo test --test configuration keymaps::merged_conflict -- --nocapture`: all three contexts and both prefix declaration orders passed.
- `cargo test --test configuration associations::layered_patterns`: retained precedence passed.
- `cargo test --test path_completion stale -- --nocapture`: two tests passed.
- `cargo test --test rectangles boundaries -- --nocapture`: six transfer combinations passed.
- `cargo test --test recovery configured -- --nocapture`: file and directory journeys passed.
- Changed-source native quality checks passed with at most five parameters,
  nesting at most two and cyclomatic complexity at most ten.

Final readiness requires controller evidence for the clean PR head after the
PR-reference follow-up commit, with this exact command order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test configuration`
- `cargo test --test path_completion`
- `cargo test --test rectangles`
- `cargo test --test recovery`
- `python3 scripts/check-source.py`
- `git diff --check`

## Risks and Follow-ups

Permission cases require an ordinary Unix user and fail visibly if access
bypasses permission bits; no permission cases are skipped. These native tests
prove owner/state and filesystem boundaries. The already integrated PTY evidence
covers emitted terminal behavior; bounded-work measurements and the complete
acceptance map retain their approved subsequent owners.
