# Slice 62: verify bounded Org work and idle resources

Status: drafting
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `perf/rev-org-62-bounded-work`
PR: pending

## Purpose

Measure ORG18 and ORG31 with the existing release performance collector.
[Plan](../../../plan/rev-org-mode.md) and [tracker](../../../tracker/rev-org-mode.md)
retain the approved resource profile and task boundaries.

## Delivered Behavior

The additive native workloads observe a 1 MiB, 10000-heading source, a real
256-file agenda, a maximum 733-day habit window, a 10000-cell dependency chain
and all eleven native export backends. Independent expected counts, numeric
results, actual completion marks and format signatures precede passing records.

## Implementation Notes

Native observations use the existing configuration-performance executable,
flushed setup barrier, release build, CPU ticks and resident high-water mark.
Org fixtures own HOME, every XDG root and cwd; source fingerprints must remain
unchanged. The 96 MiB process RSS and 32 MiB additional resident-growth budgets
are separate from the unchanged historical 128 MiB ceiling. Resident growth is
measured while results are retained and includes allocator retention; it is not
an exact count of logical heap allocation.

## Contracts Added or Changed

- ORG18: actual bounded source/projection/job observations.
- ORG31: release measurements extend existing collection and preserve earlier ceilings.

## Files of Interest

- `examples/configuration_performance/org.rs`: native resource observations.
- `scripts/configuration_ui/org_performance.py`: isolated fixtures and frozen Org budgets.
- `scripts/configuration_ui/performance.py`: additive collection through the existing stage.

## Validation

Implementation checks are in progress. The approved required command order is
unchanged. No full delivery gate has run in this slice.

## Risks and Follow-ups

Resident memory measurements depend on the recorded operating system and build
profile. They do not establish kernel IO latency bounds.
