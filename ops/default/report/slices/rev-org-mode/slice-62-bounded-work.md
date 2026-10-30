# Slice 62: verify bounded Org work and idle resources

Status: reviewing
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `perf/rev-org-62-bounded-work`
PR: `#163`

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

A subsequent required run exposed the same unsynchronized descendant-state
observation in the feed lifetime fixture after session drop. The existing execution
fixture's stopped-state observer is now shared with feeds. It still requires a
missing or zombie descendant within the same 150 ms response budget; direct-leader
reap, private HOME cleanup, unchanged source, unpublished inbox and no idle polling
remain mandatory. Kernel state is observed with a finite predicate loop rather
than inferred from leader reap. No process runtime or acceptance budget changed.

Canonical PR: [#163](https://forge.adrianpk.com/adrianpk/rev/pulls/163), targeting `dev`.

## Contracts Added or Changed

- ORG18: actual bounded source/projection/job observations.
- ORG31: release measurements extend existing collection and preserve earlier ceilings.

## Files of Interest

- `examples/configuration_performance/org.rs`: native resource observations.
- `scripts/configuration_ui/org_performance.py`: isolated fixtures and frozen Org budgets.
- `scripts/configuration_ui/performance.py`: additive collection through the existing stage.
- `scripts/configuration_ui/org_resource_terminal.py`: real release timer/view measurements.
- `scripts/configuration_ui/evidence.py`: same-candidate baseline and Org admission.
- `tests/org_io/lifetime.rs`: shared bounded descendant-state observation.

## Validation

Required ordered checks passed on clean implementation candidate
`650d140a0b3654d15f3246cda7ef047acabbd8c2`, controller receipt
`47AMM4CQMPHRLKWXKUPS66ESVB`:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_source --test org_time --test org_agenda --test org_tables --test org_io`
- `python3 scripts/check-configuration-ui.py --suite performance`
- `python3 scripts/check-source.py`
- `git diff --check`

All 522 selected native tests passed: 81 source, 79 time, 97 agenda, 73 tables
and 192 IO, with zero failures, ignored or filtered tests. Source quality retains
safe Rust, five parameters, nesting three and cyclomatic ten. The actual complete
performance log was admitted for the same commit/tree. All thirteen historical
native workloads and four historical PTY sessions remain admitted, plus six Org
workloads and one owned Org PTY session, with zero skips.

| Org workload | Maximum sample, ms | Peak RSS, KiB | Retained resident growth, KiB |
| --- | ---: | ---: | ---: |
| 1 MiB / 10000 headings | 134.440 | 11200 | 3828 |
| 256-source agenda | 8.441 | 11200 | 776 |
| 733-day habit window | 0.009 | 11200 | 48 |
| 10000-cell dependency chain | 6.906 | 11200 | 880 |
| Eleven native exports | 2.703 | 11200 | 176 |
| Sixteen cancellation cycles | 0.037 | 11200 | 208 |

The largest source chunk was 669.302 us, below 10 ms. Native observations saw
775 actual opens and 778 accesses. Terminal responses peaked at 12.174 ms. All
ten idle states observed zero output, zero measured CPU and no filesystem reads,
with one thread, six descriptors and no children. Countdown expiry occurred once.
Owned restoration, leader reap and descriptor closure passed. The complete warm
performance stage took 14.658 seconds excluding build, including 5.294 seconds for
Org. Existing baseline ceilings and the Org 150 ms / 96 MiB / 32 MiB budgets
remain unchanged. These release observations are separate from debug native tests
and are not a full-gate result.

Focused controller receipt `UBMRES45DDOSP5JS5ABV4ZYUOG` passed on the same candidate:

- `cargo test --test org_io lifetime -- --nocapture`
- `python3 scripts/test_acceptance.py`
- `git diff --check`

Both lifetime tests passed all seven execution/feed modes, observing descendant
stop in 5.850..8.436 us; the other 190 IO tests were intentionally filtered in
that focused command. All 20 evidence-checker tests passed. The full required
command above ran all 192 IO tests without filtering. Earlier focused receipt
`DMW7INDQCRPFXLDAY5E5TG7GR5` at `768fa497cb624a28a233b630d51f90e27f80c832`
verified the entrypoint source-quality correction and all 20 checker tests.

Task commits: T62.1 `caf38b6ea4f1`, T62.2 `6d97dc26eadc`, T62.3 `b2f41d52875a`.
Follow-up fixture and dispatch corrections preserve the approved observable
assertions and resource ceilings. Final readiness requires fresh required and
focused evidence on the clean PR-reference candidate; the controller owns merge
and the unchanged full delivery gate.

## Risks and Follow-ups

Resident memory measurements depend on the recorded operating system and build
profile. They do not establish kernel IO latency bounds.
