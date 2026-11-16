# Slice 4: Finite pulse lifecycle

Status: reviewing
Delivery set: rev-delimiter-presentation
Plan: [Delimiter Presentation Delivery](../../../plan/rev-delimiter-presentation.md)
Tracker: [Delimiter Presentation Tracker](../../../tracker/rev-delimiter-presentation.md)
Branch: `feat/rev-delimiter-presentation-04-pulse`
PR: pending

## Purpose

Deliver T4.1-T4.3: one finite, view-local pulse from accepted source input or
successful delimiter motion, with current source qualification and exact expiry.

## Delivered Behavior

Pulse style marks the active confirmed endpoint and its mate for the resolved
50..1000 ms duration, defaulting to 250 ms. Accepted source input and successful
native delimiter motion can start a generation. Unchanged point, failed motion,
release metadata, repaint, resize and returning focus cannot extend or restart
it. Selecting a style leaves point and viewport unchanged.

Expiry clears both marks and the pair-owned wakeup at the original deadline.
Source/point changes, a different selected view, prompts, focus loss, C-g,
reload, style replacement and retired qualification cancel stale generations.
Pending qualification can confirm a mark only within its captured lifetime;
late results cannot revive it. New accepted input replaces the prior lifetime.

Pulse uses the existing match foreground/background and underline, preserving
native quote foregrounds and selection/search precedence. NO_COLOR retains
underline without RGB or terminal blink. Steady depth/match/off have no pulse
deadline, and an independent native Org timer retains its original due time.

## Implementation Notes

One inline pulse, monotonic expiry and generation counter belong to the native
view presentation owner. Its actual size is charged through the existing
buffer-owned interest before admission, under the unchanged shared 2 MiB cap.
The counter deduplicates nested command, accepted-outcome and deferred-publication
boundaries. The source point and live qualification ticket guard that generation;
major-mode retirement is detected even when the source stamp is unchanged.

Input admits a raw endpoint candidate without scanning or waiting for its pulse
duration. The existing bounded qualifier confirms it cooperatively, retaining
foreground priority and the 256-token/4 ms poll limits. The native minimum-deadline
owner combines pending work, pulse expiry and unrelated deadlines. Expiry requests
redraw even when an earlier native outcome retires the pulse in the same turn.

Frame reprojection can refresh the source-range ticket while preserving the
original expiry. Suspended geometry retains that admitted lifetime; restoration
cannot recreate an expired pulse. Cancellation releases its native interest even
when no frame ticket is present. There is no second parser, source copy, row cache,
worker or timer thread. Derived pulse state remains outside semantic View equality.

## Contracts Added or Changed

- One finite monotonic pulse per admitted view, triggered only by accepted input.
- Current source, point, view and qualification ownership govern marks and wakeups.
- Repaint and cooperative completion preserve the original expiry.
- Expiry/cancellation compose through native redraw and minimum-deadline ownership.
- Frozen acceptance bytes, catalog, bindings, dependencies and hooks are preserved.

## Files of Interest

- `src/view/pairs/pulse.rs`: inline generation, admission ticket and expiry.
- `src/session/pairs/presentation/pulse.rs`: triggers, qualification and retirement.
- `src/session/pairs/presentation/rendering.rs`: cooperative work and wakeups.
- `src/session/events.rs`, `layout.rs`: accepted input and deferred publication.
- `src/buffer/pairs.rs`, `src/pairs/state/presentation.rs`: live interest checks.
- `src/display/pairs.rs`: confirmed active endpoints through native immutable paint.
- `tests/pair_presentation/pulse/`: controlled-clock input and lifetime cases.
- `CHANGELOG.md`: finite pulse behavior in the existing visible-feature entry.

## Validation

Exact clean-HEAD evidence is recorded through looper validate after the canonical
PR-reference commit. The controller owns integration and the combined gate.

Controlled clocks advance through 50/250/1000 ms boundaries;
tests do not sleep through production pulse durations. The accepted-key case
checks that dispatch returns within 250 ms for a configured 1000 ms pulse; this
is a native-handler observation, not physical terminal input acceptance.

- PASS: `cargo fmt --all -- --check`.
- Required pulse subset: all 18 cases PASS in the broader native batch below;
  the exact required command is recorded for the final clean PR HEAD through
  looper validate.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pair_presentation --test display --test pair_navigation -- --test-threads=1`: all 165 cases (68 presentation, 49 display, 48 navigation), with no ignored cases or empty filters.
- PASS: focused native source policy over all 21 changed Rust source/test files,
  retaining safe Rust, parameters <=5, nesting <=3 and complexity <=10.
- The preserved staged hook checks full workspace formatting, strict Clippy and
  native source policy before committing; its successful receipt is added with
  the canonical PR reference.
- PASS: `git diff --check`.
- PASS: frozen acceptance SHA-256 remains
  `8a0ccc8e6a9ac4ebb24f4445c2f314b472045f6a426014b53ced065f477da771`.

Development checks exposed missing redraw propagation when an earlier callback
retired an expired pulse. The implementation preserves that redraw, and the exact
expiry assertion remains in the controlled-clock case.

## Risks and Follow-ups

Full 24-burst continuity, frozen workload/input-service measurements and the two
representative PTYs remain Slices 5-6. No physical-key acceptance or full delivery
gate is claimed here. The existing vendored crossterm parentheses warning remains.
The controller owns integration, the combined gate and successor startup.
