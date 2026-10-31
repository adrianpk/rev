# Slice 6: Joint acceptance and guidance

Status: reviewing
Delivery set: rev-bookmarks
Plan: [Native Bookmarks Delivery](../../../plan/rev-bookmarks.md)
Tracker: [Native Bookmarks Tracker](../../../tracker/rev-bookmarks.md)
Branch: `fix/rev-bookmarks-gate-2`
PR: [#192](https://forge.adrianpk.com/adrianpk/rev/pulls/192)
Original slice branch: `feat/rev-bookmarks-06-acceptance`
Original slice PR: [#190](https://forge.adrianpk.com/adrianpk/rev/pulls/190)

## Purpose

Exercise BMK01-BMK07 together through native sessions and actual Linux PTYs,
measure bounded work, and complete the bookmark learning/reference path.

## Delivered Behavior

The acceptance entrypoint uses one owned temporary fixture with independent
native and terminal HOME/XDG roots. It drives named set/refusal, canonical
edit/undo/save, alias identity, shared views, list deletion, store restart,
checked context, fold reveal and unavailable/malformed-store protection.
Two real editor processes exercise persisted jumps and deletion across a
restart, independent markers/numbering, resize, cancellation, idle silence and
terminal restoration.

A task guide is linked from the existing search/commands learning chapter and
the how-to index. The command reference documents exact status, store layout,
save/reload and marker behavior, with focused prompt/feedback examples. The
line-number reference describes the independent marker cell and number-priority
geometry. The single Unreleased bookmark entry links the guide.

## Implementation Notes

The Bash/Rust fixture follows the existing native/PTY approach. Each terminal
child retains the eight-second deadline and 1 MiB output cap. Cleanup closes
input, reaps the owned child and reader, restores terminal attributes and removes
the temporary roots. No installed binary or user store participates.

The list deliberately hides its cursor. Completed-screen detection accepts its
full final row and reset trailer as well as an ordinary cursor-show trailer;
an incomplete redraw cannot reuse a preceding frame. Native readiness and
relocation wait only for bounded session-owned work. Resource measurements use
1024 bookmarks, a large source with a long wrapped line, reversal, resize and
idle observations. They report actual latency and Linux peak RSS.

## Contracts Added or Changed

The configured acceptance command gains its native/PTY producer. The existing
runtime resource and persistence contracts remain the acceptance assertions.
Directly related gutter clauses now include BMK03 while preserving every GUT
requirement identity and the original numbering candidate/gate metadata. The
bookmark tracker owns its separate integrated gate evidence; unrelated pending
capabilities retain their own scope.
The controller retains the unchanged combined gate and integration authority.

## Files of Interest

- `scripts/check-bookmarks.sh`: temporary roots, release build and PTY wrapper.
- `examples/bookmark-acceptance.rs`: joint acceptance entrypoint.
- `examples/bookmark_acceptance/`: native journey, resource probe and bounded PTY driver.
- `docs/how-to/use-bookmarks.md`: task guide and focused feedback examples.
- `docs/reference/commands.md` and `docs/reference/gutter.md`: exact behavior and presentation.
- `ops/default/spec/editor-gutter*.md`: reconciled marker clauses and preserved numbering evidence.

## Validation

Development `bash -n scripts/check-bookmarks.sh` and
`bash scripts/check-bookmarks.sh` passed. The first release build took 1m33s;
a subsequent fixture run built in 4.24s. Its native journey took 113ms and both
PTY processes completed in 484ms, emitting 61135 and 21276 bytes. Both restored
TTY attributes, cursor, paste mode and alternate screen; the final session had
350ms of output silence. The saved empty collection was reopened through the
native loader and confirmed empty, beyond checking its version-only encoding.

Initial setup corrections returned from explicit tree focus before invoking a
bookmark command and recognized the existing omitted empty record array. These
preserve checked destination, source, record-count and restoration assertions;
no runtime behavior, deadline or output allowance changed.

Development resource observations on a 2,477,825-byte source with 1024 records:

| Observation | Result |
| --- | --- |
| Collection admission | 83.576ms; the next record was refused without eviction |
| Cold / maximum warm frame | 0.633ms / 0.311ms |
| Maximum movement / resize | 0.544ms / 0.540ms |
| Far reversal / 64 insert-delete reversal | 0.591ms / 68.447ms |
| Checked relocation | 14.036ms over 17 event-loop passes |
| Longest measured projection/input/relocation pass | 0.898ms |
| Process peak RSS | 24468 kB |
| Idle | No deadline; future expiry returned Wait |

Development `cargo run --locked --quiet --example documentation-check` passed
319 commands, 189 configuration entries, 59 pages and 341 links/anchors. Its
initial strict command-table check treated code-styled status labels as command
names; those labels now use ordinary text. The checker and command assertions
remain unchanged. All complete sample/catalog defaults remain consistent.

Final clean-HEAD checks passed after all task and canonical PR-reference commits,
in this exact required order:

1. `cargo fmt --all -- --check`
2. `cargo test --locked --test bookmarks --test gutter --test wrapping --test commands --test configuration`
3. `bash -n scripts/check-bookmarks.sh`
4. `bash scripts/check-bookmarks.sh`
5. `cargo run --locked --quiet --example documentation-check`
6. `git diff --check`

A separate focused batch passed `cargo test --locked --lib bookmark` and
`cargo clippy --locked --all-targets -- -D warnings`
on the same candidate before the required receipt. The combined gate remains
pending under controller ownership.

## Risks and Follow-ups

The terminal fixture requires Linux procfs, util-linux `script`, `stty`, GNU
`timeout`, Bash and the repository's Cargo toolchain. Measured timings describe
the local fixture and do not establish performance on every terminal or host.

Canonical PR #190 merged through Rebase + Fast-forward into `dev` at
`9217d4dec10ed76860e6eab13939064746973ed1`. Required receipt
`6IM6JZYJ55PDLPZPJYQH4NHGW6` passed the exact six-command list and 416
integration tests. The native journey completed in 124ms; checked relocation
took 12.761ms over 17 passes, with a maximum measured pass of 0.686ms and
24480 kB peak RSS. The two PTY processes completed in 484ms, emitted
61134/21275 bytes, restored TTY attributes and retained 350ms of idle silence.
Documentation passed 319 commands, 189 settings, 59 pages and 341 links.
Focused receipt `HXGI5YRKCZCGNXGX572RYYK45M` passed 12 bookmark owner tests and
all-target clippy on the same clean candidate. The controller verified and
recorded the canonical merge in Acta. Slice 6 of 6 completed.

## Gate repair 1: Mouse fixture geometry

The controller's unchanged full gate failed at
`eba31f5dee666bb3cc02f009449fba0d6e7238dc` in receipt
`674A2DA3QLY2I2GXI2ZXLOCXBO`. Formatting and all-target clippy passed; the
library suite passed 89 tests and failed `session::events::tests::timeout_release`.
The remaining combined-gate commands did not run.

The shared event-test helper still pressed column 6. Default numbering occupies
six cells and BMK03 reserves the next cell for markers, so column 6 now belongs
to the gutter. Native hit admission correctly refused that press; no selection
or autoscroll deadline existed when the fixture expected to exercise timed input.
`cargo test --locked --lib session::events::tests::timeout_release -- --exact`
reproduced the retained failure on the failed base and passed after correction.

The helper now derives its text column from the selected editing pane's current
gutter projection. Both callers assert a live selection; the timeout fixture
also asserts the original 100 ms deadline before testing polling. The existing
ready/boundary/timeout answers, release-before-tick ordering, top-row results
of 0 or 3 and indefinite idle wait remain intact. Markers stay enabled. The
repair changes only `src/session/events/tests.rs` and this report, with no
runtime, gate, acceptance, deadline or resource-limit changes.

Final clean-HEAD validation passed after the repair and canonical PR-reference
commits in the pinned order:

1. `cargo fmt --all -- --check`
2. `cargo test --locked --lib session::events::tests`
3. `cargo test --locked --test pointer --test gutter --test bookmarks`
4. `git diff --check`

Canonical repair PR [#191](https://forge.adrianpk.com/adrianpk/rev/pulls/191)
merged through Rebase + Fast-forward into `dev` at
`f6f09ba63cea3e5f0af10984aa264ec9dc29db48`. Required receipt
`INYL32A5XSUTQCTFTOHL3IVJGF` passed the exact four-command list: 12 event
tests and 158 integration tests (57 bookmarks, 36 gutter and 65 pointer).
The controller verified and recorded the repair merge in Acta. All six slices
and the authorized repair are integrated; the unchanged full gate remains
pending under controller ownership.

## Gate repair 2: Explicit store lock release

The controller's unchanged full gate failed at
`359b392858646941fe5c65b384b9609c583341ad` in receipt
`4QZXBGUBDCSJQ275WAQXLEQDWS`. Formatting and all-target clippy passed; the
library suite passed 89 tests and failed `bookmark::store::tests::publication_faults`
when an immediate publication retry reported a busy lock. The remaining
combined-gate commands did not run.

The acquired lock was a raw `File`, so scope exit only closed that descriptor.
A duplicate shares its lock and can retain it after the writer's handle closes;
concurrent process creation can retain such a duplicate until exec. The gate
log does not identify a particular inheriting child. A deterministic regression
keeps a real duplicate open across owner release and checks fresh nonblocking
acquisition. `cargo test --locked --lib bookmark::store::tests::lock_lifetime -- --exact`
failed with the same busy-lock diagnostic before the correction.

An owning lock guard now explicitly calls the standard library's unlock on
drop, covering successful publication and every early error return. Acquisition
remains nonblocking and retains the existing lock-inode checks. The regression
also verifies exclusion while either owner is live and that closing the old
duplicate cannot release a subsequent owner's lock. Existing fault-injection,
external-change, unresolved-publication, permission and real-process conflict
assertions remain intact. Development
`cargo test --locked --lib bookmark::store::tests` passed all seven store tests.

This repair changes only `src/bookmark/store.rs`, its existing test module and
this report. BMK05 compare-and-publish serialization, disk/memory preservation,
store bounds, process deadlines and the unchanged full gate retain their
approved contracts.

Final clean-HEAD validation follows the repair and canonical PR-reference
commits in the pinned order:

1. `cargo fmt --all -- --check`
2. `cargo test --locked --lib`
3. `cargo test --locked --test bookmarks --test recovery --test files`
4. `git diff --check`

The controller owns repair integration and rerunning the unchanged full gate.
