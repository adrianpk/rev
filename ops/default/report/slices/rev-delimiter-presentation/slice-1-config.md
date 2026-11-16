# Slice 1: Configuration and view controls

Status: delivered
Delivery set: rev-delimiter-presentation
Plan: [Delimiter Presentation Delivery](../../../plan/rev-delimiter-presentation.md)
Tracker: [Delimiter Presentation Tracker](../../../tracker/rev-delimiter-presentation.md)
Branch: `feat/rev-delimiter-presentation-01-config`
PR: [#67](https://forge.adrianpk.com/rev/rev/pulls/67)

## Purpose

Deliver DPR03 configuration and view controls through T1.1-T1.3, preparing
presentation without claiming visible delimiter rendering.

## Delivered Behavior

Global and optional language pairs.highlight/pulse_ms resolve independently of
automatic editing, defaulting to depth and 250 ms. Durations admit 50..1000 ms.
Omitted language leaves inherit final globals and retain native leaf origins.
Malformed types, styles, durations and colors reject the whole candidate,
including invalid included values masked by later overrides.

Four optional palette.pairs roles resolve after ordinary palette merging.
Absent roles inherit selection, foreground, warning and the ordered syntax
function/number/keyword/type/string/punctuation cycle. Explicit opaque RGB
colors win; depth arrays replace the cycle and admit 1..8 entries.

The unbound pair-highlight-mode and pair-highlight-style commands change only
the selected view. The bounded style prompt admits depth/match/pulse/off with
Tab guidance and C-g cancellation; supplied numeric prefixes refuse. Off
remembers the last non-off style, using depth when no prior active style exists.
New views and splits use resolved language/global defaults; sibling overrides
stay independent and explicit overrides survive source rebinds. Read-only
source and disabled automatic editing remain admitted. Source bytes, point,
viewport, mark, undo/redo and settings files remain unchanged.

## Implementation Notes

The existing partial configuration, merge, validation, origin and projection
owners admit both preferences. String-only style deserialization prevents Serde
from treating a unit-variant table as a valid configuration string. Palette role
absence survives composition until final RGB projection.

View PairPresentation owns only runtime style and last-active overrides. It
reads startup defaults from the buffer's resolved preferences, preserving native
language/reassociation updates without a second settings lookup. The session
captures the selected view for prompt completion and uses native focus and
argument admission. Pair-control dispatch stays with the pair owner to retain
the root dispatcher's complexity ceiling. No source cache, timer, service or
dependency is added.

T1.1-T1.3 form one cohesive implementation commit because the new typed settings,
view controls, independent catalog assertions and reference schema must agree.
The implementation/report-introduction commit is `fdc1aa13dedfcef2c6ee4902669c5caad90dca6d`.
Canonical review: [PR #67](https://forge.adrianpk.com/rev/rev/pulls/67).

## Contracts Added or Changed

- Two independent global/language preferences and four global palette roles.
- Exactly two unbound catalog additions: 717 -> 719 commands.
- All existing factory bindings and frozen oracle bytes remain unchanged.
- Reference examples and documentation checks cover dynamic palette fallback;
  drift checks reject fixed sample overrides or a different documented role.
- No release note: qualification, visible rendering and pulse lifetime remain
  assigned to their later approved slices.

## Files of Interest

- `src/config/pairs.rs`: typed preferences, inheritance and merge origins.
- `src/config/palette.rs`: optional and effective pair roles.
- `src/config/project.rs`: fallback after palette composition.
- `src/view/pairs.rs`: view-local style and last-active override.
- `src/session/pairs/presentation.rs`: command and prompt ownership.
- `tests/pair_presentation.rs`: 14 nonempty configuration/control cases.
- `examples/factory_shortcuts_acceptance/matrix.rs`: independent complete maps.
- `examples/documentation_check/catalogs.rs`: dynamic-default drift checks.

## Validation

All three ordered required commands PASS through looper validate at clean
published `f1515a01cf5434ce163840a99c1729da0bfaea0e`, integrated by PR #67.
Acta receipt: `01M7GSHBNQNRM98SFPE3E6MGCC`. The later controller gate PASS is
recorded in [delivery-set closure](slice-6-acceptance.md#delivery-set-closure).

- PASS: `cargo fmt --all -- --check`.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pair_presentation config -- --test-threads=1`: 14 cases.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test configuration pairs -- --test-threads=1`: 11 existing cases.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pairs inventory -- --test-threads=1`: two independent inventory cases.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pairs shared_toggle -- --test-threads=1`: existing automatic-editing dispatch after the routing change.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pair_navigation catalog_and_focus -- --test-threads=1`: one existing case.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --example documentation-check -- --test-threads=1`: nine drift/navigation cases.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example factory-shortcuts-acceptance -- --native target`: 719 commands, six actions, 28 complete maps and native effective hints in both profiles.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-documentation.sh`: 340 schema entries, 112 reachable pages, 796 local links/anchors, 10 command and 12 configuration reference cases.
- PASS: scoped native source policy over configuration/session/view, command,
  new presentation tests and documentation checker: 365 files plus four focused
  root/dispatch inputs, with parameter, nesting and complexity ceilings retained.
  Preserved commit hooks additionally
  enforce the exact staged candidate before each commit.
- PASS: implementation commit's exact staged hook: full workspace Clippy with
  zero blocking/deferred diagnostics, all 1,977 first-party files and 81 Cargo
  roots, with no unsafe code or parameter/nesting/complexity exceptions.
- PASS: `git diff --check`.

Development checks exposed Serde table admission and root dispatch complexity 11.
The first preserved hook rejected staged tree `e06b25418756a1be5fa92c8ad2c16b2d1fb1cf4a`
for that dispatch ceiling after full workspace Clippy passed with zero blocking
or deferred diagnostics. String-only decoding and pair-owned routing correct
both issues without a lint exception. The corrected configuration/control rerun
passes all 14 cases, and the existing automatic-editing toggle passes.

## Risks and Follow-ups

PR #67 integrated validated commit `f1515a01cf5434ce163840a99c1729da0bfaea0e`
into dev. All three ordered required checks passed once through looper validate.
Fetched ancestry, clean state and no owned processes were verified; the integrated
worktree was removed without force. Qualified facts, rendering and finite pulses
were delivered in Slices 2-6; the final gate receipt is linked above.
The existing vendored crossterm unused-parentheses warning remains unchanged.
