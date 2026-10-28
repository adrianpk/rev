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

C-c a and M-x org-agenda observe configured sources. Empty configuration asks
for current, buffers or a literal file/directory path. Current eligible Org
buffers can be selected without saving; cancelling the prompt scans nothing.
The result reports complete/incomplete status, retained documents/bytes and
source failures. Eight failures appear in the bounded echo; all admitted
failures remain in the read-only result. Explicit reinvocation refreshes.

Typed agenda sources select local files or directories. Directory discovery
includes immediate .org children only; no home, project or nested directory
is scanned implicitly. Empty selection performs no discovery. Paths resolve
relative to the configuration source that assigns the replacing array.

## Implementation Notes

The existing session Job slot owns source discovery. A request queues behind
existing color work, never adds a worker, and remains cancellable with C-g.
The initiating cursor/document does not own the multi-source result: every open
source stamp is rechecked separately before publishing. Editing and view motion
continue while IO runs; stale sources become explicit failures, with available
sources retained. An owned cancelled worker is reaped before another starts.
Finished discovery adds no inactive polling deadline.

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
- Development source quality found source matching above cyclomatic 10, first
  at 12 then 11. Disk ingestion, alias matching and argument dispatch now have
  separate owners. Changed-source quality passes at unchanged limits.
- `cargo test --test org_agenda --test org_io sources:: -- --nocapture`: eight
  native/filesystem cases passed, zero skips. A real 256-file observation read
  85906 bytes while normal input remained usable, about 39 ms on this host.
- `cargo test --test commands reference`: nine selected command/reference cases
  passed, zero skips.
- `cargo clippy --lib --test org_agenda --test org_io -- -D warnings`: passed.
- `python3 scripts/check-config-reference.py`: 141 entries, 54 RGB roles,
  192 bindings and 16759 packaged bytes agree after updating the editor map
  count for its actual new binding.

Development fixtures were corrected to use native history, prompt and ownership
APIs. Clippy removed an unnecessary owned comparison. No assertion, deadline or
resource limit was weakened.

## Risks and Follow-ups

Additional failure and boundary checks follow in T26.3.
The delivery-set full gate has not run.
