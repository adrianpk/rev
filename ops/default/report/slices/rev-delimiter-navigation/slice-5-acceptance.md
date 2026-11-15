# Slice 5: Integrated navigation acceptance

Status: delivered
Delivery set: rev-delimiter-navigation
Plan: [Delimiter Navigation Delivery](../../../plan/rev-delimiter-navigation.md)
Tracker: [Delimiter Navigation Tracker](../../../tracker/rev-delimiter-navigation.md)
Branch: `test/rev-delimiter-navigation-05-acceptance`
PR: [#66](https://forge.adrianpk.com/rev/rev/pulls/66)

## Purpose

Verify the seven navigation commands together, preserving their shared source,
input and presentation owners through native and physical-input workflows.

## Delivered Behavior

The new acceptance script checks every selected filter is nonempty before
running the complete navigation and existing pair suites. Focused configuration,
count/history, macro motion/region, modal percent/object and Org/Markdown
continuity cases protect shared owners. One independent factory helper checks
717 commands, six actions and all 28 complete maps in both startup profiles.

Exactly two representative Linux PTYs exercise actual C-M bytes and named mate
invocation in non-modal and modal Insert editing. Saved source effects prove
group/nesting/expression/mate endpoints and counts independently of cursor labels.
They also check active-mark copying, prompt focus, native read-only shell output,
undo, truthful hints, save/reopen and unchanged configuration bytes.

The User Guide now provides one cohesive navigation exercise. The command
reference records count, context, normalization, bounded refusal and effective
profile behavior; one Unreleased entry describes the delivered capability.

## Implementation Notes

The existing safe-Rust terminal harness owns util-linux script, TTY restoration
and child reaping. Its eight-second/1-MiB journey caps and 30-second outer timeout
are unchanged. Owned fixtures isolate HOME/XDG state and are removed after the
checks. No historical feature scripts or complete command-dispatch suite run.

The modal journey overrides C-M-f with forward-char and binds forward-sexp to
M-j in keys.vim_insert. It proves both the saved one-character effect and the
replacement hint. Independent factory checks retain all six navigation chords.
Development runs corrected fixture assumptions about hint precedence and the
independent modal map; no runtime change or timeout extension was needed.

Literal SHA-256 checks preserve the frozen catalog, mapping manifest, reference
JSON/Elisp and mate contract. Historical source bytes are not regenerated from
runtime output. New native cases protect decoded Repeat/Release metadata and
exact completed heading cells in two shared, read-only Org/Markdown views
through every pending navigation turn.

## Contracts Added or Changed

No production policy, command, binding or resource bound changes. Counted
navigation retains 262,144 inspected bytes, depth 256, 2,097,152 metadata bytes
per buffer, 256 work steps per poll and count magnitude 1,000,000. Four
milliseconds remains a scheduling target. Source/view/point/mark/focus changes
and C-g retire pending publication; queued input runs after one complete target.
Group/expression targets remain distinct from typed Confirmed mate evidence.

The first four delivered reports remain authoritative for their own commits and
integration. This report supplies their integrated local acceptance evidence:
[mate](slice-1-mate.md), [groups](slice-2-groups.md),
[nesting](slice-3-nesting.md) and [expressions](slice-4-expressions.md).
Navigation stays Approved, editing Completed, presentation Draft and the shared
model Partially approved. Paragraph motion, EIP04 as a whole and F2 stay pending.

## Files of Interest

- `scripts/check-delimiter-navigation.sh`: nonzero native selections and owned fixture orchestration.
- `examples/delimiter-navigation-acceptance.rs` and its journey module: two physical-input workflows.
- `tests/pair_navigation/acceptance.rs`: decoded input and exact heading-cell continuity.
- `tests/pairs/regions/helpers.rs` and `tests/pairs/typing/lifecycle.rs`: exact additive inventory expectations.
- User Guide, command reference and Unreleased entry: cohesive delivered navigation.
- Navigation spec/model/index and EIP04 expression progress: integrated, with combined-gate closure pending.

## Validation

All five required commands passed through looper validate in canonical order at
clean, pushed `74927143e7ad04b5ec594e4b7d6ddddcae0395d4` in 467.503 seconds,
using one Cargo build job, disabled incremental output and no dev/test debuginfo.
Canonical PR #66 integrated that commit into dev. Acta records validation as
`01M7G9Q9N97FJ3WHR41DVNTS82` and the merge as `01M7G9RSB59G7SD9454RJ3RHM5`.
The controller owns the combined delivery gate. The earlier development timings
below remain observations of their original runs.

- PASS: `cargo fmt --all -- --check`: 4.204 seconds.
- PASS: `bash scripts/check-delimiter-navigation.sh`: 520.992 seconds; 180 native cases, one complete factory helper and exactly two Linux PTYs.
- PASS: `bash scripts/check-documentation.sh`: 27.095 seconds; 717 commands, 332 configuration entries, 112 reachable pages, 794 local links/anchors and 22 focused reference tests.
- PASS: `bash scripts/build-user-guide.sh`: 4.247 seconds; strict Spore links/render and local target/userguide output, no publication.
- PASS: `git diff --check`: 0.105 seconds.

| Native batch | Passed cases |
| --- | --- |
| Complete navigation | 48 |
| Complete shared pairs | 78 |
| Configuration keymaps/pairs | 22 |
| Count/history lifecycle | 3 |
| Macro motion/count/region | 10 |
| Modal percent/objects | 2 |
| Org / Markdown continuity | 6 / 7 |
| Native point/mark/focus invalidation | 4 |

The successful ordered run took 556.643 seconds including focused compilation.
The factory helper compares all 28 maps in both profiles, 717 commands and six
actions against independent frozen expectations plus literal approved additions.
All five frozen digest checks pass. The shared pair retention boundary measured
2,097,144 retained bytes before Limited(Retention), below the 2,097,152-byte cap.

| Physical journey | Elapsed ms | Output bytes | Peak RSS KiB | Idle CPU ticks |
| --- | --- | --- | --- | --- |
| Non-modal | 1,741 | 136,393 | 38,172 | 0 |
| Modal Insert with override | 1,853 | 143,416 | 42,360 | 0 |

Both journeys observed 350 ms of idle silence, unchanged config bytes, restored
TTY attributes, alternate-screen/paste/cursor restoration and successful child
reaping within the unchanged caps. Process RSS includes the editor; it is not
the request metadata counter. Owned temporary fixtures were removed on exit.

The first local script passed all 48 navigation cases and 76/78 existing pair
cases. Both failures were editing-era inventory expectations of 710 commands.
The two literal sets now add exactly the seven approved navigation names to the
three editing additions, retaining the immutable 707-command baseline, exact
set equality, descriptions and unbound editing helpers. The focused command
`env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pairs inventory -- --test-threads=1`
passes both cases. The initial diagnostic log was retained through slice validation.

Acceptance implementation is `2c67bdfdbd9baabedaf0d16ee4a698a6744fbe81`;
the two inventory updates are `263bde3ffe17c1dabadd2bbf72f353de1213c117`.
Guidance/report introduction is `7006dae9907357d3417ef13ce24876614c6651e7`.
Both exact staged hooks passed formatting, strict all-target/all-feature workspace
Clippy and native source policy across 1,971 files, two packages and 80 Cargo
roots. No hook, lint profile, acceptance limit or frozen input was changed.
The report-introduction and PR-reference commits pass the installed docs/data
whitespace and index-integrity hook. Canonical PR #66 merged into dev with the
exact approved title and immutable report link after its final clean-HEAD checks.

Fresh unoptimized requests measured 88 source-size samples across all four
navigation owners, complete/malformed forms, long atoms/strings and both
directions. Allocation precedes timing; the separate staged hook can compete for
CPU. Totals below are minimum/maximum microseconds across 22 samples per size.

| Source bytes | Total range us | Maximum poll us | Maximum inspected bytes | Maximum retained bytes |
| --- | --- | --- | --- | --- |
| 1,024 | 387 / 557 | 143 | 1,024 | 27,640 |
| 65,536 | 26,409 / 43,546 | 286 | 65,536 | 27,640 |
| 262,144 | 109,441 / 165,675 | 371 | 262,144 | 27,640 |
| 1,048,576 | 108,479 / 156,989 | 291 | 262,144 | 19,448 |

All sized samples remain within 256 work steps per poll. Oversized unproved
destinations refuse at the byte cap; nesting can accept a qualified prefix
without claiming its uninspected suffix. Dedicated cases admit depth 256, refuse
257, exercise million counts and dense retention refusal, and assert no partial
point, stale result, source/history or mark changes. An earlier preflight under
competing hook work observed a 13,422-us poll. Wall-clock maxima can exceed the
four-millisecond scheduling target without changing the work caps.

Actual configured native events on 8,194/8,198-byte sources measured dispatch /
completion including queued insertion: mate 309 / 25,804 us, groups 313 / 27,525
us, nesting 289 / 25,912 us and expressions 325 / 26,370 us. Mate queued-key
admission was 5 us. Cancellation and source/view replacement pass separately.

Both heading fixtures used 8,206 source bytes, two views and 32 continuation
turns. Exact glyph/style cells remained unchanged; Org measured 252,054 us
total / 8,015 us maximum host turn, Markdown 257,845 / 8,613 us. These host
turns include projection and cell comparison, not just the bounded lexical poll.

## Risks and Follow-ups

All five slices are integrated. Fetched dev ancestry, a clean/unlocked Slice 5
worktree and absence of owned processes were verified; the worktree was removed
without force. Runtime, acceptance, guidance and all five delivered reports are
present for the full gate. Only delivery-state Markdown changes in preparation.

The unchanged combined gate and the eleven-file Markdown-only closure remain
controller-owned. The plan estimates 10-20 minutes warm or 20-35
minutes cold for that combined gate; this worker's selected commands and hook
are separate evidence. No full delivery gate, hook installation, main promotion,
release or publication runs here. The existing vendored crossterm parentheses
warning is preserved.
