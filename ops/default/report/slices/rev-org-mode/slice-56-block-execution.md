# Slice 56: execute selected Org blocks through bounded tools

Status: reviewing
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-56-block-execution`
PR: pending

## Purpose

Deliver selected native Org block jobs under the
[approved plan](../../../plan/rev-org-mode.md#slice-56-execute-selected-org-blocks-through-bounded-tools)
and [tracker](../../../tracker/rev-org-mode.md#slice-56-execute-selected-org-blocks-through-bounded-tools).

## Delivered Behavior

Explicit selected block commands resolve a declared language profile and ask for
confirmation by default. Source no/never restrictions always apply. Typed
stdin/file adapters and optional compile/run steps retain actual stdout, stderr,
status and source without changing existing RESULTS. Missing programs and failed
jobs remain usable diagnostics. Canonical source and history remain unchanged.

## Implementation Notes

The existing process-group owner receives optional staged stdin and private
HOME/XDG paths. One owned worker applies combined output and operation deadlines,
reaps children and removes its private workspace. Source/header observations
never execute. Literal noweb reuses the bounded native expander. Profiles are
closed typed configuration data with assigning-file origins and explicit array
replacement. All controls retain finite limits and source revision admission.

See [source execution](../../../../../docs/reference/org-source-blocks.md#execution)
and the Org manual's [evaluation](https://orgmode.org/manual/Evaluating-Code-Blocks.html).

## Contracts Added or Changed

- ORG28: explicit confirmed source jobs and retained revision-qualified input.
- ORG18: bounded owned programs, streams, cancellation and cleanup.

## Files of Interest

- `src/config/source.rs`: typed program profiles and finite admission.
- `src/org/execution.rs` and `src/org/execution/`: expansion and owned job work.
- `src/org/io/process.rs`: shared process groups and staged stdin.
- `src/session/execution.rs`: selection, consent and retained result inspection.
- `tests/org_io/execution.rs`: real shell and source-preservation workflows.

## Validation

Development checks passed:

- `cargo check --lib`
- `cargo test --test configuration source_profile`
- `cargo test --test org_io execution`
- `python3 scripts/check-config-reference.py`
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-execution`

Two configuration tests and nine native execution tests passed. The real terminal
journey passed in RGB and NO_COLOR modes, zero skips. It checks both physical
execution chords, default confirmation, decline, retained output inspection,
unchanged source/old RESULTS and absence of execution during visit/export.
Native cases cover read-only and unsaved BOM/CRLF source, explicit runner
selection, stdin/file delivery, real multi-step programs, missing/nonzero tools,
combined output limits, bounded noweb and actual cancel/timeout/drop cleanup.
Direct children are reaped, descendants stop and private workspace/input modes
and removal are checked. The corrected pipeline fixture checks that overflow
prevents subsequent stages, without assuming rollback of earlier host effects.

References agree on 171 schema/catalog entries, 54 active RGB roles and 259
default bindings. The unchanged Rust source checker accepts the changed files.
Required exact-head controller validation remains pending.

## Risks and Follow-ups

Configured programs have ordinary host authority. Ownership and HOME/XDG isolation
do not sandbox confirmed code. Canonical result insertion belongs to the next
recorded slice. No implicit block execution occurs during visits or export.
