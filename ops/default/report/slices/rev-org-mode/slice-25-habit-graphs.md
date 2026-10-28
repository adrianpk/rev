# Slice 25: render native Org habit consistency graphs

Status: reviewing
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-25-habit-graphs`
PR: `#126`

## Purpose

Deliver bounded consistency graphs under the
[approved plan](../../../plan/rev-org-mode.md#slice-25-render-native-org-habit-consistency-graphs)
and [tracker](../../../tracker/rev-org-mode.md#slice-25-render-native-org-habit-consistency-graphs).

## Delivered Behavior

C-c C-x h opens the current buffer's read-only native habit agenda. Graphs show
not-yet-due, available, imminent and overdue states. Actual completion uses *,
and today uses ! when no completion takes precedence. ASCII status glyphs
retain meaning without color. Late completions stay overdue by default; an
explicit done-always-green preference selects ready status. Past unmarked
non-overdue days use subdued backgrounds rather than imply recorded success.

K toggles visibility, A selects all habits on today's agenda, and C-u K toggles
today-only display. The d prompt selects a view date; G accepts preceding days,
following days and preferred column. Invalid editable drafts preserve the active
policy. C-g cancels a prompt back to its agenda, then q/C-g returns to source.
Up/Down scroll rows. Source edits, paste and pointer input cannot mutate the
canonical document through this viewport.

The g command explicitly refreshes source and captured time. Dirty source is
observed without saving; old rows remain marked stale until refresh. A today
view follows a new captured day only on refresh. Repaints retain immutable rows
and create no graph clock reads, source writes or inactive polling deadlines.

## Implementation Notes

Factory preferences match Org: 21 preceding days, 7 following days, column 40,
today-only true, show-all-today false and done-always-green false. Each visible
side admits 0..366 days and column admits 0..4096. Typed layers, leaf provenance
and complete-candidate rejection cover all preferences and eight backgrounds.

Graph reconstruction uses actual completion dates and distinct +, ++ and .+
modes. Org's graph conversion floors 30.4-day months and 365.25-day years; this
display policy does not replace checked source calendar recurrence. One graph
has at most 733 cells and 4096 source completions. One selected-buffer agenda
admits 128 rows, retaining existing entry/history/source observation bounds.
Excess, incomplete or stale requests reject before replacing the current view.

The existing native report worker owns graph work, cancellation and retained
source/view admission. Graph rows remain separate from editable source, history
and independent peer views. Narrow panes retain today and complete glyphs. No
new worker owner, dependency, source database or notification service is added.

Behavior references: [Org habit tracking](https://orgmode.org/manual/Tracking-your-habits.html)
and [Org habit implementation](https://raw.githubusercontent.com/emacs-mirror/emacs/master/lisp/org/org-habit.el).

## Contracts Added or Changed

- ORG20: bounded native consistency graphs and actual completion markers.
- ORG16: typed graph preferences, native controls and semantic backgrounds.

## Files of Interest

- `src/org/agenda/habits/graph/`: daily history reconstruction.
- `src/display/habit.rs`: bounded native pane projection.
- `src/session/habits/`: read-only controls and prompt ownership.
- `tests/org_agenda/habit_views.rs`: source, geometry, time and idle assertions.

## Validation

T25.1: `60428ea7e25156b6ca292a24fd83b28f88be6860`.
T25.2: `78bbf7e8d2c3db3f74366863fb5e951b12a4a56e`.
T25.3: `5f32a30d3fe59bd71d7636f74205fad7eccae829`.

Development checks passed:

- `cargo check`.
- `cargo test --test org_time --test display --test configuration habit --no-fail-fast`:
  fifteen selected graph/configuration/display cases passed, zero skips.
- `cargo test --test org_agenda habit`: native control/refresh journeys passed.
- `cargo test --test commands context_reference`: complete target-set agreement.
- `cargo test --test commands org_`: ten routing/catalog cases, zero skips.
- `cargo test --test org_time --test org_agenda --test display habit --no-fail-fast`:
  twenty-one selected cases passed, zero skips. Includes exact row limits,
  stale request rejection, dirty completion observations, new-day refresh,
  BOM/CRLF source, shared/narrow panes, late state and an isolated NO_COLOR child.
- `python3 scripts/check-config-reference.py`: 136 schema entries, 54 RGB roles,
  191 bindings and 16169 packaged template bytes agree.
- Affected library/test Clippy checks passed with warnings denied.
- Changed-source quality and `git diff --check` passed at original limits.

Development checks corrected native type/visibility and command identities,
Unicode gutter byte assumptions, a restart deadline expectation, a color check
that ignored NO_COLOR, and a default-field initializer. Native cancellation
retires input focus before dispatch; pending prompt identity now prevents C-g
from also leaving the agenda. Documentation groups q/C-g aliases to preserve the
complete-reference assertion. Metadata dispatch, report routing and projection
stages have cohesive owners to retain complexity limits. The row-limit fixture
now checks the actual dated header rather than mistaking its error echo for a
published agenda. No assertion, deadline or resource bound was weakened.

Final controller receipts bind the clean canonical PR head. Required order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_time --test org_agenda --test display --test configuration`
- `python3 scripts/check-source.py`
- `git diff --check`

## Risks and Follow-ups

The delivery-set full gate has not run. Source discovery and the complete
calendar agenda follow their recorded slices.

[Canonical PR #126](https://forge.adrianpk.com/adrianpk/rev/pulls/126) targets dev.
Final required/focused controller receipts bind the clean PR-reference head;
verified integration remains pending.
