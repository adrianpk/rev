# Slice 1: Everyday Editor Guide

Status: reviewing
Delivery set: rev-documentation
Plan: `ops/default/plan/rev-documentation.md`
Tracker: `ops/default/tracker/rev-documentation.md`
Branch: `docs/rev-documentation-01-editor`
PR: pending

## Purpose

Let readers learn everyday editing without taking an Org course. Keep Org
available through its own learning entry and preserve the existing examples.

## Delivered Behavior

- The User Guide has six chapters: first file, editing, buffers/panes,
  search/commands, tree and preferences.
- Org has a separate three-chapter path for notes, tasks/agenda and tables.
  Its index links focused procedures and the existing complete control catalog.
- README and all quadrant indexes distinguish those paths. Editor recovery
  stays with ordinary file tasks; Org procedures have their own heading.
- All 53 documentation pages remain reachable from the documentation index.

## Implementation Notes

Existing chapter filenames and URLs remain. The visible sixth editor chapter
uses the original preferences filename; renaming it would break a published
link without helping the reader. Next/Previous links stay within each path.

Every original guide code block and the complete configuration sample are
unchanged. This slice changes navigation and introductory prose, so it reuses
the baseline's native workflow evidence rather than repeating unrelated PTY,
performance or whole-editor checks. T1.1 is commit
`4f8c8dc13d169fbbcb13f2477b5f828b8067ddfb`.

## Contracts Added or Changed

The core learning sequence no longer crosses Org before reaching preferences.
The Org guide states its shared editing prerequisites and returns to the
User Guide index. This does not change the CLI, keybindings, configuration,
packaged sample or application behavior.

## Files of Interest

- `docs/tutorials/user-guide/index.md`: the core chapter order.
- `docs/tutorials/user-guide/org/index.md`: independent Org learning entry.
- `docs/tutorials/user-guide/05-file-tree.md` and `09-preferences.md`:
  the direct tree-to-preferences navigation.
- `README.md` and the four quadrant indexes: separate reader entry points.

## Validation

- `python3 -c 'from pathlib import Path; import runpy; c=runpy.run_path("scripts/check-config-reference.py"); links=[(p,t) for p in [Path("README.md"),*Path("docs").rglob("*.md")] for t in c["link_targets"](p.read_text()) if "://" not in t]; [c["check_link"](p,t) for p,t in links]; print(f"PASS {len(links)} local links and anchors")'` passed: 253 local links and heading anchors.
- An inline check using the existing Markdown parser confirmed reachability
  of all 53 documentation pages from `docs/index.md`.
- Both Next/Previous sequences were checked, including their first/last
  chapters. All existing chapter URLs, guide code blocks and the packaged
  reference sample were compared with the slice base and retained.
- `git diff --check` passed. Changes are limited to documentation and this
  delivery's plan, tracker and report.
- External HTTP links were not checked. No runtime instructions changed.

## Risks and Follow-ups

The shared command, binding and configuration references still contain Org
material. Their extraction and completeness-guard updates belong to Slice 2.
The v0.1.0 changelog and final documentation gate belong to Slice 3. This report
records a review candidate; the slice is not yet integrated into dev.
