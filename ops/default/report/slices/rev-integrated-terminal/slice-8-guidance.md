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
placement without adding an image or runtime dependency.

## Contracts Added or Changed

The interactive section in the command catalog links the dedicated reference;
its stable anchor and complete command inventory remain available.

## Files of Interest

- `docs/tutorials/user-guide/16-integrated-terminal.md`
- `docs/how-to/use-integrated-terminal.md`
- `docs/reference/integrated-terminal.md`

## Validation

Pending the configured clean-HEAD Slice 8 commands.

## Risks and Follow-ups

Learning-path/catalog updates, baseline/model reconciliation and final configured
checks remain in this slice. The controller owns the combined gate after merge.
