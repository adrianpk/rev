# Slice 32: expand and edit native Org capture drafts

Status: implementing
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-32-capture-drafts`
PR: pending

## Purpose

Deliver bounded, data-only expansion and an editable transient capture draft
under the [approved plan](../../../plan/rev-org-mode.md#slice-32-expand-and-edit-native-org-capture-drafts)
and [tracker](../../../tracker/rev-org-mode.md#slice-32-expand-and-edit-native-org-capture-drafts).

## Delivered Behavior

T32.1 expands declared selection, annotation/link, time, kill, user and clipboard
text substitutions. String prompts expose defaults and completions. A first
cursor marker records an offset in generated text. Inserted values remain opaque:
escape-looking source and answers are never rescanned or executed. Checked origin
links require an actual buffer point; unnamed origins do not invent a path.

## Implementation Notes

Admission rejects malformed/unsupported instructions before prompts or IO.
The engine allows 4096 tokens, 64 prompts, 256 KiB per supplied value and 1 MiB
output, with finite 64-token quanta and explicit cancellation. Time substitutions
share one captured civil date/time, render timestamps at whole-minute precision,
and retain seconds for explicitly requested formatting. Formatting admits
`Y m d e H M S F R T a A b h B u w j % n t`; unavailable zone directives reject.
Unavailable user/clipboard values require explicit text input rather than an
implicit executable host query. The original template remains unchanged.

## Contracts Added or Changed

- ORG10: bounded data-only capture expansion and checked origin links.
- ORG02: actual source identity remains separate from generated text offsets.

## Files of Interest

- `src/org/capture/expansion.rs`: bounded expansion and prompt ownership.
- `src/org/capture/expansion/`: instruction admission and civil formatting.
- `src/org/capture/origin.rs`: checked, escaped origin links.
- `tests/org_agenda/capture_expansion.rs`: independent output, failure and bounds assertions.

## Validation

- `cargo test --test org_agenda capture_expansion -- --nocapture`: five tests
  pass, zero failures or ignored tests.
- `cargo clippy --lib --test org_agenda -- -D warnings`: passed during T32.1;
  final clean-head controller checks follow all task and PR-reference commits.
- Changed-source quality passes without changing parameter, nesting or complexity limits.

## Risks and Follow-ups

T32.2 and T32.3 remain in progress. Publication remains the recorded Slice 33
responsibility; this report does not claim the complete capture lifecycle.
