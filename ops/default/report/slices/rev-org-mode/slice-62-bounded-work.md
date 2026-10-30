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

Sixteen actual analysis cancellations verify progress before cancellation,
Busy refusal, worker observation, source/history preservation and eventual return
to one thread, unchanged descriptors and zero children within 150 ms. The owned
release terminal measures Org idle, relative/paused timers, armed/paused countdown,
one-shot expiry, explicit stop, open/closed habit view and inactive text mode.
No timer state writes source. Each quiescent interval checks output, CPU, file
reads and process ownership; terminal restoration/reap/descriptor closure remains
mandatory.

Initial native controller receipt `JT537DVCUWIND2HHKPP27BLM6M` at
`caf38b6ea4f115a6c6008e918c6e84b3a214e5f8` passed the six workloads. Source
maximum was 129.305 ms, largest index chunk 624.601 us, cancellation maximum
37.860 us and process RSS 11608 KiB. Later checks must cover the added terminal
observations and final candidate. The first terminal fixture incorrectly selected graph preferences rather than
opening the habit view. The corrected call uses the shipped show-graph command
and C-g return. It also uses the existing agenda frame collector, which admits
a fully drawn hidden-cursor projection as well as a source cursor. This preserves the same closed-view observation and eight-second
PTY deadline. Source helpers were split by result ownership
to retain the existing complexity ceiling without exceptions.

Acceptance extends the existing manifest additively. Same-commit/tree identity,
raw counts, source chunk and cancellation timing, all ten idle states, memory,
read controls and actual cleanup are mandatory. Negative fixtures reject stale
identity, missing/duplicated/reordered work, changed ceilings, fabricated completion
counts, continuing CPU, timer repetition and leaked process resources. The earlier
thirteen native workloads, four PTY sessions and every baseline budget remain
unchanged. The first complete release performance stage measured 14.643 seconds excluding
build, including 5.260 seconds of additional Org measurements. The
approved complete-gate target remains three to six warm minutes.

The required implementation run passed all 522 selected native tests and the
complete historical/Org performance stage, then correctly rejected entrypoint
complexity 11. Argument parsing and workload dispatch are now separate owners;
no source-quality rule or resource limit changed. Measurements use Linux
7.2.2-arch1-1 on x86_64, Ryzen 5 7430U, 12 logical CPUs, 14.54 GiB RAM,
Rust 1.98.0, release profile and an 80 by 24 terminal. Raw receipts retain
native samples, CPU ticks, RSS, fixture fingerprints and exact commit/tree.
The full delivery gate was not run; the warm target above is not a measured
full-gate result.

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
