# Slice 8: User Guidance and Joint Integration

Status: drafting
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

## Validation

Pending the configured clean-HEAD Slice 8 commands.

## Risks and Follow-ups

The unchanged combined gate remains controller-owned after Slice 8 merges.
Slice-local documentation and catalog checks do not close terminal delivery.
