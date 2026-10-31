# Slice 5: Directory targets and initial tree expansion

Status: drafting
Delivery set: rev-wrap-navigation
Plan: [Wrap and Navigation](../../../plan/rev-wrap-navigation.md)
Tracker: [Wrap and Navigation Tracker](../../../tracker/rev-wrap-navigation.md)
Branch: `feat/rev-wrap-navigation-05-directory-targets`
PR: pending

## Purpose

Deliver NAV01, NAV02, NAV03 and NAV06 through shared CLI/find-file target
admission, explicit root intent and one-time first-level tree initialization.

## Delivered Behavior

T5.1 admits a directory through the same session opening owner
used by startup, find-file and tree file activation. Directory acceptance
finishes the prompt, presents its root and preserves canonical document state.
An empty prompt draft accepts its base directory. File visits preserve an
explicit root even outside it; automatic roots follow the current file parent
unless the configured cwd policy applies.

## Implementation Notes

File and directory targets share checked path-component and alias resolution.
Regular-file Location/Observation contracts stay intact. Candidate root/list
preparation precedes publication; failed opening retains the old root and
source/view/history. A directory-only session retains its existing scratch
buffer. Find-file remains available while the tree owns focus.

## Contracts Added or Changed

Return accepts a directory as a complete browsing target. Tab keeps its
existing completion behavior for the separately approved Slice 6.
Explicit directory intent takes precedence over automatic file-parent roots.

## Files of Interest

- `src/session/file.rs`: shared target admission and publication.
- `src/tree/root.rs`: prepared explicit and automatic root choices.
- `src/file/location.rs`: shared component/alias resolution.
- `src/file/completion/navigation/accept.rs`: checked final target spelling.
- `tests/files/directories.rs`: root precedence, empty targets and failures.

## Validation

Development checks passed:

- `cargo check --locked`
- `cargo test --locked --test files directories::` (three native cases).
- `cargo test --locked --test path_completion --test tree` (105 tests).
- `git diff --check`

Historical expectations that directories descend in the prompt or that every
file visit preserves an automatic root now assert the approved directory-open
and explicit-root behavior. Regular-file ingress still rejects directory text.
 Clean-head controller evidence is
required before canonical PR preparation. The combined delivery gate is not
part of this slice's focused checks.

## Risks and Follow-ups

Complete T5.2 first-display initialization, deliberate collapse/hide/show,
narrow-geometry focus restoration, bounded listing diagnostics and focused
native/CLI coverage. Update the two directory tickets and product references
with final exact-head evidence before handoff.
