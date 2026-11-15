# Slice 7: Integrated acceptance and guidance

Status: delivered
Delivery set: rev-delimiter-editing
Plan: [Automatic Delimiter Editing Delivery](../../../plan/rev-delimiter-editing.md)
Tracker: [Automatic Delimiter Editing Tracker](../../../tracker/rev-delimiter-editing.md)
Branch: `test/rev-delimiter-editing-07-acceptance`
PR: [#61](https://forge.adrianpk.com/rev/rev/pulls/61)

## Purpose

Provide reproducible integrated acceptance and final user guidance for automatic
delimiter editing. Prepare editing-only closure evidence while retaining the
controller's integration and combined-gate boundary.

## Delivered Behavior

The acceptance entrypoint batches the complete native pair suite and focused
configuration, edit/region/count, paste, macro record/replay, modal pair-object
and Org/Markdown projection/continuity regressions. Every declared filter must
select at least one test. The existing native factory helper verifies all 28
accepted maps and the independent 707-command baseline plus three pair commands;
historical PTY gates are not chained.

Two Linux PTY journeys use actual non-modal and modal Insert input, generated
mates and closer skipping, literal bracketed paste, raw-key/literal macro replay,
one-step replay undo/redo, save/reopen and prompt-focus separation. They compare
exact saved/reopened bytes and unchanged configuration. Both use the existing
terminal owner, eight-second child deadline, 1 MiB output cap, Linux idle checks,
restored termios/modes and owned child reaping. Native cases additionally verify
read-only insertion, skip and Backspace refusal in both profiles.

The editing lesson demonstrates empty-pair deletion/undo, selection wrapping,
explicit wrap/unwrap and counts. A how-to explains buffer controls, independent
preferences and singular per-language inheritance. The reference retains exact
command descriptions, context/literal policies and fixed resource limits. One
Unreleased entry describes the complete user capability.

## Implementation Notes

Task/report introduction commit: `cb3c09ca63e00d43b3f8a0642e7cbe2627874bae`. T7.1-T7.3
were committed together as one complete acceptance and guidance path. The script uses one Cargo job,
disabled incremental output and no dev/test debuginfo. Native fixtures own their
sources; PTYs and the native helper use isolated HOME/XDG roots. No network
provider, dependency, persistent service or installed editor is required.

The existing qualification measurements run eleven cold requests per row at
1 KiB, 64 KiB, 256 KiB and 1 MiB for complete, malformed and oversized-token
sources. Buffer construction is outside measurement. Median/p95/max-poll timings
are observations; inspected bytes, depth, retention and lexical steps are asserted
against the original fixed bounds. Existing tests cover depth 256/257, actual
metadata exhaustion, counted continuation, cancellation and stale revisions.

New integrated measurements cover both profiles and two views at each size.
They queue physical opener/body/closer input before publication, assert FIFO
release and exact final bytes, and measure maximum native host turn. Count-three
requests publish one complete transaction at admitted sizes and refuse atomically
at the 1 MiB prefix. Both views retain the canonical source revision. Existing
pair projection cases compare completed and pending Org/Markdown heading rows and
cursor mapping during same-line typing/wrap/unwrap; focused legacy continuity
cases retain their original refusal and frame assertions.

Measured native qualification results from the implementation run follow.
Values are median / p95 microseconds for eleven cold requests per cell. These
unoptimized native tests are not a like-for-like speed comparison with the plan's
optimized raw-scan baseline, which lacks context/work admission.

| Source bytes | Complete | Malformed | Oversized lexical token |
| --- | --- | --- | --- |
| 1024 | 439 / 470 | 439 / 476 | 411 / 468 |
| 65536 | 33680 / 38040 | 34650 / 35790 | 30088 / 32007 |
| 262144 | 150911 / 169178 | 147537 / 171527 | 143244 / 179223 |
| 1048576 | 136805 / 160623 | 139367 / 146025 | 119717 / 122922 |

Complete and oversized-token requests through 256 KiB confirm; malformed requests
report typed mismatch. Every 1 MiB row refuses at exactly 262,144 inspected bytes,
without a fabricated mate. Maximum qualification poll was 939 microseconds;
maximum lexical steps were 256. Ordinary rows retained 19,440 or 27,632 bytes.
Depth 256 confirmed after 512 bytes; depth 257 refused after 272 bytes with peak
256. Actual metadata exhaustion refused at 2,097,136 retained bytes after 129,872
inspected bytes, below the 2,097,152-byte cap.

Integrated two-view queue release took 2.9 ms to 1.81 seconds across the eight
profile/size rows, with maximum host turns of 0.885 to 20.941 ms. Host turns can
exceed the four-millisecond target; the measured qualification polls above remain
separate observations. The earlier focused three-case run reached a 24.705 ms
host turn; it also passed without changing limits. Count-three requests took 0.993 to 561.053 ms at admitted
sizes, while the 1 MiB cases refused atomically (170.384 ms non-modal and
1,584.587 ms modal). Both profiles published exactly one history group on success
and none on refusal. Existing pending-frame cases preserve Org/Markdown headings
and cursor placement before fresh analysis completes.

## Contracts Added or Changed

Adds one reproducible acceptance script and one native PTY example, plus three
native integrated cases. Runtime sources, public protocols, command identities,
factory mappings, queue capacity and all resource limits stay unchanged.
Specification/model/index text now distinguishes the original inspected baseline
from implemented editing and prepares exact-candidate closure without prematurely
marking editing Completed. Navigation and presentation remain Draft.

## Files of Interest

- `scripts/check-delimiter-editing.sh`: nonempty selection, focused batches, fixed digests, native map helper and two PTYs.
- `examples/delimiter-editing-acceptance.rs`, `examples/delimiter_editing_acceptance/journey.rs`: physical input and exact disk/configuration oracles.
- `tests/pairs/acceptance.rs`: measured queue release, count expansion/shared views and read-only/focus boundaries.
- `docs/tutorials/user-guide/02-edit-text.md`, `docs/how-to/configure-delimiter-editing.md`, `docs/reference/delimiter-editing.md`: tutorial, task controls and exact policy.

## Validation

Successful implementation checks:

- `cargo fmt --all -- --check`: passed.
- `bash scripts/check-delimiter-editing.sh`: passed, 192 cases with no failures or skips, the native factory helper and both Linux PTYs. The log's creation-to-last-write interval was approximately 332 seconds; this run reused the initial native build.
- `bash scripts/check-documentation.sh`: passed in 21.44 seconds; 710 commands, 332 configuration entries, 112 reachable pages, 791 local links/anchors and all 22 command/configuration reference tests.
- `bash scripts/build-user-guide.sh`: passed in 3.27 seconds, including strict Spore link checking and local rendering. No site was published.
- `git diff --check`: passed.

| Native target | Selected cases | Test execution seconds |
| --- | --- | --- |
| pairs | 78 | 175.27 |
| configuration | 11 | 22.09 |
| editing | 28 | 27.23 |
| commands/paste | 12 | 4.58 |
| macros | 40 | 37.57 |
| modal | 2 | 12.35 |
| org_headings | 8 | 9.37 |
| markdown | 13 | 20.01 |

Every declared filter printed nonzero coverage before its batch. The fixed factory
catalog/manifest digests passed. The native helper retained 28 full maps, 707 old
commands plus three additions, inherited/retired leaves and both profile routes;
its process had one thread. The non-modal PTY took 1,244 ms and 41,170 output bytes,
with 36,656 kB peak RSS; modal took 1,428 ms and 43,583 bytes, with 38,976 kB peak
RSS. Both observed 350 ms idle silence with zero CPU ticks and restored/reaped
terminal state. Fixture directories were removed after their checks.

The initial focused three-case build took 1 minute 49 seconds from a cold ordinary
Cargo target; those cases then passed in 17.67 seconds. Focused native source
checks passed for all three new Rust files under the existing safe-Rust,
parameter/nesting/complexity limits. Early PTY rehearsals corrected the harness's
gutter observation and the existing last-buffer `*scratch*` label. Their exact
source oracle and all acceptance bounds were unchanged. The vendored crossterm
parentheses warning is pre-existing and non-blocking.

Full staged-source hooks passed: formatting, strict all-target/all-feature
workspace Clippy with zero blocking or deferred diagnostics, and native policy
for 1,956 first-party Rust files, two packages and 78 Cargo roots. The immutable
staged tree was `db522de16d14b829d9cf082b976dadce80d12f9d`; the cold private-cache
hook took 250.11 seconds. The final exact five-command clean-HEAD controller
validation passed at `51e97dceef8db07021e9be2f7eef5cf77f61fdc8` in 369.20 seconds.
The canonical PR contract was verified; PR #61 integrated that same candidate
into dev, confirmed by Forgejo, fetched ancestry and Acta. Combined delivery-set
acceptance remains controller-owned; this script run is slice acceptance, not that full gate.

## Delivery-set closure

| Slice | Report | Canonical PR | Current boundary |
| --- | --- | --- | --- |
| 1 | [Configuration](slice-1-config.md) | #55 | Delivered. |
| 2 | [Qualification](slice-2-qualification.md) | #56 | Delivered. |
| 3 | [Typing](slice-3-typing.md) | #57 | Delivered. |
| 4 | [Deletion](slice-4-deletion.md) | #58 | Delivered. |
| 5 | [Regions](slice-5-regions.md) | #59 | Delivered. |
| 6 | [Provenance](slice-6-provenance.md) | #60 | Delivered. |
| 7 | This report | #61 | Delivered; combined gate PASS. |

Gate preparation verified canonical PRs #55-#61 as merged ancestors of fetched
dev. All seven reports are delivered and all 21 task entries are complete. The
last owned Slice 7 worktree was clean and inactive and was removed without force;
unrelated worktrees were preserved. Required gate executables, scripts and
native/PTY fixtures are present; existing hooks remain configured unchanged.

Preparation checks passed:

- `bash -n scripts/check-quality.sh scripts/test-hooks.sh scripts/check-delimiter-editing.sh scripts/check-documentation.sh scripts/build-user-guide.sh`
- `git diff --check`

The preparation checks above preceded the controller's full gate. The controller
subsequently ran this unchanged command with result PASS at tested candidate
`eb65b227dee8b42bf8d0c8a6e0ef12f5675c756b`, receipt
`QD5ALW2WJR64VCCTK4CJAOZPDK`:

```sh
env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash -c 'set -euo pipefail; make lint; make test-hooks; bash scripts/check-delimiter-editing.sh; bash scripts/check-documentation.sh; bash scripts/build-user-guide.sh; git diff --check'
```

- Formatting, strict all-target/all-feature workspace Clippy and native source policy passed for 1,956 first-party Rust files, two packages and all 78 Cargo roots; zero blocking or deferred Clippy diagnostics.
- Real Git hook acceptance passed, including immutable/partial index cases, source bounds, linked-worktree ownership, cache isolation and concurrent-owner refusal. Its cold fixture took 35 seconds and warm fixture two seconds.
- All 192 selected native acceptance cases passed with no failures/skips: 78 pair, 11 configuration, 28 editing, 12 paste, 40 macro, two modal, eight Org and 13 Markdown cases. Every filter selected nonzero coverage. Fixed byte/depth/metadata/work, cancellation, stale revision, counted atomicity, shared views and pending-frame continuity assertions remained intact.
- The unchanged factory helper passed 28 complete maps, the frozen 707-command baseline plus three editing commands and both profile routes.
- Both Linux PTYs passed exact physical typing, skip, literal paste, macro replay, undo/redo, save/reopen and focus checks with restored/reaped terminal state. Non-modal took 1,261 ms and 41,170 bytes; modal took 1,393 ms and 43,583 bytes. Both observed zero idle CPU ticks over 350 ms; peak RSS was 36,612 and 38,936 kB respectively.
- Integrated queue release completed in both profiles at all four source sizes; the maximum observed host turn was 20.815 ms. The 1 MiB count-three requests refused atomically. Depth 256 confirmed, 257 refused, and actual retention exhaustion stopped at 2,097,136 retained bytes. The four-millisecond scheduling value remains a target.
- Documentation passed for 710 commands, 332 configuration entries, 112 reachable pages and 791 local links/anchors, with all 22 reference tests. The strict local User Guide build and final whitespace check passed. No site was published.

All 21 tasks, seven canonical PRs and seven reports are delivered. Integrated
owned worktrees were removed without force; unrelated worktrees remain. After
validation no owned editor fixture process remained. The pre-existing vendored
crossterm parentheses warning remained non-blocking.

The approved closure is a subsequent documentation-only descendant of the tested
candidate, limited to the twelve configured Markdown paths: this plan/tracker,
seven reports, editing specification/model status and operations index. The
exact closure commit is retained in Git and Acta and pushed on dev. All files
outside that allowlist, including runtime sources, configuration, fixtures,
scripts, public identities and acceptance assertions, retain the tested bytes.
Editing alone becomes Completed. The shared model remains Partially approved;
navigation/presentation and their specifications remain Draft. No successor,
main promotion, release, tag, mirror, installation or publication is part of
this closure.

## Risks and Follow-ups

The four-millisecond cooperative scheduling value is a target, not a wall-clock
guarantee. Host turns include native source/projection work and are reported
separately from individual qualification polls. All seven slices are integrated.
The combined gate passed at the immutable candidate above. Closure preserves
that receipt separately from its later Markdown-only commit. Conservative
uncertainty/refusal policies and navigation/presentation Draft boundaries remain.
