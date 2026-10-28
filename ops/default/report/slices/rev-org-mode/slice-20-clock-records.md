# Slice 20: record native Org task clock intervals

Status: reviewing
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-20-clock-records`
PR: `#121`

## Purpose

Deliver native task-owned clock intervals under the
[approved plan](../../../plan/rev-org-mode.md#slice-20-record-native-org-task-clock-intervals)
and [tracker](../../../tracker/rev-org-mode.md#slice-20-record-native-org-task-clock-intervals).

## Delivered Behavior

T20.1 connects compiled clock-in/out/cancel/goto commands and reviewed C-c C-x
bindings to canonical source. An interval belongs to the selected heading,
retains captured UTC instants and the start zone, and closes with compatible
inactive timestamps and elapsed H:MM. Minute endpoints follow source precision;
offset changes do not become elapsed-time errors. Default placement is LOGBOOK;
CLOCK_INTO_DRAWER and LOG_INTO_DRAWER select checked local/inherited ownership.

T20.2 adds effective `org.clock.drawer` and `org.clock.remove_zero` policy through
normal configuration layering, exact provenance and validated source-safe names.
Zero-minute intervals remain by default. Explicit removal and cancel delete the
owned interval and an otherwise empty drawer; other logs and metadata remain
exact. Newline wrappers added at unterminated EOF are removed with the interval.

T20.3 verifies read-only admission before time acquisition, late capture failures,
literal/child ownership, revision-stale rejection, source bounds and hidden-buffer
commands. Native deadline tests advance the host across one second through a day:
no clock capture, source revision, history entry or redraw appears. Shared-view
save tests preserve BOM, CRLF, Unicode and unterminated EOF through both operations
and complete undo. The existing task PTY journey now exercises clock keys/M-x,
goto, cancel, exact interval/save contents and undo/redo in RGB and NO_COLOR.

## Implementation Notes

Clock values are separate from source authority. Native locators carry the exact
buffer revision; changed source rejects use and retains its open interval.
The current section observation admits at most 262144 UTF-8 bytes and excludes
child entries and literal block contexts. Duplicate selected drawers and an
existing incomplete source clock diagnose before publication. Each clock action
uses the existing buffer/window/history transaction. A hidden source can close
without changing the invoking view. The active value adds no idle deadline.

Storage and bindings follow the [Org clock command reference](https://orgmode.org/manual/Clocking-commands.html).
The installed Emacs 31.1/Org source was consulted for minute endpoint precision,
source spacing and empty-drawer behavior; native code invokes no reference editor.

## Contracts Added or Changed

- ORG19: task-owned native CLOCK source, checked instants and normal edit authority.
- ORG21: recorded interval duration independent of work-unit configuration.

## Files of Interest

- `src/org/time/clock.rs`: captured interval and elapsed minute records.
- `src/org/visibility/tags/tasks/clocks.rs`: checked task/drawer source placement.
- `src/session/outline/clocks.rs`: native command and transaction ownership.
- `tests/org_time/clocks.rs`, `tests/org_editing/clocks.rs`: native interval and editing boundaries.

## Validation

T20.1 development checks passed:

- `cargo check --quiet`.
- `cargo test --test org_time --test org_editing clock -- --nocapture`: ten selected tests, zero skips.
- `cargo test --test commands org_visibility`: existing command identity checks passed.
- `cargo test --test commands clock_keys`: clock/radio/cancel binding parity passed.
- `cargo test --test commands catalog_reference`: compiled command references agree.
- `cargo clippy --lib -- -D warnings`.
- `python3 scripts/check-config-reference.py`: 115 schema/catalog entries, 46 RGB roles, 169 default bindings and 14706 packaged template bytes agree.
- `python3 scripts/check-source.py`: existing source quality thresholds passed.

T20.1 commit: `8a6a6d74d2797e381e31d299a243b064ef08c5c5`.

T20.2 development checks passed:

- `cargo test --test org_editing --test configuration clock`: eight selected tests, zero skips.
- `cargo clippy --lib --test org_editing --test configuration -- -D warnings`.
- `python3 scripts/check-config-reference.py`: 117 schema/catalog entries, 46 RGB roles, 169 default bindings and 14758 packaged template bytes agree.
- Changed-source quality passed at the original limits.

Development validation caught a missing initializer, a collapsible conditional
and a complexity boundary. The initializer and conditional were corrected, and
source owner conversion now uses the existing qualified span. Checks were retained.

T20.2 commit: `8395add8fb79eb215a3e1c78880fb5bdeb664152`.

T20.3 development checks passed:

- `cargo test --test org_editing clock_edges -- --nocapture`: five native boundary/save/deadline cases passed, zero skips.
- `cargo clippy --test org_editing --test org_time --test commands -- -D warnings`.
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-tasks`: four owned editing/read-only RGB/NO_COLOR task PTYs passed, zero skips.
- Changed-test source quality passed at the original thresholds.

Terminal assertions retain all prior task/timestamp/repeat journeys, exact source
snapshots, normal save/history, isolated HOME/XDG/cwd, deadlines and owned cleanup.

T20.3 commit: `4cc89289a740cdb97a8a4f35ed1f9875aeabb1fa`.

[Canonical PR #121](https://forge.adrianpk.com/adrianpk/rev/pulls/121) targets dev.
Final required validation is recorded by the controller against the final clean
PR head, in the approved order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_time --test org_editing --test commands`
- `python3 scripts/check-source.py`
- `git diff --check`

Final focused evidence also covers full configuration/display regressions,
configuration-reference agreement and the four owned task PTYs.

Final focused receipt `QJ2KXRM5CKINNO4OG5PBD24MWN` rejected the earlier head
because a pre-existing keymap test classified `org-clock-in` as a future unknown
command. The unknown-command case now names an absent command, and a positive
case verifies all four delivered clock aliases reach their compiled targets.
Reserved cancellation, role admission and prefix assertions remain unchanged.
`cargo test --test configuration org_keymaps` passed three cases after correction;
affected configuration Clippy and changed-test source quality also passed.
Final successful receipts are regenerated for the corrected clean head.

Required receipt `XEU6NEXRVS4VDATLV3WRKGKA3O` exposed the equivalent historical
catalog absence assertion and an over-limit fixture timeout under suite load.
The catalog now retains only absent commands. The entry fixture still asserts
exactly 262145 UTF-8 bytes and unchanged rejection/source/view/history outcomes,
using ordinary comment rows to avoid unrelated markup work. The existing
five-second observation deadline and production limits remain unchanged. Focused
admission passed in 1.07 seconds; catalog, affected Clippy and source quality pass.

## Risks and Follow-ups

Active-clock switching, completion/quit/close recovery and optional persistence
belong to the recorded Slice 21. The delivery-set full gate has not run.
