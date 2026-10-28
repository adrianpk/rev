# Slice 17: resolve Org local times and zone changes

Status: reviewing
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-17-time-zones`
PR: pending

## Purpose

Observe native system/named time zones and resolve local times without guessing.

## Delivered Behavior

Native immutable Zone observations admit system localtime, captured local TZ,
explicit UTC and validated relative IANA names. Local TZ also admits absolute
TZif paths and bounded POSIX rules. Explicit selection takes precedence over TZ.

## Implementation Notes

TZif v1/v2/v3/v4 parsing checks headers, transitions, type indices, designations,
indicator consistency and footer agreement. Unknown -00 records and exhausted
coverage return errors. POSIX rules cover both hemispheres, month/week/weekday,
Julian and ordinal dates, explicit offsets, negative DST, all-year DST and signed
rule times. No implementation-defined implicit DST rule is invented.

Bounds are 1 MiB per regular file, 65536 transitions, 256 types, 4096 designation
bytes, 1024 POSIX bytes and 64 abbreviation bytes. Known offsets remain within
24 hours. Leap-second TZif files reject under the existing Unix-time profile.
Future recurrence evaluates at most eight events and retains no expanded years.
Named paths reject empty, . and .. components. Files are preflighted before
opening, checked again through the descriptor, and read with a bounded take.

The parser follows [RFC 9636](https://www.rfc-editor.org/rfc/rfc9636.html) and
local selection uses documented [TZ formats](https://sourceware.org/glibc/manual/latest/html_node/TZ-Variable.html).
Independent fixture encoders use the published byte format; no external reader
or copied implementation is part of native operation.

T17.2 adds ZonedOperation, which owns one fallible injected clock capture, its
effective civil date and a complete immutable zone. Local resolution checks at
most 258 distinct observed offsets, orders fold choices by UTC instant, and
rejects invalid forced offsets. A forward gap returns Nonexistent; unknown
coverage returns an error rather than a manufactured gap.

Typed org.time settings use normal include layering/provenance. local is the
factory selection, system ignores TZ, and UTC/named policy overrides it. Only
prompt is admitted for ambiguous/nonexistent policies. Task logging now consumes
this policy through the shared owner, retaining a single timestamp through note
entry. The former private task-zone reader is removed. No source directive can
select a zone file or execute a program.

## Contracts Added or Changed

- ORG07: native explicit zone offsets and known/unknown coverage.
- ORG18: finite immutable observations without a service, dependency or idle loop.

## Files of Interest

- `src/org/time/zone/`: native TZif/POSIX parsing and explicit observation inputs.
- `tests/org_time/zone_*.rs`, `zones.rs`: independent format, operation and IO fixtures.
- `src/config.rs`, `src/config/`: typed time policy, validation and provenance.
- `src/session/outline/tasks/clock.rs`: selected native task-log zone consumer.
- `scripts/configuration_ui/org_tasks.py`: real configured-UTC task transitions.

## Validation

T17.1 development checks passed:

- `cargo check --quiet`
- `cargo test --test org_time zone -- --nocapture`: five tests passed, zero skips.
- Focused changed-source quality passed at the original safe-Rust, parameter,
  nesting and complexity thresholds.

Fixtures cover exact transition boundaries, future footer recurrence, empty
coverage, unknown types, both hemispheres, negative/all-year DST, leap rules,
quoted names, fractional offsets and real owned system/named zone files.

T17.2 development checks passed:

- `cargo test --test org_time --test configuration time -- --nocapture`: four
  configuration and two civil-format/time tests passed.
- `cargo test --test org_time zone -- --nocapture`: nine zone tests passed,
  including explicit fold/gap results, civil-year edges and immutable operation
  snapshots after owned zone-file replacement and captured-TZ changes.
- `python3 scripts/check-config-reference.py`: 114 schema/catalog entries,
  143 default bindings and 13624 exact packaged template bytes agree.
- Focused changed-source quality passed with the original thresholds.

T17.3 development checks passed:

- `cargo test --test org_time zone -- --nocapture`: all 20 focused zone tests
  passed, zero skips, including exact parser limits and every truncated prefix.
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-tasks`:
  all four owned RGB/NO_COLOR editing/read-only sessions passed, zero skips.
  Explicit configured UTC overrides an invalid child TZ. Captured completion
  stamps lie within independently observed UTC minute bounds; all prior note,
  source-byte, save, undo/redo, snapshot and child/descriptor cleanup assertions
  remain intact. No operator environment or document is mutated.
- Focused changed-source quality passed with the original thresholds.

Independent failing fixtures reproduced unknown POSIX -00 designations being
admitted and footer agreement checking only the numerical offset. Unknown
POSIX designations now reject, and the footer must agree with the last offset,
DST indicator and designation. The fixtures now pass without weakening their
observable expectations. Valid generated fixture labels were made consistent
with that published rule; their transition instants, offsets and fold choices
remain unchanged.

Additional fixtures cover malformed headers/counts/records, reserved bytes,
unused invalid types, version mismatches, indicator ordering, leap data,
regular-file/size/path limits, invalid source selection and clock failure.
Half-hour and negative-DST folds/gaps, a full skipped day, three simultaneous
fold choices, boundary years and external file replacement remain explicit.
The existing task-log release note now documents selectable operation zones.

Final required ordered checks will be recorded through the controller after
the canonical PR-reference follow-up is committed and pushed. Focused same-HEAD
checks also cover native Org editing, configuration reference agreement and
four owned task terminal sessions. Receipt identities belong in Acta and the
checkpoint without moving the validated candidate afterward.

## Risks and Follow-ups

Filesystem reads are bounded; a blocked kernel filesystem operation has no hard
execution deadline. Leap-second TZif and implicit POSIX DST defaults remain
explicit diagnosed outcomes, as documented. Timestamp entry and its interactive
offset/replacement choices belong to the recorded Slice 18; the native time
owner already retains all valid candidates and rejects invalid forced offsets.
