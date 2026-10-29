# Slice 28: query Org tasks tags and properties

Status: reviewing
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-28-agenda-queries`
PR: `#129`

## Purpose

Deliver bounded TODO, text and tag/property agenda queries under the
[approved plan](../../../plan/rev-org-mode.md#slice-28-query-org-tasks-tags-and-properties)
and [tracker](../../../tracker/rev-org-mode.md#slice-28-query-org-tasks-tags-and-properties).

## Delivered Behavior

Typed query plans select unfinished tasks, explicit task states, text phrases,
Boolean words/regexp terms and tag/property expressions. Query length and depth
have admitted typed limits. Parsing has no discovery or execution authority.

## Implementation Notes

The existing tag/property parser retains Boolean precedence, parentheses, TODO
suffixes, numeric/string comparisons, escaped names and missing-value rules.
Effort comparisons interpret native durations as minutes with the selected units;
quoted comparisons retain original text. Search phrases tolerate whitespace and
case changes. Leading + selects signed Boolean terms; braced regexps use a
finite Thompson automaton with at most 512 states and no backtracking.

Supported regexp forms include literals, dot, classes/ranges, anchors,
grouping/alternation and *, +, ? repetition; word edges and whitespace/word
escapes are finite native assertions. Unsupported escapes, counted repetition,
backreferences and malformed expressions reject. Search has at most 64 terms.
Query byte/depth defaults are 4096/32 and maxima 65536/128.

Behavior references: [search view](https://orgmode.org/manual/Search-view.html),
[TODO lists](https://orgmode.org/manual/Global-TODO-list.html), and
[tag/property matching](https://orgmode.org/manual/Matching-tags-and-properties.html).

Result observations use real unqualified structural facts and the existing
property resolver for local, file and inherited values. They retain canonical
buffer revisions or checked disk observations and original heading offsets.
Local workflows, priority declarations, file tags, selective inheritance,
additive properties and native durations feed the same typed match plan.
COMMENT/archive subtrees are excluded; malformed metadata and excess rows stay
explicitly incomplete. Text search covers each entry's section, not descendants.

Compiled TODO/search/match commands and local t/s/m controls publish read-only
rows in one owned pane. Invalid/cancelled drafts preserve the previous view;
changing query kind retains its selected sources. Refresh observes current
canonical text and time; checked Enter return reuses buffer revision checks or
bounded disk text/metadata equality. Peers remain editable, and owner deletion
retires the projection. Native prompts retain their visible cursor.

## Contracts Added or Changed

- ORG08: bounded query plans over explicitly selected canonical sources.
- ORG23: typed TODO/search/tag/property grammar without evaluated predicates.

## Files of Interest

- `src/org/query/`: Boolean matching, duration comparisons and finite text search.
- `src/org/agenda/query.rs`: typed agenda query plans.
- `tests/org_agenda/query_language.rs`: grammar, escaping and admission bounds.
- `src/session/agenda_queries/`: native controls and pane ownership.
- `tests/org_agenda/query_native.rs`: real refresh, return, cancel and peer panes.
- `docs/reference/org-agenda.md`: supported grammar, source scopes and bounds.

## Validation

T28.1: `82119810b9a5ddfd2e82311e3c4ac8a8206cce90`.
T28.2: `153010d778bd659d7db642e143bc324f292233ee`.
T28.3: `99c634d747f493ad9d0e8025c1f2678f7d7ce86f`.

- `cargo test --test org_agenda query_ -- --nocapture`: two parameterized cases
  passed with zero ignored tests.
- `cargo clippy --lib --test org_agenda -- -D warnings`: passed.
- Changed-source quality passes at unchanged parameter, nesting and complexity
  limits after splitting automaton transition and parser responsibilities.
- `python3 scripts/check-config-reference.py`: 146 schema entries, 54 RGB roles,
  201 bindings and 17289 packaged bytes agree.

Development checks exposed the missing duration interpretation, reference rows
and an over-complex transition function. They are corrected without relaxing
assertions or resource limits.

- `cargo test --test org_agenda query_ -- --nocapture`: four query cases passed.
- `cargo test --test org_editing propert`: fourteen existing property cases
  passed, zero ignored cases. One first exposed an incorrect hours assumption;
  effort comparisons now use duration minutes, preserving the original numeric
  scalar assertion. New clock-duration fixtures use a 120-minute threshold.
- Focused Clippy and changed-source quality pass without exceptions after
  separating per-document and per-heading observation.

- `cargo test --test org_agenda query_ -- --nocapture`: nine query cases passed,
  zero ignored cases, including native input, refresh, shared panes and stale
  source rejection with BOM/CRLF ingress.
- `cargo test --test org_agenda query_native::query_native_cancel -- --exact --nocapture`:
  one real owned cancellation case passed with unchanged input/source/history.
- `cargo clippy --lib --test org_agenda --test configuration -- -D warnings`:
  passed. Changed-source quality passes at unchanged limits.
- `python3 scripts/check-config-reference.py`: 146 entries, 54 RGB roles,
  204 bindings and 17357 packaged bytes agree.

Native checks first exposed the missing compiled name entries. They now include
all three actual query commands. A cancellation fixture initially spent its
five-second settle deadline coloring an already open million-byte line before
query launch. The same million-byte source is now an unopened selected file,
so it exercises query ownership/cancellation without changing its source size,
assertions or deadline.

The first final focused batch found named/direct query prompt echo divergence.
The prompt now writes the same explicit feedback through both paths; the
unchanged catalogue dispatch assertion passes. Additional negative escape cases
reproduced case folding turning unsupported regexp \W/\S into supported escapes.
Folding now preserves escape syntax, and unknown quoted search escapes reject.
`cargo test --test org_agenda query_language::query_search_language -- --exact --nocapture`
and `cargo test --test commands palette::catalog_dispatch -- --exact` pass.
Changed-source quality remains green without exceptions.

Final controller evidence binds the clean canonical PR head. Required order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_agenda --test org_source`
- `python3 scripts/check-source.py`
- `git diff --check`

## Risks and Follow-ups
The delivery-set full gate has not run.

[Canonical PR #129](https://forge.adrianpk.com/adrianpk/rev/pulls/129) targets dev.
