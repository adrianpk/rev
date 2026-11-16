# Slice 3: Delimiter rendering

Status: reviewing
Delivery set: rev-delimiter-presentation
Plan: [Delimiter Presentation Delivery](../../../plan/rev-delimiter-presentation.md)
Tracker: [Delimiter Presentation Tracker](../../../tracker/rev-delimiter-presentation.md)
Branch: `feat/rev-delimiter-presentation-03-rendering`
PR: [#69](https://forge.adrianpk.com/rev/rev/pulls/69)

## Purpose

Deliver T3.1-T3.3: source-qualified depth colors and active-pair marks through the
native view projection, preserving source, navigation and authoritative overlays.

## Delivered Behavior

Depth style colors confirmed structural endpoints with the fully resolved cyclic
pair palette. At-point entry takes precedence over the endpoint immediately
before point; ordinary body text does not select an arbitrary enclosing pair.
Depth keeps its foreground on the active pair and adds background and underline.
Match style marks only the two confirmed endpoints. Eligible quote endpoints
retain their ordinary string foreground, including syntax-disabled fallback.
Off preserves ordinary source rendering. Automatic editing can remain disabled,
and read-only buffers retain the same visual behavior.

Native source/grapheme, wrap, fold, gutter and pane mappings place each mark on
the current source cell. Org/Markdown heading/list prefixes and table text retain
their existing projection. Selection/search keep their foreground/background;
selection suppresses the pair underline and search keeps its own underline.
The cursor remains owned by the native projection. NO_COLOR omits RGB escapes
while active endpoints retain underline without replacement punctuation.

A confirmed mate outside the visible projection offers its one-based source line
and terminal-cell column in an otherwise empty echo row. It does not change
point, scroll, source or session feedback. Explicit messages, errors and native
prompts retain their surfaces. Unfinished, malformed, uncertain and limited source
does not acquire invented mates, depths or error marks. Already confirmed prefix
pairs remain usable even when a later suffix is malformed.

## Implementation Notes

The session admits current visible ranges to the existing buffer-owned qualifier;
there is no second parser, source copy, row cache, worker or timer. Polls and
pending export retain the existing 256-token/4 ms bounds and foreground priority.
Only pending qualification requests another native turn. Complete depth/match
and off have no pair-owned deadline. Empty source requests no qualification.
Identical repaint interests reuse their live ticket; explicit request generations
and source invalidation retain their existing semantics.

Each view retains a ticket and pending flag alongside its existing preferences.
Derived tickets are excluded from semantic View equality, following the native
wrap-cache rule, so frame observation cannot invalidate captured calendar/edit
authority. Rebind resets derived state while retaining the view preference;
splits start from the resolved defaults. Closed/rebound/off interests are pruned
through the buffer owner. The shared retention budget now charges each admitted
view's actual inline presentation size as well as entry/publication capacities;
the qualifier receives a byte charge without depending on view implementation.

Delimiter overlays extend the existing immutable Paint cells. Native selection,
search and cursor composition remains authoritative. Source endpoint identity is
available on derived frame paints for direct writer observations. Panel finishing
and source-column observation remain cohesive helpers within native projection.

Development regressions exposed transient tickets participating in View identity
and an empty-source pending turn. The implementation corrects both; existing
calendar/timer assertions remain unchanged. The complete factory-map test reuses
the independent reviewed baseline and accepted deltas rather than deriving an
expected map from current defaults. The Unreleased changelog contains one concise
visible-feature entry using changelog-normalize.

Implementation/report introduction: `9cedf2c18e2bf90861db5b0b73aff818e5bf5500`.
Canonical review: [PR #69](https://forge.adrianpk.com/rev/rev/pulls/69).

## Contracts Added or Changed

- Current qualified endpoints participate in native immutable cell painting.
- View presentation requests have bounded cooperative progress and idle behavior.
- Derived qualification state does not change semantic View identity.
- Per-view inline presentation retention is charged to the shared buffer cap.
- Frozen acceptance bytes, catalog, bindings, dependencies and hooks are preserved.

## Files of Interest

- `src/display/pairs.rs`: depth/match paint, attributes and offscreen feedback.
- `src/display/syntax.rs`, `style.rs`, `selection.rs`: native paint precedence.
- `src/session/pairs/presentation/rendering.rs`: admission, polls and retirement.
- `src/view/pairs.rs`: view preferences, derived identity and visible source range.
- `src/buffer/pairs.rs`, `src/pairs/presentation/store.rs`: interest reuse/accounting.
- `tests/pair_presentation/rendering/`: 15 nonempty native writer cases.
- `CHANGELOG.md`: one visible product entry.

## Validation

Exact clean-HEAD evidence is recorded through looper validate after the canonical
PR-reference commit. The controller owns integration and the combined gate.

- PASS: `cargo fmt --all -- --check`.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pair_presentation rendering -- --test-threads=1`: 15 cases, also repeated by the broader regression command.
- PASS: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pair_presentation --test display --test gutter --test wrapping -- --test-threads=1`: all 163 cases (50 presentation, 49 display, 37 gutter, 27 wrapping).
- PASS: focused native source policy on all 20 changed source/test files,
  retaining safe Rust, parameters <=5, nesting <=3 and complexity <=10.
- PASS: preserved implementation hook at staged tree
  `3c41e04e61d88e5db93ba3cdabf4ba152ccaa2ef`: full workspace Clippy, zero
  blocking/deferred diagnostics, native source policy over 1993 files/81 roots.
- PASS: `git diff --check`.
- PASS: frozen acceptance SHA-256 remains
  `8a0ccc8e6a9ac4ebb24f4445c2f314b472045f6a426014b53ced065f477da771`.

The initial staged hook accepted native source policy across all 1993 files/81
roots and rejected one needless temporary collection. Retiring tickets directly
through Windows removes that collection without changing admission. The next
Clippy pass rejected an indexed test loop; enumerating the same three expected
rows preserves every assertion. The successful receipt above applies to the implementation
snapshot; this PR-reference commit changes operational documentation only.

## Risks and Follow-ups

Pulse lifetime belongs to Slice 4; the configured pulse style does not display a
perpetual mark or own a timer here. Full 24-burst continuity, physical-key/input
service measurements and the two representative PTYs remain Slices 5-6.
Confirmed malformed source retains ordinary ink; optional mismatch warning marks
are not fabricated from a global outcome without a qualified cell. The existing
vendored crossterm parentheses warning is unchanged.
