# Slice 56: execute selected Org blocks through bounded tools

Status: delivered
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-56-block-execution`
PR: `#157`

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

Task commits: T56.1 `2e6b5af51ae11d0a6d9ff407cd7779fc038f53e2`, T56.2 `2632eb3a834f3a354bb50280e50f0ae0b1ef507d`, T56.3 `926161b067ecb13eacab6611612f591e7ea6ad04`.

Canonical PR: [#157](https://forge.adrianpk.com/adrianpk/rev/pulls/157), targeting `dev`.

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

Final required receipt `YKY4RUANELNCNC4EIHB32D54WK` passed on
`95c6ec8348096dce7189c2351768605d2c591d9f`:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_io --test commands --test recovery`
- `python3 scripts/check-source.py`
- `git diff --check`

All 350 tests passed: 158 commands, 167 I/O and 25 recovery; zero failures,
ignored or filtered tests. Source quality retains safe Rust, at most five
parameters, three nesting levels and cyclomatic complexity ten.

Focused receipt `2EIU6HWMMWSD4OXTGFN357O256` passed on the same final head:

- `cargo test --test configuration --test org_editing --test org_source -- --test-threads=4`
- `cargo test --lib org::io::process -- --test-threads=1`
- `python3 scripts/check-config-reference.py`
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-execution`
- `git diff --check`

All 433 focused tests passed: 126 configuration, 226 editing, 78 source and three
process ownership cases. The library filter intentionally excluded 46 unrelated
cases. The real terminal journey passed in RGB and NO_COLOR modes, zero skips.
It checks physical execution chords, confirmation, decline, retained output,
unchanged source/old RESULTS and absence of execution during visit/export.
Native cases cover read-only and unsaved BOM/CRLF source, runner selection,
stdin/file delivery, real multi-step programs, missing/nonzero tools, combined
output limits, bounded noweb and cancel/timeout/drop cleanup. Direct children are
reaped, descendants stop, and private workspace modes and removal are checked.
The pipeline fixture checks that overflow prevents subsequent stages without
assuming rollback of earlier host effects.

References agree on 171 schema/catalog entries, 54 active RGB roles and 259
default bindings. The unchanged source checker accepts the candidate.
Canonical PR #157 merged into `dev` at
`95c6ec8348096dce7189c2351768605d2c591d9f`. Acta records the final receipts
and completed delivery. Slice 57 is the next active recorded slice.

## Risks and Follow-ups

Configured programs have ordinary host authority. Ownership and HOME/XDG isolation
do not sandbox confirmed code. Canonical result insertion belongs to the next
recorded slice. No implicit block execution occurs during visits or export.
