# Slice 6: Highlighted path completion

Status: drafting
Delivery set: rev-wrap-navigation
Plan: [Wrap and Navigation](../../../plan/rev-wrap-navigation.md)
Tracker: [Wrap and Navigation Tracker](../../../tracker/rev-wrap-navigation.md)
Branch: `feat/rev-wrap-navigation-06-path-tab`
PR: pending

## Purpose

Deliver NAV04 and the complete UI03 amendment through the existing native
filesystem prompt, retaining separate completion and Return acceptance.

## Delivered Behavior

Tab fills the highlighted candidate, including the default first match of an
ambiguous prefix. File completion retains the prompt; directory completion
keeps its separator and observes immediate children without accepting a root.
Return still uses an explicitly navigated candidate or the freely typed draft.

## Implementation Notes

The existing provider, matching order and panel selection remain authoritative.
Refresh prepares a bounded replacement before publishing it, retaining the
selected raw name through sorting shifts. A disappeared name or failed read
keeps the prior observation, editable draft and selected identity. Retrying
cannot silently complete another row. Unsupported byte names remain rejected.
Query text changes reset selection; cursor movement and resize do not observe
disk. Incomplete snapshots keep their warning; Tab selects an observed identity
without making a uniqueness claim. The existing 4096-entry and 512 KiB raw-name
limits and configured prompt scalar limit remain unchanged.

## Contracts Added or Changed

Tab completes the visible choice instead of computing a shared prefix.
Selection does not imply Return acceptance until explicit navigation. Source
identity, dirty text, view state, history and filesystem contents are preserved.
Full UI03 baseline reconciliation and terminal continuation evidence are pending.

## Files of Interest

- `src/file/completion/navigation.rs`: atomic refresh and selected identity.
- `src/file/completion/navigation/complete.rs`: selected-name completion.
- `src/file/completion/candidate.rs`: private lossless selection identity.
- `tests/path_completion/panel.rs`: real keyboard and M-x completion routes.
- `tests/path_completion/navigation.rs`, `edges.rs`: names, bounds and failures.

## Validation

The native event-route reproduction failed against the prior shared-prefix
behavior: an ambiguous `al` draft did not complete the highlighted full file.
The corrected event-route regression passed both entry routes, implicit first
selection, both keyboard navigation pairs and cancellation with dirty source.
Clean-head controller evidence and T6.2 focused checks are pending.

## Risks and Follow-ups

Directory observation can wait on the filesystem within the existing finite
listing bounds. No background traversal or observation during rendering is added.
Complete T6.2 continuation, refresh regressions, contract/reference updates and
real-terminal evidence before preparing the canonical PR.
