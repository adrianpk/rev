# Slice 43: Harden ordinary failure boundaries

Status: delivered
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `fix/rev-43-failure-recovery`
PR: `#43`
Integrated: `99c13fa2b99cecb5553bf2c5074edeeb6ba6c8ae`

## Purpose

Preserve accepted editing state and truthful publication/cleanup outcomes across
ordinary filesystem, syntax, focused-input and terminal failures.

## Delivered Behavior

- Saving rechecks the destination after staging temporary output and before
  replacement. Content changes, removal and substituted symlinks detected there
  stop publication, preserve external entries and clean up only owned temporary
  output. Failed attempts retain text, history, association and unsaved state.
- A pending tree creation captures the selected directory's device/inode and
  revalidates them before creating a file or child directory. A replacement
  directory at the same path rejects the old request without receiving an entry.
- Terminal restoration attempts every acquired obligation in order, including
  raw-mode release after output failures. Combined errors retain every failed
  step and the first error kind. Failed output obligations remain owned for
  retry during Drop.
- Real-file ingress errors after partial BOM, chunk and multibyte reads expose
  no partial candidate. A later clean read retains exact format bytes. Failed
  ingress during a focused replacement retains the source and pending argument;
  a rejected NUL replacement and canceled write preserve the shared session.
- Parser cancellation and query-result exhaustion remain isolated from source
  and history. Recovered parser/query cases save exact bytes. Accepted text
  survives a canceled parse followed by real save publication with directory
  sync and observation failures; the warning prevents false clean/quit state.
- Late read-only or peer-position changes reject a whole disjoint edit without
  changing any row, shared position, mark, syntax capture, history or redo future.
  Save recheck plus cleanup failure reports both errors and retains external bytes.

## Implementation Notes

- T43.1: `2e45d275864e289f10c83776166569fe197fce3b` reproduces and fixes destination changes
  during temporary staging and replacement creation directories. It extends
  real-file ingress errors, parser/query save recovery and individual terminal
  cursor/attribute/screen cleanup failures. All seven ordered checks pass:
  444 native tests, 12 recovery tests and 89 PTY cases.
- T43.2: `21cde797e6cc8289178dac0d7d118ce0dd9b437c` reproduces and fixes loss of later terminal
  restoration errors. It adds combined cleanup, parser/publication, ingress/focus
  and rejected compound-edit cases plus a combined terminal PTY mode. All seven
  ordered checks pass: 448 native tests, 16 recovery tests and 90 PTY cases.
- Both filesystem defects were reproduced before their fixes: staging could
  overwrite newly changed external data, and a captured creation request could
  write into a replacement directory. The regressions now assert retained disk
  data, exact buffer snapshots and owned-temporary cleanup.
- Save faults use the existing Output boundary with real files, writes, flush,
  file sync and replacement. Only otherwise nondeterministic read/sync/observe/
  cleanup failures or change timing are controlled. A published replacement is
  never removed as cleanup. Parser cancellation uses the real progress callback;
  query coverage uses the actual result bound and a later smaller valid request.
- Terminal probe modes inject one failure per actual output pattern. The
  combined mode pauses after explicit restoration so the PTY can verify normal
  termios before Drop retries failed output obligations. It then checks every
  reported operation, successful retry ordering and final screen/paste/cursor state.
- Recovery snapshots cover serialized bytes, qualified revision/state, file
  association, name, mark/rectangle state, history/redo counts, dirty/read-only
  state and warnings. Rejected compound publication also compares peer positions
  and current syntax captures, then exercises the retained redo operation.
- The focused recovery suite completes in under one second locally. Four added
  PTY failure variants reuse the existing finite eight-second fixture deadlines,
  bounded output, real controlling terminals and owned cleanup after checks.

## Contracts Added or Changed

- A save destination must still match its captured observation after staging;
  a detected change does not grant overwrite authority.
- A creation prompt identifies a directory owner, not only a pathname and type.
- Terminal restoration returns all failed operations while still attempting each
  acquired obligation and retaining unsuccessful obligations for retry.
- Recovery remains local to its owner: derived syntax failure cannot roll back
  accepted edits, and later filesystem failure cannot erase actual publication.
- The existing Unreleased entries and reference material describe the delivered
  save/tree guards, publication outcomes and restoration behavior.

## Files of Interest

- `src/file/save.rs`: destination recheck and owned failure cleanup.
- `src/tree/create.rs`: captured-directory identity and validation.
- `src/terminal.rs`: complete restoration error aggregation.
- `tests/recovery/publication.rs`, `tests/recovery/ingress.rs`:
  real boundary faults and filesystem guard regressions.
- `tests/recovery/combined.rs`: combined publication, focus and compound state cases.
- `tests/syntax/failures.rs`: parser/query recovery through exact saving.
- `examples/terminal-probe.rs`, `scripts/check-pty.py`: individual and combined
  post-acquisition errors with observable cleanup ordering.
- `docs/reference/recovery.md`, `docs/reference/file-tree.md`,
  `docs/reference/prompts.md`, `CHANGELOG.md`: delivered failure behavior.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; 448 tests.
- `python3 scripts/check-source.py`: passed; safe Rust, five parameters,
  nesting at most three and cyclomatic paths at most ten.
- `cargo test --test recovery`: passed; 16 tests, zero skips.
- `python3 scripts/check-pty.py --suite terminal`: passed; 90 cases, zero skips.
- `git diff --check`: passed.

## Risks and Follow-ups

Destination rechecks are conservative metadata observations; a final interval
between checking and replacement remains. Injected failures establish retained
state and truthful outcomes, not physical-crash durability.
