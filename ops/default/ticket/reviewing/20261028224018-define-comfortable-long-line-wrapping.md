---
id: TKT-20261028224018
title: Define comfortable long-line wrapping
status: reviewing
kind: task
severity: unclassified
priority: unclassified
scope: ui
tags: wrapping, display, navigation
source: chat
reported_at: 2026-10-28T22:40:18Z
ready_at: 2026-10-30T23:41:32Z
started_at: 2026-10-31T02:43:02Z
reviewed_at: 2026-10-31T02:43:02Z
branch: feat/rev-wrap-navigation-04-org-wrap
pr: https://forge.adrianpk.com/adrianpk/rev/pulls/177
commits: [ba7233b763b514be0c0fb0cf205abd2c4ae4c227, f8e7e974a8863f05f7fee9750eb31001cd9f3a04, ec60f353372f90d7a58627b3848e34ec7b915db8, 7a45df87806500aac2bdcbc52f33ee90447de913, 0fe5431e9ee449986bb56c3aadb669513c941331, 034b721bb110461e7106cdea970564548934d009]
---

# Define comfortable long-line wrapping

## Reported behavior

Writing a long line currently feels awkward. The owner requests a comparison
with VS Code, Neovim and Emacs to clarify word wrap and soft wrap before
settling rev's behavior.

The delivered editor deliberately has no soft wrap. Long logical lines use a
horizontal viewport that follows point. This is recorded in
[display defaults](../../../../docs/reference/defaults.md), the baseline
[editor specification](../../spec/rev-editor.md), and its GUT03 amendment.
There is no existing wrap toggle to enable in the delivered configuration.

## Terms and reference behavior

- Soft wrap changes presentation only: one logical source line occupies
  several display rows without inserting newlines. Word-aware soft wrap
  prefers word boundaries; wrapping exactly at the viewport edge can instead
  divide a word. These are separate display choices, not two file formats.
- Hard wrapping or filling inserts actual newline characters. It changes
  source, save output and undo history and is distinct from visual wrapping.
- VS Code defaults editor.wordWrap to off. Setting it to on wraps at the
  viewport width, and Alt+Z toggles the display behavior. Its additional
  wordWrapColumn and bounded settings do not establish requirements for rev.
- Neovim defaults wrap to on but linebreak to off: screen-edge continuation
  can split words. Enabling linebreak selects permitted word-break points;
  breakindent can indent continuation rows. textwidth is a separate setting
  that can insert real newlines.
- Emacs supports ordinary screen-edge continuation and optional truncation.
  M-x visual-line-mode wraps at word boundaries and changes some line commands
  to operate on visual rows. Auto Fill is separate and inserts real newlines.

## Approved direction

Approved by the product owner on 2026-10-28: native word-aware soft wrapping,
with a visual result comparable to Emacs Visual Line mode, plus an explicit
no-wrap alternative. Wrapping changes presentation without inserting real
newlines. The initial approval covered presentation; the subsequent decisions
below also adopt the selected Emacs Visual Line command behavior.
The owner subsequently approved word-aware wrapping enabled by default at the
pane's available text width, with TOML and M-x controls to disable it. Ordinary
long text should stay readable without manual activation or routine horizontal
scrolling. The owner reaffirmed maximum Emacs similarity in keybindings and
behavior, accepting Visual Line semantics: Up/Down and C-n/C-p move by visual
rows; C-a/C-e address the visible beginning/end; C-k follows kill-visual-line.
Structured regions, override lifetime and exact configuration/command spelling
remain to be finalized.

Use rev's existing configuration and native command model where appropriate.
Do not introduce an addon, external formatter or plugin requirement. An
automatic hard-wrap/fill feature is a separate product decision; it must not
be added implicitly to solve this display problem.

## Accepted delivery contract

The accepted specification fixes editor.soft_wrap = true, pane-width breaks,
M-x visual-line-mode with buffer-local lifetime across all views, Visual Line
motion/editing, grapheme-safe long-token continuation and source-preserving Org
wrapping without implicit table/source-block no-wrap exceptions. Gutter labels
and rectangle rows/columns retain logical source identities. The implementation
must enforce the specified bounded cache and record bounded work measurements.

This ticket is under review. Slices 1-3 delivered shared display/input
behavior; Slice 4 owns full wrap acceptance and the ticket lifecycle in the single
[Wrap and Navigation delivery](../../plan/rev-wrap-navigation.md).

## Validation direction and delivery boundary

Use source-preserving wrapping examples, resize/split cases, Unicode/tab
boundaries, logical numbering and representative keyboard/pointer journeys.
Enabling or disabling visual wrap alone must not change document bytes, dirty
state or undo history. An explicit future fill operation would require its own
source-edit contract.

This ticket records the accepted wrapping and Emacs interaction contract. The [Word-Aware Soft Wrapping specification](../../spec/editor-soft-wrap.md)
and its model now capture the coordinated proposal. Reconcile the delivered
no-wrap contracts during implementation under the approved plan.
The approved eight-slice delivery records the selected five interaction
topics, with soft wrap owned by Slices 1-4.

## References

- [VS Code word wrap](https://code.visualstudio.com/docs/editing/codebasics#_word-wrap).
- [Neovim wrap, linebreak, breakindent and textwidth](https://neovim.io/doc/user/options/).
- [Emacs continuation lines](https://www.gnu.org/software/emacs/manual/html_node/emacs/Continuation-Lines.html).
- [Emacs Visual Line mode](https://www.gnu.org/software/emacs/manual/html_node/emacs/Visual-Line-Mode.html).

## Implemented resolution

Slices 1-4 implement the accepted native word-aware wrapping, buffer-local
Visual Line controls, source-preserving Org behavior and bounded derived work.
The [Slice 4 report](../../report/slices/rev-wrap-navigation/slice-4-org-wrap.md)
records cache/work limits, exact source/undo checks, cold input cancellation,
resize/reversal measurements and real PTY idle CPU evidence. Canonical
[PR #177](https://forge.adrianpk.com/adrianpk/rev/pulls/177) targets `dev`.
Controller receipt `JNLPDE7N5VZQHLQ33J3U6447EA` passed all eleven configured
and focused commands at `034b721bb110461e7106cdea970564548934d009`, including
942 tests, all-target Clippy and native/PTY measurements. The final PR-reference
head receives a fresh configured controller receipt before handoff.
This ticket moved through implementation and is now under review; integration
and the solved transition remain owned by the controller delivery workflow.
