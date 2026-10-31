# Slice 6: Highlighted path completion

Status: delivered
Delivery set: rev-wrap-navigation
Plan: [Wrap and Navigation](../../../plan/rev-wrap-navigation.md)
Tracker: [Wrap and Navigation Tracker](../../../tracker/rev-wrap-navigation.md)
Branch: `feat/rev-wrap-navigation-06-path-tab`
PR: [#179](https://forge.adrianpk.com/adrianpk/rev/pulls/179)

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
The full UI03 replacement, including both scenarios, is applied to Editor
Interaction and consolidated R12. Their historical candidate/gate evidence
remains unchanged; this implementation belongs to rev-wrap-navigation.

## Files of Interest

- `src/file/completion/navigation.rs`: atomic refresh and selected identity.
- `src/file/completion/navigation/complete.rs`: selected-name completion.
- `src/file/completion/candidate.rs`: private lossless selection identity.
- `tests/path_completion/panel.rs`: real keyboard and M-x completion routes.
- `tests/path_completion/navigation.rs`, `edges.rs`: names, bounds and failures.
- `tests/path_completion/continuation.rs`, `stale.rs`: actual continuation,
  clipped selection, disappeared identities and denied-read retries.
- `scripts/check-path-navigation.py`: existing two-session RGB/NO_COLOR probe.
- `ops/default/spec/editor-interaction.md`, `rev-editor.md`: full UI03 amendment.
- `docs/reference/path-completion.md`, `prompts.md`, `defaults.md` and the file-tree
  User Guide: selected completion and separate acceptance.

## Validation

The native event-route reproduction failed against the prior shared-prefix
behavior: an ambiguous `al` draft did not complete the highlighted full file.
The corrected event-route regression passed both entry routes, implicit first
selection, both keyboard navigation pairs and cancellation with dirty source.
Development checks then passed all 46 path-completion tests and two existing
RGB/NO_COLOR terminal sessions:

- `cargo test --locked --test path_completion`
- `python3 scripts/check-path-navigation.py`

The terminal probe now exercises an ambiguous first directory, M-x navigation
into the alternate directory, another selected Unicode/spaced child, Tab with
candidate rows hidden, explicit Return, exact file preservation and cleanup.
It reuses the existing PTY driver and finite waits/output limits. No new gate or
runtime dependency is added. Native tests cover cached observation, exact entry/
name-byte ceilings, unsupported names, prompt bounds, both real key routes,
repeated disappeared selections, permission recovery and dirty source/history.

Legacy shared-prefix expectations now assert the approved full selected name;
input-limit rejection preserves the original draft instead of installing an
intermediate prefix. Incomplete-list feedback and finite observation assertions
remain active. Selection from an incomplete list completes one observed identity,
without inferring uniqueness or expanding the observation budget.

The first configured run found four stale command-suite expectations from
Slice 5: directory Return descent, Return draft-insertion limits, expansion as
implicit refresh, and the incomplete tree-context binding table. The command
fixtures now exercise final directory acceptance, bounded Tab insertion, fresh
find-file aliases, explicit tree refresh and the full native binding set. Input
phase, dirty-state, disk preservation and recursive deletion consent assertions
remain active; no runtime admission or acceptance limits are weakened.

T6.1 commit `28ff1967c7330a6619e144a70487f6dc73d7a3cd` introduces the
implementation and this report. T6.2 commit
`1f061dea66754b5090dcbd42b24faad2d0f7a46c` completes continuation, retained
error kinds, regression coverage and full UI03/reference reconciliation.
Commit `e1122a0ba59b326671c9ab97109be6191c1b801f` aligns the stale command
expectations and completes the tree binding table.

Controller receipt `3I26EZ2A36CO2PZGTFKU2M7DGM` passed these exact ordered
commands at clean head `e1122a0ba59b326671c9ab97109be6191c1b801f`:

1. `cargo fmt --all -- --check`
2. `cargo test --locked --test path_completion --test files --test commands --test configuration`
3. `git diff --check`
4. `cargo clippy --locked --all-targets -- -D warnings`
5. `cargo run --locked --quiet --example documentation-check`
6. `python3 scripts/check-path-navigation.py`

The native suites passed 414 tests: 161 command, 133 configuration, 74 file and
46 path-completion cases. Documentation passed 305 commands, 179 configuration
entries, 58 pages and 330 local links/anchors. Both terminal sessions passed,
with RGB and NO_COLOR selection, cancellation/acceptance and fixture cleanup.

Canonical PR #179 merged into `dev` at
`e9aadd9805bcd5a8b418d3575ffcd974bfa080e7`. Final configured receipt
`AQEZSXJGMWQTINQSMKLKESZQP6` passed the exact three-command prefix and all
414 tests on that clean PR-reference head. The controller recorded the merge
in Acta. Slice 6 of 8 completed; the path-completion ticket is solved. The
combined delivery gate remains reserved for final acceptance.

## Risks and Follow-ups

Directory observation can wait on the filesystem within the existing finite
listing bounds. No background traversal or observation during rendering is added.
The path-completion ticket is solved after verified integration.
