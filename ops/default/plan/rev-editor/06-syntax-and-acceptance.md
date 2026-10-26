# rev Editor: Syntax and acceptance

Plan: [rev Editor](../rev-editor.md)
Specification: [rev Editor](../../spec/rev-editor.md)
Model: [rev Editor Model](../../spec/rev-editor-model.md)

Read the master plan, its pinned decisions, and the current section before work.
This file supplies detailed tasks; the master plan/tracker own slice identities.
Every task includes appropriate behavior tests. Each slice also requires the
common quality gate, one canonical report, one PR, and controller-owned merge.

## Slice 41: Update syntax from exact rope edits

Requirements: R07, R09, R32, R40
Dependencies: Slice 19, Slice 26, Slice 34, Slice 40
Owners: syntax revision tracking and incremental parse
Focused validation: `cargo test --test syntax`

### T41.1

Translate every accepted edit/undo/redo/rectangle/paste to correct Tree-sitter byte and
row/byte-column edits, retaining parser/tree state and bounding visible capture work.

### T41.2

Exercise multibyte edits before captures, shared windows, parse/query failures,
selection/search/cursor layering, and stale syntax invalidation without whole-file
rebuild per key.

### Required evidence

Parser failure retains the accepted text and coherent history; old captures cannot color
unrelated current ranges.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-41-incremental-syntax.md`. Update the tracker after integration.

## Slice 42: Complete guarded multi-buffer session exit

Requirements: R03, R04, R11, R13, R21, R24
Dependencies: Slice 15, Slice 16, Slice 28, Slice 39, Slice 41
Owners: save/discard/cancel lifecycle coordinator
Focused validation: `cargo test --test recovery`

### T42.1

Complete save-buffers-kill-terminal across all modified named and unnamed buffers,
including sequential save decisions, canceled path prompts, read-only and
uncertain-publication outcomes.

### T42.2

Exercise cancellation after earlier saves, failed required saves, several windows of one
buffer, a removed destination, and final terminal restoration after a genuinely accepted
quit.

### Required evidence

Cancel preserves the live session; a failed/uncertain required save cannot become a
silent discard or falsely completed exit.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-42-session-quit.md`. Update the tracker after integration.

## Slice 43: Harden ordinary failure boundaries

Requirements: R03, R04, R06, R10, R13, R20, R25, R30, R32, R37, R39
Dependencies: Slice 20, Slice 31, Slice 34, Slice 39, Slice 41, Slice 42
Owners: filesystem/terminal/syntax fault boundaries
Focused validation: `cargo test --test recovery`

### T43.1

Extend real boundary fault injection for recoverable ingress, save stages, tree
operations, parser/query execution, and post-acquisition terminal errors, retaining
truthful outcomes.

### T43.2

Exercise combined failures and cleanup ordering, canceled transient interactions,
readonly rejection, and state equality after rejected compound edits. Keep fixes within
already approved behavior.

### Required evidence

Ordinary failures preserve data and usable state; failures after irreversible filesystem
effects report those effects rather than falsely claiming rollback.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-43-failure-recovery.md`. Update the tracker after integration.

## Slice 44: Drive user journeys through a PTY

Requirements: R01, R03, R04, R12, R14, R15, R18, R19, R20, R22, R25, R26, R27, R28, R31, R39
Dependencies: Slice 43
Owners: PTY acceptance harness and real executable
Focused validation: `python3 scripts/check-pty.py --suite acceptance`

### T44.1

Extend the existing standard-library Python PTY harness to drive
launch/edit/save/reopen, bindings and M-x, buffer/window operations,
selection/rings/rectangles, search, tree actions, and quit.

### T44.2

Use bounded condition waits, owned fixtures, real filesystem byte comparisons, resize
events, and termios/screen restoration checks. Separate pure-state assertions from
terminal encoding variability.

### Required evidence

The real executable performs representative required journeys without debug commands;
unsupported chord claims are documented rather than silently skipped.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-44-terminal-journeys.md`. Update the tracker after integration.

## Slice 45: Exercise cross-feature Unicode and file fidelity

Requirements: R07, R09, R10, R19, R20, R21, R26, R30, R32, R39
Dependencies: Slice 44
Owners: shared behavior regressions and boundary fixtures
Focused validation: `cargo test --all`

### T45.1

Add distinct adversarial regressions across editing/history/search/syntax and file
save/reopen using emoji, combining text, CJK, tabs, CRLF, BOM, absent final newline,
long lines, and shared views.

### T45.2

Exercise repeated split/close/resize, invalid yank-pop, rectangle seams, symlink loops,
inaccessible paths, and canceled destructive actions. Reuse subsystem suites rather than
duplicate every journey.

### Required evidence

Byte fidelity, live positions, history state, syntax coordinates, and safe failures
remain consistent after combined workflows.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-45-adversarial-text.md`. Update the tracker after integration.

## Slice 46: Measure representative editing workloads

Requirements: R07, R08, R26, R32, R40
Dependencies: Slice 41, Slice 45
Owners: performance fixture and measurement script
Focused validation: `python3 scripts/check-performance.py`

### T46.1

Add a deterministic mixed-width UTF-8 fixture generator with tens of thousands of lines
and measure startup, insert/delete, scroll, split redraw, and highlighting on recorded
hardware context.

### T46.2

Inspect/measure work for pathological whole-file copying/reparse and idle repaint loops,
correcting only demonstrated in-scope causes. Store concise fixture/measurement evidence
without hostnames or invented guarantees.

### Required evidence

The measurement command executes every required workload and fails on missing evidence
or demonstrated pathological behavior; numeric claims remain limited to actual
measurements.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-46-performance.md`. Update the tracker after integration.

## Slice 47: Audit source quality and product reference

Requirements: R01, R22, R25, R33, R34, R35, R36, R37, R38
Dependencies: Slice 45, Slice 46
Owners: source-quality audit and product documentation
Focused validation: `python3 scripts/check-source.py`

### T47.1

Review actual production functions for cyclomatic paths, at most five parameters
including self, shallow nesting, concise names, restrained comments, ownership,
dependency necessity, and complete real command catalog.

### T47.2

Finish README usage plus concise Diataxis reference/how-to material for bindings, scope,
terminal limitations, file/undo defaults, and recovery. Keep Unreleased changelog
outcomes accurate and complete.

### Required evidence

Automated checks and explicit semantic review agree; no large dispatcher, boilerplate
test noise, undocumented terminal requirement, or functional-looking command stub
remains.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-47-quality-reference.md`. Update the tracker after integration.

## Slice 48: Verify the complete integrated editor contract

Requirements: R01, R02, R03, R04, R05, R06, R07, R08, R09, R10, R11, R12, R13, R14, R15, R16, R17, R18, R19, R20, R21, R22, R23, R24, R25, R26, R27, R28, R29, R30, R31, R32, R33, R34, R35, R36, R37, R38, R39, R40
Dependencies: Slice 47
Owners: acceptance manifest and final-gate prerequisites
Focused validation: `python3 scripts/check-pty.py --suite acceptance`

### T48.1

Complete a reviewed A01-A20 evidence map linking the real regression/PTY/performance
checks, close any remaining within-scope integration defects, and make
scripts/check-all.sh invoke the exact mandatory gate without skips.

### T48.2

Exercise the assembled user journeys through the built executable and verify all gate
prerequisites, reports, required commands, and unchanged identities. Leave primary dev
clean and pushed for controller-owned final validation.

### Required evidence

The controller runs the frozen full gate against the exact integrated candidate; closure
follows only a successful gate and every delivered report, with no unchecked tracker
tasks.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-48-acceptance.md`. Update the tracker after integration.
