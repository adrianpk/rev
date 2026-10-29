# Slice 29: configure native Org agenda views

Status: drafting
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-29-agenda-customization`
PR: pending

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

- `cargo test --test org_agenda --test configuration custom_ -- --nocapture`:
  three cases passed, zero failures or ignored tests.
- Development checks found the aligned-week/day-block date error; block spans
  now use the retained requested date. The original assertions pass.
- Changed-source quality passes at unchanged limits. The exhaustive reference agrees on 147 entries, 54 RGB roles and 204 bindings.
- `cargo test --test org_agenda custom_native -- --nocapture`: two native
  workflows passed, zero failures or ignored tests.
- `cargo test --test commands palette::catalog_dispatch -- --exact`: all named
  and direct compiled command handlers agree.
- The exhaustive reference now agrees on 147 entries, 54 RGB roles, 212 bindings
  and 17693 exact packaged bytes. Changed-source quality passes without exceptions.
- Final exact-head evidence is pending the remaining native controls and tests.

## Risks and Follow-ups

T29.3 remains active. The delivery-set full gate has not run.
