# Slice 26: discover bounded canonical Org agenda sources

Status: drafting
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-26-agenda-sources`
PR: pending

## Purpose

Deliver explicit bounded source discovery under the
[approved plan](../../../plan/rev-org-mode.md#slice-26-discover-bounded-canonical-org-agenda-sources)
and [tracker](../../../tracker/rev-org-mode.md#slice-26-discover-bounded-canonical-org-agenda-sources).

## Delivered Behavior

Typed agenda sources select local files or directories. Directory discovery
includes immediate .org children only; no home, project or nested directory
is scanned implicitly. Empty selection performs no discovery. Paths resolve
relative to the configuration source that assigns the replacing array.

## Implementation Notes

Immutable rope snapshots preserve current buffer text without saving. Real file
identity deduplicates symlink and hard-link aliases. Conflicting live buffers
for one inode are diagnosed instead of choosing an arbitrary owner. Disk ingress
retains strict UTF-8, NUL rejection, BOM metadata and original line endings;
whole files publish only after matching before/open/after observations.

The typed budgets are 512 files, 8 MiB per source, 64 MiB total and 30 seconds.
Maximum values are 4096, 32 MiB, 256 MiB and 600 seconds. A streaming directory
walk admits at most 65536 entries, including ignored names. At most 4096 roots,
4096-byte path declarations, 4096 open-buffer snapshots and 4096 diagnostics
are retained. Each read chunk is at most 8192 bytes, with one over-limit probe.
Failed reads consume the total budget. Cancellation/deadline checks occur at
iteration/read boundaries; they cannot interrupt a blocked kernel operation.

## Contracts Added or Changed

- ORG08: explicit canonical observations and truthful partial source sets.
- ORG18: finite discovery, bytes, time, ownership and cancellation.

## Files of Interest

- `src/org/agenda/sources/`: discovery, checked reads and retained provenance.
- `tests/org_io/agenda_sources.rs`: real filesystem and buffer boundaries.
- `tests/configuration/org_agenda.rs`: source layering and bounded admission.

## Validation

- `cargo check`: passed.
- `cargo test --test org_io --test configuration agenda_`: four focused cases
  passed, zero failures or ignored cases.
- `python3 scripts/check-config-reference.py`: 141 schema entries, 54 RGB
  roles, 191 bindings and 16736 packaged bytes agree.
- `cargo clippy --lib -- -D warnings`: passed.
- Changed-source quality passed after separating checked disk ingestion from
  source matching to preserve the original complexity limit.

## Risks and Follow-ups

Native session invocation and responsiveness checks follow in T26.2/T26.3.
The delivery-set full gate has not run.
