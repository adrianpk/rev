# Slice 3: Typed pairing and buffer toggle

Status: drafting
Delivery set: rev-delimiter-editing
Plan: [Automatic Delimiter Editing Delivery](../../../plan/rev-delimiter-editing.md)
Tracker: [Automatic Delimiter Editing Tracker](../../../tracker/rev-delimiter-editing.md)
Branch: `feat/rev-delimiter-editing-03-typing`
PR: pending

## Purpose

Apply admitted delimiter typing through native source/history publication and
expose the shared buffer toggle without new factory bindings.

## Delivered Behavior

Eligible non-modal and modal Insert/Replace input inserts structural mates,
reuses existing unmatched closers and overtypes confirmed closers. Quote admission
uses the recognized language and conservative source boundary. Unknown, malformed
or excluded context keeps ordinary literal input. Paste remains literal.

Positive counts prepare the complete effect once. Structural input nests and
shares one inspection budget; symmetric quotes consume an admitted mate before
further adjacent input stays literal. Zero is inert and negative input retains
refusal. A limited counted request publishes no source, point or history fragment.
Replace overwrites only physical input characters, and native grapheme boundaries
remain authoritative. Pure closer skipping creates no source/history operation.

M-x auto-pair-mode toggles one canonical buffer shared by all panes and survives
language changes without writing configuration. Exactly this one unbound command
is added; the 707 original commands and all reviewed factory maps remain intact.

## Implementation Notes

- The existing buffer qualifier owns source inspection, strict balance, language
  classification, cumulative budgets and native proof. Its typing request also
  accounts for projected nesting and a native Replace range. Virtual suffix work
  never becomes retained proof about overwritten source. Checked prefix snapshots
  predate the typing point; skipped quote facts still retain their original context.
- Session preparation borrows one ticket and captures point, view, mark, mode
  and resolved policy. It uses the native bounded layout/input FIFO and one poll
  at a time. Cancellation and stale preparation retire the ticket. There is no
  new timer, idle worker, source clone or per-view proof cache.
- All edits use native admission and shared-view publication with an explicit
  initiating cursor. Generated mates are source effects. Modal insertion capture
  retains an admitted skip, so semantic repeat replays accepted source once.
- Counted physical text carries its origin separately from accepted literal paste.
  Quote-count preparation follows repeated admitted typing, including disabled
  skipping and an existing quote. Expansion is preflighted before publication.

## Contracts Added or Changed

- One positive scalar/count typing query returns a qualified opening/reuse or
  closer-skip decision; all existing uncertain/limited outcomes remain explicit.
- The fixed 256 KiB inspection, depth 256 and 2 MiB qualified metadata bounds
  remain unchanged. Polls retain the 256-step and four-millisecond target.
- Native prefix limits and the 8 MiB generated-text ceiling remain unchanged.
  Pair admission neither resets work per generated mate nor publishes partial
  counted effects. Pending source edits retain the native cancellation queue.
- auto-pair-mode has a catalog identity, named handler, description and lookup.
  Independent additive inventory expectations preserve old names and exact maps.

## Files of Interest

- `src/pairs/work/typing.rs`: admitted context, local balance and bounded typed proof.
- `src/session/pairs.rs`, `src/session/pairs/plan.rs`: capture, admission and native publication.
- `src/session/layout.rs`, `src/session/repeat/text.rs`: existing queue and modal lifetime.
- `src/input/argument.rs`, `src/command.rs`, `src/buffer/policy.rs`: provenance, command and shared toggle.
- `tests/pairs/typing.rs`, `tests/pairs/typing/lifecycle.rs`: independent native matrices and ownership regressions.
- `examples/factory_shortcuts_acceptance/matrix.rs`: unchanged accepted maps plus one explicit added identity.
- `docs/reference/delimiter-editing.md`: reader-facing policy and command metadata.

## Validation

Validation is in progress. Initial 8-case typing checks passed, followed by a
15-case lifecycle batch. Final evidence will cover the complete selected typing
suite, changed shared qualification, command references and the existing native
factory-map helper. Full staged-source hooks remain installed and authoritative.
Personal configuration and frozen factory catalog/manifest bytes are preserved.
The exact three-command clean-HEAD controller receipt follows PR-reference commits.
The combined delivery-set gate remains controller-owned.

## Risks and Follow-ups

Unproved raw/triple/template and embedded literal contexts remain conservative.
Paired deletion, selection wrapping, explicit helpers, complete literal/macro
provenance and physical-input acceptance belong to their recorded later slices.
The new checks are native; they do not claim PTY or completed-frame acceptance.
