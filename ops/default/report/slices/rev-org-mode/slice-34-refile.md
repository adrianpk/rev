# Slice 34: refile Org subtrees across canonical documents

Status: drafting
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-34-refile`
PR: pending

## Purpose

Deliver checked complete-subtree refile/copy under the
[approved plan](../../../plan/rev-org-mode.md#slice-34-refile-org-subtrees-across-canonical-documents)
and [tracker](../../../tracker/rev-org-mode.md#slice-34-refile-org-subtrees-across-canonical-documents).
The [Org reference](https://orgmode.org/manual/Refile-and-Copy.html) supplies
complete-subtree destination selection and child placement semantics.

## Delivered Behavior

T34.1 prepares revision-qualified complete sibling subtrees and bounded target
headings. Plans preserve relative levels, metadata and literal body source;
new separators follow the destination format. Self/descendant targets and stale
same-source observations reject before edit qualification. Same-buffer boundary
moves use one replacement; cross-buffer move/copy keeps each actual owner.

## Implementation Notes

The source uses the existing complete structural model and native checked spans.
Target observations use the existing bounded agenda/source scan; BOM evidence
stays with its immutable source snapshot. Shared structural normalization serves
capture and refile. Plans qualify ordinary replacements without publishing or
writing files. No synthetic canonical buffer identity represents a fragment.

T34.2 provides `C-c C-w`, `C-c M-w` and `org-refile-results`, with typed
`org.refile.targets` and bounded explicit file/line/heading selection. The
existing worker owns discovery and normalization. Refile and copy reserve every
participating history, source point and viewport before changing either text.
The shared source-batch helper also retains existing clock publication behavior.

Refiling changes buffers without saving. Ordinary file-save outcomes update
per-participant results; partial saves retain canonical text and distinguish
rejection from actual replacement with unresolved completion. Later edits or
closure invalidate old result pins. No retry repeats the move implicitly.

## Contracts Added or Changed

- ORG11: checked complete subtree and destination plans.
- ORG02: canonical revision, ordinary history and source-format boundaries.

## Files of Interest

- `src/org/refile/`: bounded target observations and checked move/copy plans.
- `src/org/visibility/subtree.rs`: complete native subtree selection.
- `src/org/structure/entry.rs`: shared heading-level normalization.
- `src/org/agenda/sources/scan.rs`: retained source BOM evidence.

## Validation

- `cargo test --lib refile -- --nocapture`: two owned-file tests pass,
  including cross-buffer move/copy, literal source, BOM/CRLF, same-buffer
  boundary placement, descendant rejection and stale observations.
- `cargo clippy --lib -- -D warnings`: passed.
- Changed-source quality passes with unchanged parameter, nesting and complexity limits.

The literal block fixture uses the native comma-quoted heading syntax, retaining
its expected bytes during movement. Source and destination disk bytes remain
unchanged while planning and publishing the tested in-memory edits.

T34.1/report introduction: `f907554f0469a85cb6663b4641221d9160868da4`.

T34.2 checks:

- `cargo test --test org_io refile -- --nocapture`: two native owned-file
  workflows pass, including move/copy, cancellation, independent saves and undo.
- `cargo test --test configuration refile_policy -- --nocapture`: typed target
  paths, finite bounds, leaf reset and non-writing startup pass.
- `cargo test --test commands selected_arguments -- --nocapture` and
  `cargo test --test commands catalog_reference -- --nocapture`: passed after
  adding the implemented names and selecting the actual write-file row while
  retaining its held-key/save assertions.
- `python3 scripts/check-config-reference.py`: 150 schema/catalog leaves,
  54 RGB roles, 230 bindings and 18535 exact packaged reference bytes agree.
- `cargo clippy --lib --test org_io -- -D warnings`: passed.
- Changed-source quality passes at unchanged limits.

## Risks and Follow-ups

T34.3 failure/terminal evidence remains in progress. The delivery-set gate
remains controller-owned.
