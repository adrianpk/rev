# Slice 8: User Guidance and Joint Integration

Status: reviewing
Delivery set: rev-integrated-terminal
Plan: [Integrated Terminal Delivery](../../../plan/rev-integrated-terminal.md)
Tracker: [Integrated Terminal Tracker](../../../tracker/rev-integrated-terminal.md)
Branch: `feat/rev-integrated-terminal-08-guidance`
PR: pending

## Purpose

Make the integrated terminal discoverable through the User Guide, task guidance
and exact reference, then reconcile the accepted model and evidence.

## Delivered Behavior

The terminal chapter follows a real shell/lazygit session through panel and
main-buffer placement, multiple sessions, returning to source, copy and closure.
The task guide and dedicated reference describe all 37 commands, seven startup
settings, input/copy ownership, limits and pending/failure lifecycle behavior.

## Implementation Notes

Markdown remains the source for the Spore projection. Small text diagrams show
placement without adding an image or runtime dependency. Chapter 16 is linked
in the everyday editor path; the independent Org path remains separately
validated. Catalog checks compare all 37 terminal commands and seven settings
with the live command/schema/default owners, rather than a second manual list.

## Contracts Added or Changed

The interactive section in the command catalog links the dedicated reference;
its stable anchor and complete command inventory remain available. Accepted
baseline/model clauses now admit typed terminal resources without transferring
source/file/history authority. TRM01-TRM11 link their implementation reports;
combined-gate acceptance remains pending. Core contracts state their behavior
directly without relying on another product's capabilities.

## Files of Interest

- `docs/tutorials/user-guide/16-integrated-terminal.md`
- `docs/how-to/use-integrated-terminal.md`
- `docs/reference/integrated-terminal.md`
- `examples/documentation-check.rs` and `documentation_check/catalogs.rs`
- `publication/userguide/index.sptree`
- Terminal/editor specification and model companions under `ops/default/spec/`

## Validation

Focused receipt `Y2WNEWSV2PFZYVWAYCQOPNJ7A7` passed at
`10dbde18199a61eaf768bb1d81761632580d1267`:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh`
- `git diff --check`

The checker validated 76 reachable pages, 484 local links/anchors, independent
editor/Org paths, 474 commands and 247 configuration entries. The dedicated
terminal inventory covers all 37 live names and seven startup settings. Spore
strict link checks and rendering passed; the generated index and prior chapter
link chapter 16, whose HTML contains the actual move command examples.

The first focused receipt `RTY6FC3J2FVDWH4U7F2UM5KJ74` rejected a footer-only
Next link. The chapter now uses the established Next: line; no check was weakened.
Accepted TRM01-TRM11 requirement/scenario text is unchanged from the integrated
baseline. The controller records the final exact-PR-HEAD six-command receipt
separately. This is local guide generation, not website publication.

## Risks and Follow-ups

The unchanged combined gate remains controller-owned after Slice 8 merges.
Slice-local documentation and catalog checks do not close terminal delivery.
