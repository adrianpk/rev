# Slice 35: archive Org subtrees without silent loss

Status: drafting
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-35-archive`
PR: pending

## Purpose

Deliver native archive state, sibling placement and external moves under the
[approved plan](../../../plan/rev-org-mode.md#slice-35-archive-org-subtrees-without-silent-loss)
and [tracker](../../../tracker/rev-org-mode.md#slice-35-archive-org-subtrees-without-silent-loss).
The Org reference supplies [external context](https://orgmode.org/manual/Moving-subtrees.html)
and [internal archive semantics](https://orgmode.org/manual/Internal-archiving.html).

## Delivered Behavior

T35.1 delivers archive tag toggling, checked Archive sibling creation/reuse,
and external `%s_archive::` destinations. Local inherited ARCHIVE properties
and actual native ARCHIVE keywords override configuration. Exact unique target
headings can be created; ambiguous, malformed, unavailable and same-document
external destinations reject before canonical publication.

## Implementation Notes

Archive planning reuses bounded capture destination discovery/placement and
complete revision-qualified refile spans. The shared move plan distinguishes
actual agenda observations from selected existing/missing file observations;
no fragment is assigned a fabricated canonical identity. Both histories, live
points and viewports are admitted before either canonical text changes.

External metadata uses the existing checked property owner. One property drawer
retains IDs, planning and unrelated fields while adding origin file, outline
path, captured time, category, task state and inherited tags. Sibling moves keep
the original ancestor context. New separators follow the target format.

## Contracts Added or Changed

- ORG11: native archive commands, literal location policy and origin metadata.
- ORG09: archive source policy is completed by T35.2.

## Files of Interest

- `src/org/archive/`: bounded archive destination and metadata plans.
- `src/org/visibility/tags/archive.rs`: actual heading context and sibling identity.
- `src/org/refile/destination.rs`: existing/missing canonical move admission.
- `src/session/refile/archive.rs`: native command and single-worker ownership.
- `docs/reference/org-files.md`: user controls, formats, bounds and save recovery.

## Validation

- `cargo test --test org_io archive -- --nocapture`: three native workflows
  pass, including tag reversal, sibling undo, origin metadata, missing archives,
  ordinary independent saves, exact/duplicate headings and failed directories.
- `cargo clippy --lib --test org_io -- -D warnings`: passed.
- `python3 scripts/check-config-reference.py`: 151 schema/catalog leaves,
  54 RGB roles, 233 bindings and 18700 exact packaged bytes agree.
- Changed-source quality passes at unchanged parameter, nesting and complexity limits.

## Risks and Follow-ups

Archive moves change buffers without claiming a durable save. Ordinary Save,
undo and dirty-buffer close guards retain each participant independently.
Archive insertion uses the existing capture placement's 2 MiB payload bound;
structural and source discovery retain their existing limits. Full delivery-set
validation remains controller-owned.
