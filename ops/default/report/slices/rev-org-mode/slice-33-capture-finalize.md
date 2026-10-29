# Slice 33: finalize or abort Org capture safely

Status: reviewing
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-33-capture-finalize`
PR: [#134](https://forge.adrianpk.com/adrianpk/rev/pulls/134)

## Purpose

Deliver checked native capture publication under the
[approved plan](../../../plan/rev-org-mode.md#slice-33-finalize-or-abort-org-capture-safely)
and [tracker](../../../tracker/rev-org-mode.md#slice-33-finalize-or-abort-org-capture-safely).

## Delivered Behavior

T33.1 revalidates the retained destination's actual path, version, BOM and text,
then prepares bounded entry, plain, list, table or missing-heading insertion.
Entries retain relative heading levels, file preambles and target metadata.
Ordered lists reuse native counter/reset rules and separate source marker edits
from inserted text, preserving ordinary shared-point mapping. Missing date-tree
headings retain the captured date and chronological insertion anchor.

## Implementation Notes

The ordinary buffer save owner can publish a prepared edit after actual file
replacement. It reserves history/point admission before IO. Failed replacement
leaves canonical source, history and points unchanged; successful replacement
publishes one ordinary edit. Directory-sync/observation failures retain actual
publication and the ordinary unresolved-save warning. Future candidate text is
an immutable prepared rope, never a fabricated canonical buffer identity.

T33.2 exposes working `C-c C-c` / `org-capture-finalize`. Validation uses the
existing job slot; C-g restores the draft. The live source is checked again
before the ordinary saved-edit boundary. Complete save returns home; rejected
publication keeps the draft and source unchanged. A post-replacement failure
retains an explicit inserted-once receipt, canonical source and draft; retry
saves that pinned source without another insertion. Changed draft/source pins
reject automatic retry, and abort reports actual prior publication.

T33.3 verifies actual file, outline, list, table and date-tree publication,
shared-pane positions, metadata placement and draft bounds. Plain prepend skips
actual heading metadata and inserts a separator before existing body text.
One shared unit-test fixture owner and module-qualified temporary paths prevent
collisions without changing assertions or resource limits.

## Contracts Added or Changed

- ORG10: exact destination revalidation and coherent capture insertion planning.
- ORG18: ordinary save checks, read-only state, history and publication truth.

## Files of Interest

- `src/org/capture/finalize.rs`: retained evidence, bounded work and checked edits.
- `src/org/capture/finalize/`: native placement and heading normalization.
- `src/org/list/numbering.rs`: shared counter/reset behavior for capture.
- `src/buffer/save.rs`: pre-admitted edit and ordinary file publication.
- `tests/files/capture_saved.rs`: real replacement, rejection and unresolved completion.
- `src/session/capture/finalize/`: draft ownership, publication and pinned retry.
- `tests/org_agenda/capture_shapes.rs`: saved shapes and shared-pane mapping.
- `scripts/configuration_ui/org_capture_finalize.py`: owned terminal save/rejection.

## Validation

T33.1: `0cdd4a33a0f358f4de544889e761878099d618f8` (report introduction).

- `cargo test --lib finalize_ -- --nocapture`: two tests cover six placement
  scenarios and chronological date-tree planning, with unchanged disk bytes.
- `cargo test --test files capture_saved -- --nocapture`: one file-backed test
  covers complete, permission-rejected and completion-unresolved publication.
- `cargo clippy --lib --test files -- -D warnings`: passed.
- Changed-source quality passes at unchanged parameter, nesting and complexity limits.

T33.2: `7396cdeaa1fa73652e9baa5d08ab06b560883ede` (native finalize lifecycle and commands).

- `cargo test --test org_agenda capture_publication -- --nocapture`: four
  file-backed session workflows pass, zero failures or ignored tests.
- `cargo clippy --lib --test org_agenda -- -D warnings`: passed.
- Changed-source quality remains green without changing its limits.

- `cargo test --test commands --test configuration --test org_editing`: 478
  tests pass, zero failures or ignored tests, including actual finalize command
  invocation and the retained source/role regressions.
- `python3 scripts/check-config-reference.py`: 149 leaves, 54 RGB roles, 228
  default bindings and 18370 exact packaged reference bytes agree.

T33.3: `85a4a9279e7c4a67a253c0444e0062fed5dfcfbd`.

T33.3 focused checks:

- `cargo test --lib finalize -- --nocapture`: four tests cover bounded drafts,
  actual date-tree save, native placement and three unresolved-save retry cases.
- `cargo test --test org_agenda capture_shapes -- --nocapture`: two tests cover
  four saved capture kinds and retained peer positions during list renumbering.
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-capture-finalize`:
  four owned RGB/NO_COLOR save/read-only sessions pass, zero skipped.
- `cargo clippy --lib --test org_agenda -- -D warnings`: passed.
- Changed-source quality passes with unchanged limits.

A plain-prepend fixture initially exposed metadata joining; placement now keeps
its property drawer intact and the exact expected bytes pass. Concurrent unit
fixtures initially collided, and required all-target Clippy rejected duplicate
module loads. A single shared fixture owner with module-qualified paths fixes
that setup while preserving all observable assertions. The required file suite also exposed an older accepted-ingress assertion that
expected unbound `C-c`. It now asserts the actual fresh global capture prefix,
with unchanged file bytes and the same quit-guard checks, still proving that
loading clears the previous `C-x` prefix. Final controller receipts bind the
clean pushed PR head after its reference follow-up commit.

## Risks and Follow-ups

An unresolved save retains actual published source and its draft; abort does
not undo an already completed replacement. Changed source/draft pins require
explicit inspection rather than automatic reinsertion. The delivery-set full
gate remains controller-owned.
