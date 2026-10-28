# Slice 3: Recognize inline Org source forms

Status: delivered
Delivery set: rev-org-mode
Plan: [Native Org Delivery](../../../plan/rev-org-mode.md)
Tracker: [Native Org Tracker](../../../tracker/rev-org-mode.md)
Branch: `feat/rev-org-03-inline-syntax`
PR: [#104](https://forge.adrianpk.com/adrianpk/rev/pulls/104)
Integrated: `6f286b086857056fe3c09e9eb0aca2eec3586ddd`

## Purpose

Recognize native inline Org forms with exact, revision-qualified source ranges
and explicit enclosing contexts.

## Delivered Behavior

Native observations recognize bold, italic, underline, strike, code, verbatim,
standard entities, bracket/angle/plain links, dedicated/radio targets and
footnote references or inline definitions. Each result retains whole-form and
content ranges, optional description/definition ranges and parent relationships.
Original spelling, escapes and separators remain canonical.

Nested recognition respects standard and minimal object contexts. Link targets,
code, verbatim and selected literal regions remain opaque. Descriptions and radio
targets admit minimal objects; inline footnotes admit standard objects. Invalid
outer link syntax can recover a valid inner form without normalizing its source.

Results reject foreign or stale source access. Inline analysis uses the shared
observation worker and immutable rope snapshot contracts; cancellation and finite
byte/node/depth limits report their actual coverage state. Clipped tokens or
interior UTF-8 endpoints cannot become valid source locations.

## Implementation Notes

Private recognizers share a source reader and an iterative context stack. Source
reads check cancellation and yield after 10ms of native work. Failed searches
retain the exhausted prefix for their context; angle/target checks reject invalid
prefixes early. This protects recovery across rope chunks with repeated unmatched
openers while retaining later valid forms.

Default emphasis permits one internal newline. Recovery fixtures use a paragraph
boundary when distinguishing an unmatched opener from a later independent form.
The initial fixture assumed that one newline ended emphasis; its correction
preserves the reference rule and all recovery assertions.

Behavior was checked against the [Org syntax reference](https://orgmode.org/worg/org-syntax.html)
and clean Emacs 31.1 / Org 9.8.7 observations. Entity names are standard syntax
data; recognizers are in-house Rust. Native validation uses independent literal
expectations and runs without the reference runtime.

The structural owner supplies each inline region and its context. Slice 4 owns
heading, drawer, table and block recognition, including actual literal regions.

## Contracts Added or Changed

- ORG02: preserve unknown or malformed text, exact UTF-8 ranges and canonical
  file fidelity; reject obsolete observations before access.
- ORG15: retain link targets/descriptions and target source without following them.
- ORG16: distinguish markup from code, verbatim and literal contexts.
- ORG30: retain source coordinates for later semantic presentation and previews.

## Files of Interest

- `src/org/inline.rs`: typed observations, source qualification and owned analysis.
- `src/org/inline/engine.rs`: bounded context stack and parent relationships.
- `src/org/inline/read.rs`: checked source reads, cancellation and yielding.
- `src/org/inline/markup.rs`, `bracket.rs`, `link.rs` and `entity.rs`: native forms
  and delimiter recovery.
- `tests/org_source/inline.rs` and `inline_bounds.rs`: ten tests covering source
  families, nested contexts, Unicode, limits, recovery, stale workers and fidelity.

## Validation

Implementation checks passed:

- `cargo check --lib`
- `cargo clippy --test org_source -- -D warnings`
- `cargo test --test org_source`: 32 passed, zero failed or ignored.
- `git diff --check`

Focused source-quality checks passed for all new recognizers and tests.
Controller receipt `WVGRL3W5TZU3RFM6UXYBPFTHXQ` passed this exact ordered
batch at clean head `6f286b086857056fe3c09e9eb0aca2eec3586ddd`, subsequently
integrated through PR #104. All 56 tests passed with zero skips:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_source --test coordinates`
- `python3 scripts/check-source.py`
- `git diff --check`

## Risks and Follow-ups

None recorded.
