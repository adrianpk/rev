# Slice 8: Native acceptance, footprint and guidance

Status: delivered
Delivery set: rev-autocompletion
Plan: `ops/default/plan/rev-autocompletion.md`
Tracker: `ops/default/tracker/rev-autocompletion.md`
Branch: `feat/rev-autocompletion-08-acceptance`
PR: `#202`

## Purpose

Connect native completion's language and interaction contracts through actual
editing, finite resource observations and a release-editor PTY. Complete the
ordinary editing guidance required by the [plan](../../../plan/rev-autocompletion.md)
and [tracker](../../../tracker/rev-autocompletion.md).

## Delivered Behavior

The Rust/Bash acceptance entrypoint checks all ten bundled parser/query pairs,
Unicode replacement, selection, cancellation, resize, checked acceptance and
one-step undo/redo. Automatic typing, paste suppression, six native Org contexts,
a real externally compiled Python grammar and bundled Rust fallback use actual
sessions. Source/history observations and exact unsaved/disk bytes are asserted.

The normal release editor also runs through a Linux controlling PTY using its
ordinary arguments and commands. Manual cancellation/selection/acceptance,
automatic debounce, literal insertion, undo, unchanged disk bytes, idle silence
and terminal attributes/screen/paste/cursor restoration are checked together.

The User Guide gains a seventh ordinary editing chapter with a small cursor-list
example. Focused procedures cover completion controls and versioned external
assets; references and Unreleased link the delivered behavior. The independent
Org learning path remains supplementary. Existing exhaustive catalogs still
cover all 326 commands and 202 configuration entries.

## Implementation Notes

The wrapper preserves the original Cargo/Rustup homes while isolating HOME,
XDG roots and source files. It builds with the normal release profile, locked
dependencies and two jobs. Owned Rust/Python libraries are compiled before any
editor construction. Runtime only loads supplied assets. PTY children retain
eight-second deadlines, a 1 MiB output bound and owned reader/process cleanup.
No installed editor or user configuration/store is used.

The approved all-production source check exposed existing parameter, nesting
and complexity violations in bookmark/store/navigation, coordinate/display/wrap,
key dispatch and session layout/pointer/file owners. Cohesive helpers and small
argument records make those owners pass the unchanged checker across all 700
production Rust files. Actual crate roots and the introduced example roots and
modules are selected explicitly. Check budgets, cooperative charge order,
held-input ordering, bookmark publication checks and existing assertions remain
in force. Shared grammar fixtures now use one fixture module per test/example
binary. No source-check exclusions or allowances were introduced.

## Contracts Added or Changed

`bash scripts/check-completion.sh` provides native acceptance and reports actual
size, bounded observations, burst timing, peak RSS and idle CPU work. Source
admission over 2 MiB or 4096 symbols publishes no partial popup and preserves
source/history. Forty open peer buffers yield only 31 retained peer names after
the current buffer, omitting the oldest observations. A 128-event insertion/
reversal burst over 1024 declarations retains exact source and returns to an
idle state without a completion deadline. Timing is observed, not a new latency
guarantee. Strict worker/candidate/snapshot limits remain unchanged.

Documentation distinguishes observed syntax names from semantic LSP proposals.
This slice contributes native acceptance for CMP01 and CMP03-CMP09. CMP02's
semantic/native source switching remains assigned to the approved LSP successor.

## Files of Interest

- `examples/completion-acceptance.rs`, `examples/completion_acceptance/`: native workflows, checked source/history, resource probes and controlling PTY.
- `scripts/check-completion.sh`: isolated ownership, normal-release build and complete production source check.
- `src/bookmark/`, `src/coordinates/`, `src/display/`, `src/view/`, `src/session/`, `src/input/`: helper decomposition required by the unchanged source bounds.
- `tests/support/language_assets.rs`: shared real native-library fixture ownership.
- `docs/tutorials/user-guide/10-complete-names.md`, `docs/how-to/use-completion.md`, `docs/how-to/register-grammar.md`: guided insertion and focused procedures.
- `examples/documentation-check.rs`, documentation indexes/references and `CHANGELOG.md`: checked navigation, exhaustive catalogs and user-facing outcomes.

## Validation

T8.1 begins at `8955c808c29a25a903698eb6cc61de6ca0ec7aa6`; follow-up commits
correct static diagnostics, preserve the held-request assignment and share the
fixture root. T8.2 is `f3ca520791ebb8777670e73d706af338019cb775`, with canonical
Next navigation corrected at `642fa80e874d01383eff0cb71b77b0dad15738b3`.

Focused controller receipt `FCMVJ7OOV26XSCOJDACS26DCQB` passed at clean
`dd6babbb7a68cf5e24ddb8b7da871b4afd5b241b`: all-target Clippy and 575 existing
library/bookmark/command/configuration/file/display tests, in this order:

1. `cargo clippy --locked --all-targets -- -D warnings`: passed.
2. `cargo test --locked --lib`: passed, 107 tests.
3. `cargo test --locked --test bookmarks --test commands --test configuration --test files --test display`: passed, 468 tests.
4. `git diff --check`: passed.

At clean `f3ca520791ebb8777670e73d706af338019cb775`, controller run
`FZI462AXGH5EJ2BUATTOE56UES` passed `bash scripts/check-completion.sh`, including
all-production source quality and native/PTY workflows. Its subsequent
`cargo run --locked --quiet --example documentation-check` failed the new chapter's
Next-link format. Receipt `HGUZ45RRL2DSHXUPPJUMQI33TS` then passed the unchanged
documentation command and `git diff --check` at clean
`642fa80e874d01383eff0cb71b77b0dad15738b3`: 63 reachable pages, with independent
editor/Org paths and complete command/configuration catalogs.

