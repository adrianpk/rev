# Slice 8: User guidance and joint interaction acceptance

Status: reviewing
Delivery set: rev-wrap-navigation
Plan: [Wrap and Navigation](../../../plan/rev-wrap-navigation.md)
Tracker: [Wrap and Navigation Tracker](../../../tracker/rev-wrap-navigation.md)
Branch: `feat/rev-wrap-navigation-08-interaction-acceptance`
PR: pending

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
Historical gate identities remain unchanged; the final delivery state stays open.

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

## Risks and Follow-ups

The terminal route requires the installed Linux util-linux/coreutils tools. Its
finite output/time limits deliberately fail on lost input or redraw loops. The
controller owns the unchanged final integrated gate and delivery closure.
