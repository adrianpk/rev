# Slice 8: User guidance and joint interaction acceptance

Status: delivered
Delivery set: rev-wrap-navigation
Plan: [Wrap and Navigation](../../../plan/rev-wrap-navigation.md)
Tracker: [Wrap and Navigation Tracker](../../../tracker/rev-wrap-navigation.md)
Branch: `fix/rev-wrap-navigation-validation-2`
PR: [#183](https://forge.adrianpk.com/adrianpk/rev/pulls/183)

## Purpose

Join WRP01-WRP07, NAV01-NAV06 and UI03 through native event/composition and
representative Linux terminal acceptance, with reader guidance matching the
delivered interaction contracts.

## Delivered Behavior

The native journey opens a directory, completes a file with Tab without accepting
it, then clicks a different entry into a remembered split pane. Rust and Org
views retain their canonical documents while wrapping, folding, visual motion,
kill/yank/undo, cross-fragment selection/copy, search cancellation and resizing
operate together. Explicit visual-line-mode disablement verifies logical-line
motion; canceling a filename draft creates no file. The User Guide now explains
continuation rows, visual editing, browsing while split and explicit no-wrap
preferences. CLI, defaults, tree focus/root references and affected baseline
clauses agree with the implemented behavior; all five selected tickets are solved.

## Implementation Notes

The same observable journey runs against absent-file factory settings, the
packaged complete TOML and the compiled complete template. This verifies native
behavior for each source rather than inferring equivalence from the shared asset.
The existing configuration tests retain full schema and ordinary binding coverage.

The Bash entrypoint builds native release artifacts once, isolates runtime
HOME/XDG locations and owns one temporary fixture. Cargo retains its original
cache. The Rust terminal driver uses installed Linux script/stty/timeout tools,
real SGR wire input, completed ANSI redraws and the production executable.
It admits at most 1 MiB of output plus one read chunk and eight seconds per
terminal journey. Its timeout targets the editor directly; failure cleanup closes
input and reaps the owned process and reader. Successful exit checks exact saved
bytes, raw-mode acquisition, restored terminal attributes, alternate-screen/paste
cleanup and a 350 ms observation with no idle output.

The native session covers narrow 48-column split geometry. The PTY uses
100/140/120-column resizes so both split status rows retain their full source and
point fields. It waits on real state, not fixed startup delays. The native Org
request is settled through its existing deadline owner before checking folds.
Tab's full selected component remains relative when the typed query is relative;
the completed filename and accepted canonical file identity are checked separately.

The existing wrap-performance example supplies token, words, Unicode/tabs,
regional-indicator and 40,000-line workloads. Its finite 512-batch settling,
queued input/reversal, exact source serialization and retired idle-deadline
assertions remain intact. No new crate, Python gate or product dependency is added.
The expected added gate cost is one release build plus a subsecond PTY journey
and the existing bounded native probe, not a chain of historical terminal suites.

## Contracts Added or Changed

The new shell command implements the already pinned joint gate. It changes no
product setting, runtime behavior, gate identity or earlier evidence. T8.2
reconciles R02/R07/R28 and retained defaults, UI03/PTR02 references, directory
startup usage, the inventory and the existing Unreleased outcomes. GUT03 and its
consolidated copy already specify blank continuation labels and logical distances.
The complete config asset, compiled template, command and ordinary binding
catalogs retain the delivered soft_wrap/visual-line-mode and click policy.
Historical gate identities remain unchanged; final closure is recorded below.

## Files of Interest

- `scripts/check-wrap-navigation.sh`: isolated fixture/build/terminal lifetime.
- `examples/wrap-navigation.rs`, `examples/wrap_navigation/`: native journey and
  bounded Linux PTY adapter.
- `examples/wrap-performance.rs`: reused long-line input/layout measurements.
- Core User Guide editing/tree/preferences chapters, CLI/default/tree references
  and `CHANGELOG.md`: delivered user behavior and explicit alternatives.
- `ops/default/spec/rev-editor.md`, soft-wrap/navigation specs and the operational
  index/tracker: affected clause and solved-ticket reconciliation.

## Validation

Development command `bash scripts/check-wrap-navigation.sh` passed the native
journey under all three settings sources and the Linux PTY route. The latter
completed in 462 ms with 70,026 output bytes, exact saves and restored attributes.
The five native performance fixtures passed; maximum observed native step was
1.958 ms, maximum resize was 6.695 ms and maximum queued reversal was 13.986 ms.
These are local measurements, not portable timing guarantees. All five selected
tickets resolve to one solved path with canonical PR evidence; reports 1-7 are
delivered. T8.1/report introduction is commit
`ea088b08e97c1c108fabf81da857f7f6f3fbdbfe`. Final clean-HEAD
controller evidence is recorded after the task and PR-reference commits.

T8.2 commit `30945f95121559878db1e9f80c8b848bce59d475` passed controller receipt
`GS6IDBXSHHMCRIIXMPJ6MBN7KW` with these exact ordered commands:

1. `cargo fmt --all -- --check`
2. `cargo test --locked --test wrapping --test path_completion --test tree --test pointer --test configuration --test commands`
3. `bash scripts/check-wrap-navigation.sh`
4. `cargo run --locked --quiet --example documentation-check`
5. `git diff --check`
6. `cargo clippy --locked --example wrap-navigation -- -D warnings`

The six native suites passed 499 tests: 161 command, 133 configuration,
46 path-completion, 65 pointer, 68 tree and 26 wrapping cases. The shell gate
passed all three native settings journeys and the PTY route (464 ms, 70,025
bytes). Long-line maximum observed native step was 2.261 ms; maximum resize
was 6.316 ms and maximum queued reversal was 16.851 ms. Documentation passed
305 commands, 180 configuration entries, 58 pages and 331 local links/anchors.
Focused Clippy passed without warnings.

Canonical PR #181 merged into `dev` at
`4ec549c581d919b72e3e2e89cb4752507e0b1380`. Final configured receipt
`AUDRUYJUFLGQSXH3OTQIH56NFY` passed the first five commands above on that exact
clean head: 499 tests, all three native settings journeys, the Linux PTY route
(472 ms, 70,025 output bytes), all five long-line fixtures and documentation.
The controller recorded the merge in Acta as `01M682VDRQ3BVAVDSPBZAYWXC3`.
Slice 8 is delivered; final integrated validation and closure are recorded below.

## Gate repair 1: failed-open fixture

The original Slice 8 branch `feat/rev-wrap-navigation-08-interaction-acceptance`
was delivered through PR #181 as recorded above. This report also owns the
pinned repair `test(gutter): use an invalid file for failed-open coverage`.

The controller's full gate at `90712acf1ba4dc4bb8db091b87285f0929897039`
passed formatting and all-target Clippy, then failed in the all-target test run
at `gutter::lifecycle::file_rebind`. Retained diagnostic identity is
`TQRG3K3G75J6KTCHNZUYIPHCIA`; 35 other gutter tests passed. The assertion used
the fixture directory as a rejected file-open target, but NAV01 deliberately
makes directory opening valid. The directory fixture therefore no longer
exercised the test's intended failed-ingress path.

`tests/gutter/lifecycle.rs` now creates a real file containing invalid UTF-8
and requires `InvalidData` from loading it. The existing complete active-view,
selected-window, shared-peer, BOM/CRLF/tab/Unicode byte fidelity, clean-state
and empty-history assertions remain intact. This preserves failed-file coverage
while respecting accepted directory targets. No runtime or gate requirement is
changed; only that test file and this existing report are modified.

Focused reproduction `cargo test --locked --test gutter lifecycle::file_rebind -- --exact`
failed at the retained assertion before the correction and passed afterward
(one test, 35 filtered). The pinned final repair command order is:

1. `cargo fmt --all -- --check`
2. `cargo test --locked --test gutter`
3. `cargo test --locked --test files`
4. `git diff --check`

Repair commit `3fe555017ec8b3c3a22af8ac86c826ca47fdaa64` passed all four
commands above at its exact clean head in controller receipt
`NG64D6QWH24UWQV76BJ4CBDT2M`: 36 gutter tests and 74 file tests passed.
Canonical [PR #182](https://forge.adrianpk.com/adrianpk/rev/pulls/182) merged
into `dev` at `16debf9b98acdac1581a203726041b61ac036774`. Final controller
receipt `VBNOTDVN3ENDHZIBSSQUQ4ERCJ` passed the four policy-required commands
and all 110 tests on that exact clean PR-reference head. The controller recorded
the merge in Acta as `01M68JWDE5DYK7H87XWWFGA9R8`. Gate repair 1 is delivered.
At repair 1 integration, the unchanged full gate awaited its controller rerun.
The final pass below preserves earlier slice and failed-gate candidate identities.

## Gate repair 2: explicit horizontal clipping fixtures

The covering additional authorization pins this repair to
`test(syntax): keep horizontal clipping checks explicitly unwrapped` and the
failed candidate `ef210e1aef98b4e9d6a907a46767c90f8d2d3ca1`. The retained
full-gate diagnostic `AWKEE7BOZ53PGQZYEQ6YE4DDNM` reached the syntax suite after
formatting and all-target Clippy passed. `edges::cell_projection` and
`incremental::failures::visible_queries` failed; the other 28 syntax tests passed.
Focused reproduction `cargo test --locked --test syntax` reproduced both failures.

Both fixtures set raw source-row/horizontal offsets but inherited the newly
wrapped factory policy. Those offsets therefore did not describe their intended
horizontal clipping setup. Each now asserts the wrapped factory state, explicitly
toggles only its own live buffer to the WRP06 unwrapped alternative, and asserts
that result before constructing the view. No shared helper or startup default is
changed; other syntax fixtures and the wrapping suite keep their existing policy.

The tests retain their source data, three wide-glyph clipping offsets, exact
string paint ranges, unstyled partial-glyph cell, 1,000-line workload, incremental
Unicode edit, at-most-32 capture bound, exact byte containment, 16-cell paint bound
and `item_549` row assertion. This corrects fixture setup for the approved explicit
horizontal alternative without replacing or weakening its observables. The repair
changes only `tests/syntax/edges.rs`, `tests/syntax/failures.rs` and this report.

The pinned final command order is:

1. `cargo fmt --all -- --check`
2. `cargo test --locked --no-fail-fast --test syntax --test wrapping --test tree --test windows`
3. `git diff --check`

Repair commit `413d805ccf9bf6e964313c313a809f08af059eef` passed all three
commands above at its exact clean head in controller receipt
`A3XDJU5G537CYNKN3LOHXY7HKO`: 30 syntax, 26 wrapping, 68 tree and 41 window
tests passed (165 total). Both reproduced syntax failures now pass with every
original clipping and bounded-query assertion retained.

Canonical [PR #183](https://forge.adrianpk.com/adrianpk/rev/pulls/183) merged
into `dev` at `ccfc383bf301425ebb21cd95bb5fb6de32c3e0cc`. Final controller
receipt `XSCA366D6QCOO6S3NT7UIYPBPI` passed the three policy-required commands
and all 165 tests on that exact clean PR-reference head. The controller recorded
the merge in Acta as `01M68KC86KT18T7ZKR7F4R1AR7`. Gate repair 2 is delivered.
At repair 2 integration, the unchanged full gate awaited its controller rerun.
The final pass below preserves prior repair, slice and failed-gate identities.

## Delivery-set closure

The controller passed the exact pinned full gate at
`6b2afd4e07f269c979cd26faf60b6863314e69e9` in receipt
`F7OQU2CWIOPB3VW234VVECCQNG`. All-target testing passed 1,800 tests. The
three native settings journeys and Linux PTY route passed (448 ms, 70,025 bytes),
with exact source/save fidelity, idle silence and restored terminal attributes.
All five long-line probes passed; maximum observed native step was 1.645 ms,
resize 5.837 ms and queued reversal 12.582 ms. Documentation passed 305 commands,
180 configuration entries, 58 pages and 331 local links/anchors. Earlier focused
and failed-gate evidence remains identified above.

All eight slices and both authorized repairs are integrated; all five selected
tickets are solved. The delivery set is completed by the documentation-only
closure recorded in the [tracker](../../../tracker/rev-wrap-navigation.md).

## Risks and Follow-ups

The terminal route requires the installed Linux util-linux/coreutils tools. Its
finite output/time limits deliberately fail on lost input or redraw loops. The
controller owns the unchanged final integrated gate and delivery closure.
