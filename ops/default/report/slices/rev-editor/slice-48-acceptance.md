# Slice 48: Verify the complete integrated editor contract

Status: reviewing
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `test/rev-48-acceptance`
PR: pending

## Purpose

Connect every accepted editor scenario to real regression evidence and prepare
complete, exact-candidate validation for controller-owned integration.

## Delivered Behavior

- The A01-A20 manifest links 89 named native-test references to four real PTY
  journeys and all eight required performance workloads. Native discovery
  rejects missing/renamed cases; gate evidence must show their actual success.
- The full gate runs the approved eight commands in their original order.
  Its six execution stages produce owned temporary logs and checksums tied to
  one clean HEAD/tree. The acceptance stage rejects missing, skipped, filtered,
  duplicate, reordered, failed, altered or stale evidence.
- Readiness verifies both frozen 48-row identity tables, the required command
  prefix, pinned direct dependency scope, report metadata, task state and
  integrated report ancestry. The current report can remain under review until
  the controller completes integration and the final gate.
- Command-history retention and its product reference now match the approved
  P12 limit of 128 distinct accepted names. Existing catalog, history, editing
  and terminal behavior remain covered by the real command paths.

## Implementation Notes

- T48.1: `b5fb082836f69ce6395874c1394b396012ea0205` introduces the acceptance map, same-run gate evidence and focused rejection tests.
- T48.2: `3399f763425bef65b40116a99823dc1604e17254` verifies delivery prerequisites, aligns P12 history retention and completes evidence guards.
- `--prerequisites` discovers compiled native cases without executing another
  test suite. It verifies the 47 delivered reports and permits pending PR
  metadata for the active report. It cannot establish a full-gate pass.
- The full acceptance stage requires the active report's canonical PR reference.
  It checks every earlier report's full integrated hash is an ancestor of HEAD.
  Both existing report integration formats retain their exact meaning.
- JSON links identify specific native target/test pairs. The PTY map names only
  functions actually enumerated by the acceptance suite. Semantics were checked
  against the test assertions/comments and the four composed journey bodies.
- Source metrics must contain real measurements and the accepted safe-Rust,
  five-parameter, depth-three and cyclomatic-ten completion. Performance
  evidence must contain both large fixtures, every workload, release timings,
  incremental/cold parser observations and real PTY startup/idle completion.
- Logs stay under an owned ignored temporary directory and are removed after
  the gate, including failure. Command output remains in the controller log.
  This slice adds no runtime dependency or normal editor command.

## Contracts Added or Changed

- [Acceptance manifest](../../../../../scripts/acceptance.json) preserves every
  A-identity exactly once. All native references, PTY journeys and performance
  workload names must resolve; duplicate or absent entries fail validation.
- The canonical plan/tracker table digest pins branch, title and report identity
  together. Matching edits to both tables cannot silently redefine this set.
- A failed command stops the gate before acceptance. Receipts from another
  candidate or changed logs cannot satisfy the completeness stage.
- Report/tracker delivery completion remains controlled by verified integration
  and the final gate. Readiness does not mark pending tasks delivered.

## Files of Interest

- `scripts/acceptance.json`: complete, named A01-A20 evidence map.
- `scripts/acceptance.py`: native/source/PTY/performance completeness and
  same-candidate receipt checks.
- `scripts/acceptance_delivery.py`: approved identities, scope and reports.
- `scripts/check-acceptance.py`, `scripts/check-all.sh`: readiness, complete
  execution and the frozen eight-command entrypoint.
- `scripts/test_acceptance.py`: seven focused negative-evidence regressions.
- `src/input/palette.rs`, `docs/reference/commands.md`,
  `docs/reference/defaults.md`: P12 history retention alignment.

The map below summarizes targets; the linked manifest gives every exact native
case name. PTY labels denote `acceptance_<label>` in `scripts/check-pty.py`.

| ID | Accepted behavior | Native targets | PTY journeys | Performance |
| --- | --- | --- | --- | --- |
| A01 | Editable empty startup and restored terminal | display, editing | document, failure | — |
| A02 | Mode-free printable insertion at point | editing, commands | workspace | — |
| A03 | Unicode-aware character, word, line and display motion | coordinates, display | document | — |
| A04 | Find, save, write-file and exact file bytes | files | document, blocks, failure | — |
| A05 | Buffer inventory, switching and guarded close independent of windows | files | workspace | — |
| A06 | Nested splits with shared text and independent points | windows | document, workspace | split_redraw |
| A07 | Window deletion, keep-selected and focus preserve buffers | windows | document | — |
| A08 | Visible region, exchange, focus and cancellation | editing | blocks | — |
| A09 | Directional kills, copy, yank and current-range yank-pop | editing | blocks | — |
| A10 | Display-column rectangles and one-transaction history | rectangles | blocks | — |
| A11 | Named commands and bindings use the same real handlers | commands | document, workspace | — |
| A12 | Per-buffer undo/redo for insertion, deletion and paste | editing, commands | document, blocks | — |
| A13 | Forward/backward smart-case search and confirmed literal replacement | search | document | search |
| A14 | Lazy keyboard tree, safe operations and retained visited paths | tree | workspace | — |
| A15 | Bundled current Rust highlighting and editable parser fallback | syntax | document, workspace | highlighting, syntax_work |
| A16 | Canonical rope ownership through file, edit, history and syntax operations | core, editing, files, syntax | document, blocks | load, insert, delete, syntax_work |
| A17 | Bounded native editor scope and complete static command catalog | commands, syntax | document, workspace | — |
| A18 | Safe Rust, bounded source structure, meaningful tests and lint checks | commands, examples/source-check | document | syntax_work |
| A19 | Guarded close/quit, fresh save consent and retained text after failure | files, recovery, tree | workspace, failure | — |
| A20 | Resize and ordinary boundary failures retain state and terminal lifetime | windows, recovery, commands | failure | scroll |

## Validation

All eight ordered slice checks passed through Looper on task HEAD `3399f763425bef65b40116a99823dc1604e17254` under
validation `UWFQMD5Y4OKIXYB6KQVQ5UMLJM`. Final clean PR-head evidence is recorded
after the report and PR-reference commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; 455 native tests, zero ignored.
- `python3 scripts/check-source.py`: passed; safe Rust and the accepted limits.
- `python3 scripts/check-pty.py --suite acceptance`: passed; four assembled
  real-terminal journeys, zero skipped, exact file bytes and restored terminals.
- `python3 -m unittest discover -s scripts -p test_acceptance.py`: passed;
  seven focused checker tests with named negative variants.
- `python3 scripts/check-acceptance.py --prerequisites`: passed; native-case
  discovery, complete map, fixed identities, scope and report prerequisites.
- `git diff --check`: passed.

The existing command-history regression also passed after the P12 correction.

## Risks and Follow-ups

The worker's focused checks do not constitute the combined integrated gate.
The controller must execute `scripts/check-all.sh` on the exact clean integrated
candidate and retain its result before final documentation closure. Performance
limits remain those of the existing representative fixtures and hardware
observations; this slice adds no latency or general file-size guarantee.