The release run used Linux x86_64, Rust 1.98.0, an AMD Ryzen 5 7430U with 12
logical CPUs and 100 CPU ticks/second. These are local observations from that
candidate, not measurements of the later report/PR-reference commits:

| Observation | Result |
| --- | --- |
| Normal release executable | 18,068,672 bytes |
| Recorded baseline at `6e38bb95fdf0ff81cec6d48a6e8b3bdaa9130fda` | 13,326,584 bytes |
| Delta / ratio to baseline | +4,742,088 bytes / 1.355837 |
| Bounded 40-peer request | 31 retained names; 2,018 microseconds |
| 1024-declaration, 128-event burst | 207,119 microseconds total; longest event/frame 736 microseconds |
| Native idle / PTY idle, 350 ms each | 0 / 0 CPU ticks; PTY silent |
| Native acceptance / editor PTY peak RSS | 29,904 / 13,900 kB |
| PTY output | 44,191 bytes; exact source and restored terminal |

The recorded baseline uses the same normal release profile, locked dependencies
and two jobs. No post-link stripping/compression is added. The earlier grammar
experiment's 1.330297 ratio is comparative evidence, not a size ceiling.

Earlier focused Clippy runs failed on iteration style, item ordering and duplicate
fixture modules; corrections retained the checker and assertions. One intermediate
layout assignment lacked a semicolon; it was restored before the successful
focused receipt. Fixture diagnostics also corrected JSON's complete quoted
insertion and Org's encoded heading target, and made PTY navigation wait for the
observable popup before sending selection keys. No observable assertion or
resource bound was weakened.

Final controller receipt `HXYP54647FFOQV7IHZWI3OXGLO` passed all six required
checks and 421 tests at clean PR candidate
`646723ab0b1c123c53df042b290d1ac73b2c35e5`, in this exact order:

1. `cargo fmt --all -- --check`
2. `cargo test --locked --test completion --test language_assets --test syntax --test coordinates --test wrapping --test pointer --test org_editing`
3. `bash -n scripts/check-completion.sh`
4. `bash scripts/check-completion.sh`
5. `cargo run --locked --quiet --example documentation-check`
6. `git diff --check`

Canonical [PR #202](https://forge.adrianpk.com/adrianpk/rev/pulls/202) links the
full immutable report-introduction commit
`07473382afc1097b3a9e5c189842cc11c09def6b`. Final validation covers the
PR-reference commit. Canonical PR #202 merged through Rebase + Fast-forward
into dev at `646723ab0b1c123c53df042b290d1ac73b2c35e5`. Slice 8 of 8 completed.

That final candidate repeated all ten parser workflows, Org and real external
assets, 700-file production source quality, checked PTY edits and restoration,
and the complete documentation catalogs. Release size remained 18,068,672 bytes
(delta +4,742,088; ratio 1.355837). The 40-peer request took 2,269 microseconds;
the 128-event burst took 213,909 microseconds with a longest event/frame of
753 microseconds. Native and PTY idle probes each recorded zero CPU ticks.
Peak native/PTY RSS was 31,496/13,700 kB; PTY output was 44,191 bytes.

## Delivery-set closure

The controller passed the unchanged full gate on immutable integrated candidate
`0f7cc36cb1082aa56954fa602294f47aac80e889`, receipt
`DAIRXONWTTLVG6RDIQQZKBSGXA`, on 2026-10-31. Exact successful command:

```sh
cargo fmt --all -- --check && cargo clippy --locked --all-targets -- -D warnings && cargo test --locked --all-targets && bash scripts/check-completion.sh && cargo run --locked --quiet --example documentation-check && git diff --check
```

The full gate passed formatting, all-target Clippy and tests, all 700 production
source-quality checks, native completion acceptance and real Linux PTY
restoration. Documentation passed with 326 commands, 202 configuration entries,
63 reachable pages and 371 local links/anchors. The recorded gate took about
7 minutes 50 seconds, including a 3 minute 16 second release build.

The integrated release measured 18,068,672 bytes against the recorded
13,326,584-byte baseline: +4,742,088 bytes, ratio 1.355837. Forty open peers
retained 31 names, with a 2,708 microsecond request. The 1024-declaration,
128-event burst took 210,681 microseconds; its longest event/frame was
762 microseconds. Native and PTY idle probes each measured zero CPU ticks
over 350 ms. Peak native/PTY RSS was 29,616/13,844 kB; the PTY emitted 44,191
bytes and restored terminal attributes and screen/paste/cursor state.

All eight canonical PRs are integrated. No gate repair was needed. Owned fixture
processes and temporary roots finished their checks and were cleaned up by
their owners. Documentation closure changes only configured operational paths;
the tested runtime candidate and historical receipts remain unchanged. Native
CMP01 and CMP03-CMP09 acceptance is complete. CMP02 semantic/native switching
remains assigned to the approved LSP successor and its separate gate.

## Risks and Follow-ups

Measurements describe this machine and fixture, not a production latency or
memory guarantee. Native grammar code remains explicitly trusted and mapped for
process lifetime. Native delivery and its full gate are complete. The approved
LSP successor owns semantic/native source switching and controller handoff.
