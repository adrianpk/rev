# Slice 1: Checked manual completion

Status: reviewing
Delivery set: rev-autocompletion
Plan: [Native Autocompletion Delivery](../../../plan/rev-autocompletion.md)
Tracker: [Native Autocompletion Tracker](../../../tracker/rev-autocompletion.md)
Branch: `feat/rev-autocompletion-01-session`
PR: pending

## Purpose

Deliver the manual Rust path for CMP01, CMP03-CMP04 and the manual part of
CMP05 through retained syntax, native input and canonical buffer transactions.

## Delivered Behavior

`M-TAB` and `M-x completion-at-point` show declarations from the current unsaved
Rust buffer. A compact cursor popup preselects the first candidate, fits above
or below point in the selected pane and preserves source focus and text layout.
Arrows or C-p/C-n navigate; Tab/Return accepts; Escape/C-g cancels. Outside a
popup, ordinary bindings remain active. Other editing, movement, prefixes,
prompts and source changes dismiss the list before their normal action.

Display labels include declaration kinds; literal insertion text excludes those
annotations. Smart-case filtering ranks prefixes before ordered subsequences,
then declaration proximity and stable lexical ties. Equal insertions are
deduplicated. Acceptance replaces the whole identifier, including Unicode/raw
identifiers and suffix text, as one ordinary undo operation. Cancellation and
failed admission leave source and history unchanged.

## Implementation Notes

The existing syntax owner retains the sole Rust parser/tree and compiles an
explicit completion query beside its highlight query. Direct function, type,
trait, module, field, constant, local and parameter declarations supply names;
ordinary identifier uses do not. There is no provider registry or second index.

One disposable session binds invocation identity, view, buffer revision, point
and replacement span. Cancelled/superseded results cannot be installed; edits,
point movement and view/buffer changes reject obsolete targets. Canonical
preparation rechecks read-only state and publishes all shared view points/history.
Source parsing/querying is confined to explicit invocation; drawing reads only
the retained candidates and the ordinary source frame.

The manual path admits 2 MiB, 4096 symbols, 512-byte labels and 200 candidates.
Parse/query progress callbacks share a 50 ms work deadline and discard limited
results. Current-buffer-only synchronous manual requests establish this slice;
automatic scheduling, broader declaration/scope coverage and other open buffers
remain with their planned slices.

## Contracts Added or Changed

Five working commands enter the compiled catalog: `completion-at-point`,
`completion-next`, `completion-previous`, `completion-accept` and
`completion-cancel`. The complete startup sample adds the M-TAB editor binding;
no automatic setting or inactive toggle is exposed. Command/binding/default
catalogs, the completion reference and Unreleased describe the manual behavior.
Popup selection and border colors use existing palette roles.

## Files of Interest

- `src/completion.rs`, `src/completion/`: proposal identity, matching and ranges.
- `src/syntax/completion.rs`, `src/buffer/syntax.rs`: bounded retained Rust queries.
- `src/session/completion.rs`: invocation, invalidation and canonical publication.
- `src/input/completion.rs`, `src/command/completion.rs`: native controls.
- `src/display/native_completion.rs`: pane-bounded cursor overlay.
- `tests/completion.rs`: real command/edit/history and rendered popup coverage.
- `docs/reference/completion.md`: current user-facing contract.

## Validation

Development checks passed:

- `cargo test --locked --lib completion`: 3 initial library tests passed.
- `cargo test --locked --test completion --lib completion`: 5 library tests
  passed; the name filter selected no integration cases in that invocation.
- `cargo test --locked --test completion`: all 8 interaction cases passed.

Coverage includes literal replacement/undo/redo, preselection, arrows/C-p/C-n,
cancel, Unicode/raw identifiers, obsolete revision/view/point/buffer targets,
retired invocation results, read-only invocation/publication, invalid coordinates,
source immutability, deadline admission, normal command-prefix continuation,
held acceptance keys, above-cursor placement, split isolation and tiny/resize
geometry. The final library batch also covers oversized-buffer refusal.

T1.1 is commit `f80f24b2d61c4dfba385d6206865779b46169137`.
T1.2 introduces this report with the integrated manual interaction.
After the PR-reference follow-up, the controller validates the final clean HEAD
and retains its immutable receipt outside the repository. The canonical PR
Validation section lists that batch's successful commands in order, including
the required format and six-suite regression prefix, focused completion-library,
configuration, new-owner source-quality and documentation checks, then diff
hygiene. The full delivery-set gate remains controller-owned after Slice 8.

## Risks and Follow-ups

A diagnostic source-check over the changed dispatch files also reports eight
pre-existing complexity violations, reproduced against the unchanged `dev`
baseline `609d538129870837b2e18b9928964857da90495d`: `poll_frame`, `outcome` and
`input_feedback` in `src/session.rs`; `focus`, `label` and `focused` in
`src/input.rs`; `name` in `src/command.rs`; and `handle_ready_at` in
`src/session/events.rs`. This slice adds no violation to those functions.
The focused source-quality batch validates the new completion owners. The
approved Slice 8 acceptance owns the eventual all-production source-check batch.

Native names are syntactic observations, without type/member resolution.
Automatic invocation, scope-aware extraction and same-language open-buffer
observations remain pending in the approved delivery set.
