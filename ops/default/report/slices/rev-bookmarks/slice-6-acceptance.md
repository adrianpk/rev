# Slice 6: Joint acceptance and guidance

Status: delivered
Delivery set: rev-bookmarks
Plan: [Native Bookmarks Delivery](../../../plan/rev-bookmarks.md)
Tracker: [Native Bookmarks Tracker](../../../tracker/rev-bookmarks.md)
Branch: `feat/rev-bookmarks-06-acceptance`
PR: [#190](https://forge.adrianpk.com/adrianpk/rev/pulls/190)

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
