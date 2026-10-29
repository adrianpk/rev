# Slice 29: configure native Org agenda views

Status: delivered
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-29-agenda-customization`
PR: `#130`

## Purpose

Deliver typed native agenda customization under the [approved plan](../../../plan/rev-org-mode.md#slice-29-configure-native-org-agenda-views)
and [tracker](../../../tracker/rev-org-mode.md#slice-29-configure-native-org-agenda-views).

## Delivered Behavior

T29.1 adds typed view/block records, query/filter intersections, source subsets,
property columns, stable sorting, and day/week/month/year/custom date spans.
Records are data only and reject unknown keys and kinds. Configuration arrays
replace by leaf with retained declaring-source provenance.

## Implementation Notes

The existing bounded source observer supplies COW canonical snapshots. Selected
path memberships survive inode deduplication, so block scopes do not add scans.
Blocks share the job deadline and global row budget; columns retain at most
8 MiB of property text. Calendar blocks retain original heading locations;
clock blocks use native recorded-clock instant resolution and civil clipping.
Requested dates remain separate from aligned week/month/year starts.

T29.2 adds compiled selection, filter/reset, sorting, columns and span controls,
with matching local bindings. Presentation jobs use retained COW observations;
only explicit refresh reads sources again. Preset filters survive refresh and
runtime reset. Empty blocks remain visible in the block summary. Checked source
return shares the canonical query navigation owner; prompts retain their cursor.

T29.3 verifies exact/over limits and 32-block composition, native query kinds,
clock clipping/open counts, canonical file/directory aliases, default/explicit
source scope, retained filtering versus fresh reads, source-return rejection,
Unicode pane clipping, owner deletion, no-color columns and cooperative cancel.
All blocks have scrollable section headers, including empty blocks.

The pane case exposed stale feedback clipped behind the date header. Staleness
now precedes dates. Directory aliases now traverse their selected spelling for
membership while sharing the unchanged global entry budget; content reads and
retained source identities stay deduplicated. No assertions or limits weakened.

## Contracts Added or Changed

- ORG23: typed blocks, bounded filters/sorts and observed property columns.
- ORG17: admitted typed settings and exhaustive sample defaults.

## Files of Interest

- `src/config/agenda.rs`: typed block records and admission bounds.
- `src/org/agenda/custom.rs`: canonical block observations.
- `src/org/agenda/calendar.rs`: declared civil spans.
- `tests/org_agenda/custom_views.rs`: canonical projections and date boundaries.
- `docs/reference/org-agenda.md`: fields, defaults and native grammar.

## Validation

T29.1: `d6626fc1a2d11f1a5f1ca52928eaa8e0e2012465` (report introduction).
T29.2: `0cf592cb0ab313eea7eb293ee04e9f41a29781d1`.
T29.3: `8fd93c685e4661a958c9fa0ce4636eb4a3dc13ff`.


- `cargo test --test org_agenda --test configuration custom_ -- --nocapture`:
  three cases passed, zero failures or ignored tests.
- Development checks found the aligned-week/day-block date error; block spans
  now use the retained requested date. The original assertions pass.
- Changed-source quality passes at unchanged limits. The exhaustive reference agrees on 147 entries, 54 RGB roles and 204 bindings.
- `cargo test --test org_agenda custom_ -- --nocapture`: thirteen custom cases passed after the boundary checks.
- `cargo test --test org_agenda habit_views::habit_no_color -- --exact --nocapture`: passed.
- `cargo test --test org_agenda custom_native -- --nocapture`: two native
  workflows passed, zero failures or ignored tests.
- `cargo test --test commands palette::catalog_dispatch -- --exact`: all named
  and direct compiled command handlers agree.
- The exhaustive reference now agrees on 147 entries, 54 RGB roles, 212 bindings
  and 17693 exact packaged bytes. Changed-source quality passes without exceptions.
- Final required controller order binds the clean pushed PR head:
  `cargo fmt --all -- --check`;
  `cargo clippy --all-targets --all-features -- -D warnings`;
  `cargo test --test org_agenda --test configuration --test display`;
  `python3 scripts/check-source.py`;
  `git diff --check`.
- The first final focused batch caught the agenda context reference missing
  the eight new binding rows. The actual context table now includes every
  compiled target without changing the catalogue assertion.
- Focused final checks cover commands, source observations, IO, property editing,
  pane ownership and the exhaustive configuration reference through the controller.

## Risks and Follow-ups

The delivery-set full gate has not run.

[Canonical PR #130](https://forge.adrianpk.com/adrianpk/rev/pulls/130) targets dev. All task commits and this PR-reference follow-up are pushed before exact-head controller validation.

PR #130 merged into dev at `99125c9f9cc7c0ce774e7aa8b97556e30e225aaa`, verified on Forgejo and origin/dev. Required receipt `CF2NCOJATDVH4V6E7ITQJTNETK` passed 207 tests; focused receipt `RVS434VRIFBRQQK3UOFBGJ6CTL` passed 472 tests and the configuration reference. Zero failures or ignored tests.
