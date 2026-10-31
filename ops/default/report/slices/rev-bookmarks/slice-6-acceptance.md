# Slice 6: Joint acceptance and guidance

Status: drafting
Delivery set: rev-bookmarks
Plan: [Native Bookmarks Delivery](../../../plan/rev-bookmarks.md)
Tracker: [Native Bookmarks Tracker](../../../tracker/rev-bookmarks.md)
Branch: `feat/rev-bookmarks-06-acceptance`
PR: pending

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
The controller retains the unchanged combined gate and integration authority.

## Files of Interest

- `scripts/check-bookmarks.sh`: temporary roots, release build and PTY wrapper.
- `examples/bookmark-acceptance.rs`: joint acceptance entrypoint.
- `examples/bookmark_acceptance/`: native journey, resource probe and bounded PTY driver.

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

Final clean-HEAD checks follow all task and canonical PR-reference commits,
in this exact required order:

1. `cargo fmt --all -- --check`
2. `cargo test --locked --test bookmarks --test gutter --test wrapping --test commands --test configuration`
3. `bash -n scripts/check-bookmarks.sh`
4. `bash scripts/check-bookmarks.sh`
5. `cargo run --locked --quiet --example documentation-check`
6. `git diff --check`

A separate focused batch checks `cargo test --locked --lib bookmark` and
`cargo clippy --locked --all-targets -- -D warnings`
on the same candidate before the required receipt. The controller runs the
combined gate after this slice integrates; this report does not claim that gate.

## Risks and Follow-ups

The terminal fixture requires Linux procfs, util-linux `script`, `stty`, GNU
`timeout`, Bash and the repository's Cargo toolchain. Measured timings describe
the local fixture and do not establish performance on every terminal or host.
