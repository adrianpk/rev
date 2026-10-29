# Slice 47: export Org to text Markdown and Org

Status: reviewing
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-47-text-exports`
PR: `#148`

## Purpose

Deliver native text, Markdown and Org export under the
[approved plan](../../../plan/rev-org-mode.md#slice-47-export-org-to-text-markdown-and-org)
and [tracker](../../../tracker/rev-org-mode.md#slice-47-export-org-to-text-markdown-and-org).

## Delivered Behavior

T47.1 renders checked snapshots as ASCII, UTF-8 text, Markdown or Org. Native
headings, metadata/options, lists, tables, links, footnotes and literal blocks
retain their meaning. ASCII rejects unrepresentable Unicode before publication.
Markdown chooses safe code fences and escapes literal paragraph markup. Org
retains the prepared source syntax. No renderer edits or saves canonical source.

T47.2 adds four compiled M-x commands with explicit scope, options and literal
local destination prompts. Existing files require observed replacement consent.
Canonical and disk inputs, live destination buffers and physical aliases are
checked before publication. Sources, view changes and cancellation retain their
ordinary owners; a late cancellation cannot hide a completed file effect.

T47.3 verifies handwritten document expectations, scope/options, source and peer
view preservation, replacement freshness, Unicode failures and finite rendering
budgets. RGB and NO_COLOR terminal journeys verify exact output bytes, ordinary
undo, declined replacement, cancellation, terminal restoration and owned process
cleanup. Known dynamic report bodies render their recorded data without refresh;
unknown bodies remain literal. All admitted metadata leaves retain output.

## Implementation Notes

Rendering uses the native structure and inline observations over immutable
projection text. Task keywords come from the existing native workflow owner.
Recorded report bodies share original snapshot policy and source evidence; their
derived offsets never become canonical source identity. The bounded file writer is shared with table interchange; the ordinary atomic publisher owns
staging, replacement, permissions and truthful completion outcomes.

Task commits: T47.1 `bdc974e583247ec26a039bea1bd5d3c1e8fbac18`, T47.2 `d21ad7715eacc14f323019991006df0163ac23f8`, T47.3 `1da01bb882a5306396853445fec375a26b2efcc6`.

Canonical PR: [#148](https://forge.adrianpk.com/adrianpk/rev/pulls/148).

## Contracts Added or Changed

- ORG25: native text, Markdown and Org rendering from checked snapshots.
- ORG27: table, link and footnote data retain explicit representation.

## Files of Interest

- `src/org/export/render.rs`: bounded native output traversal.
- `src/org/export/render/inline.rs`: markup, code, link and note conversion.
- `tests/org_io/text_exports.rs`: handwritten document expectations and native file workflows.
- `scripts/configuration_ui/org_exports.py`: owned terminal journeys.

## Validation

- `cargo test --test org_io text_export`: 13 renderer and file workflows passed, zero failures or ignored tests.
- `cargo clippy --lib --test org_io --test commands -- -D warnings`: passed during implementation.
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-exports`: two owned RGB/NO_COLOR sessions passed, zero skipped.
- Changed-source quality: passed with unchanged native limits.
- `cargo test --test commands`: the first final-candidate run detected a digit in the UTF-8 command identity and missing complete-catalog documentation. The identity is now `org-export-to-unicode`; both independent references include all four commands. Final evidence follows this correction.

Final acceptance runs through controller validation on the clean PR-reference
HEAD. The canonical PR and controller evidence retain that exact candidate.

## Risks and Follow-ups

None recorded.
