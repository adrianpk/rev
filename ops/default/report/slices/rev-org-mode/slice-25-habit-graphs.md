# Slice 25: render native Org habit consistency graphs

Status: drafting
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-25-habit-graphs`
PR: pending

## Purpose

Deliver bounded consistency graphs under the
[approved plan](../../../plan/rev-org-mode.md#slice-25-render-native-org-habit-consistency-graphs)
and [tracker](../../../tracker/rev-org-mode.md#slice-25-render-native-org-habit-consistency-graphs).

## Delivered Behavior

T25.1 supplies bounded graph data and pane-row rendering with not-yet-due,
available, imminent and overdue states. Actual completion uses *, and today uses
! when no completion mark takes precedence. ASCII status glyphs retain meaning
without color. Late completions retain overdue status by default; the explicit
done-always-green preference selects ready status.

## Implementation Notes

Factory preferences match Org: 21 preceding days, 7 following days, column 40,
today-only true, show-all-today false and done-always-green false. Each visible
side admits 0..366 days and the preferred column admits 0..4096. Typed layers,
leaf provenance and complete-candidate rejection cover all preferences and eight
semantic graph backgrounds.

Graph reconstruction uses actual source completion dates and the distinct +,
++ and .+ modes. Org's graph day conversion floors 30.4-day months and 365.25-day
years; this display policy does not replace checked source calendar recurrence.
A graph contains at most 733 cells and admits at most 4096 source completions.
Absent history supplies no completion mark. No source rewrite or clock read
belongs to graph construction. Narrow panes retain today and complete glyphs.

Behavior references: [Org habit tracking](https://orgmode.org/manual/Tracking-your-habits.html)
and [Org habit implementation](https://raw.githubusercontent.com/emacs-mirror/emacs/master/lisp/org/org-habit.el).

## Contracts Added or Changed

- ORG20: bounded native consistency graphs and real completion markers.
- ORG16: typed graph preferences and semantic palette backgrounds.

## Files of Interest

- `src/org/agenda/habits/graph/`: daily history reconstruction.
- `src/display/habit.rs`: bounded pane row rendering.
- `tests/org_time/habit_graphs.rs`: independent date/state expectations.

## Validation

T25.1 development checks passed:

- `cargo check`.
- `cargo test --test org_time --test display --test configuration habit --no-fail-fast`:
  fifteen selected cases, zero skips, including graph modes, late completion,
  exact 733-cell bound, typed configuration and eight actual rendered backgrounds.
- `python3 scripts/check-config-reference.py`: 136 schema entries, 54 RGB roles,
  181 bindings and 15850 packaged template bytes agree.

- `cargo clippy --lib --test org_time --test display --test configuration -- -D warnings`.
- Changed-source quality and `git diff --check` passed at original limits.

Development checks corrected a default-field initializer, an incorrect test's
restart deadline expectation, byte-based slicing of a Unicode gutter and a
palette assertion that ignored the process NO_COLOR setting. Geometry now uses
cell/character boundaries and the existing decoded terminal surface verifies
all backgrounds. No behavior assertion or resource bound was removed.

## Risks and Follow-ups

Native agenda controls and integration checks follow in T25.2/T25.3. The
required final controller validation and delivery-set full gate have not run.


T25.1 commit: `60428ea7e251`.

T25.2 adds C-c C-x h and six compiled habit controls. The current buffer's
read-only agenda supports K visibility, A all-today, C-u K today-only, a selected
view date, typed graph window/column, row scroll and explicit g refresh. Prompts
preserve invalid editable drafts and C-g returns to the owning agenda. Source
view, history and edit authority remain separate from displayed rows.

One selected-buffer snapshot admits 128 graph rows. The existing native report
worker owns graph completion/cancellation and exact source/view admission.
Repaints reuse immutable rows; an external source edit marks the graph stale
until refresh. A today view follows a new captured day only on explicit refresh.
No inactive graph deadline, new worker owner or source database is introduced.

T25.2 development checks passed:

- `cargo test --test org_agenda habit`: two native control/refresh journeys.
- `cargo test --test commands context_reference`: complete target-set agreement.
- `cargo test --test commands org_`: ten routing/catalog cases, zero skips.
- `python3 scripts/check-config-reference.py`: 136 schema entries, 54 RGB roles,
  191 bindings and 16169 packaged template bytes agree.
- `cargo clippy --lib --test org_agenda --test commands -- -D warnings`.
- Changed-source quality and `git diff --check` passed at original limits.

Native checks caught cancellation focus retiring before dispatch: the pending
prompt now identifies its owner so C-g cannot also exit the agenda. The complete
binding-reference assertion identified duplicate q/C-g targets; documentation
now groups those aliases. Native type/visibility and command identities were
corrected during compilation. Metadata dispatch, report routing and projection
stages have separate owners to retain source-complexity limits. No check or
assertion was weakened.
