# Org commands

These names use the same [M-x matching and invocation](../commands.md#matching-and-invocation)
and handlers as the [editor command catalog](../commands.md#implemented-catalog).
Availability depends on the current Org source, agenda, calendar or draft context;
see [Org bindings](bindings.md) and the [capability map](../org-controls.md).

## Implemented catalog

| Group | Names |
| --- | --- |
| Native mode | `org-mode`, `text-mode` |
| Org visibility | `org-cycle`, `org-shifttab`, `org-overview`, `org-content`, `org-show-all`, `org-hide-subtree`, `org-show-children`, `org-show-subtree`, `org-hide-drawer`, `org-show-drawer`, `org-toggle-drawer` |
| Org estimates | `org-priority`, `org-priority-up`, `org-priority-down`, `org-inspect-priority`, `org-set-effort`, `org-inc-effort`, `org-inspect-effort` |
| Org timestamps | `org-timestamp`, `org-timestamp-inactive`, `org-schedule`, `org-deadline`, `org-goto-calendar`, `org-date-from-calendar` |
| Org query agendas | `org-todo-list`, `org-search-view`, `org-tags-view` |
| Org calendar agendas | `org-agenda`, `org-agenda-day-view`, `org-agenda-week-view`, `org-agenda-later`, `org-agenda-earlier`, `org-agenda-goto-today`, `org-agenda-redo`, `org-agenda-switch-to`, `org-agenda-goto-date` |
| Org habit graphs | `org-habit-show-graph`, `org-habit-toggle-habits`, `org-habit-toggle-habits-only-for-today`, `org-habit-toggle-show-all-today`, `org-habit-graph-date`, `org-habit-set-graph-window` |
| Org timers | `org-timer-start`, `org-timer-set-timer`, `org-timer`, `org-timer-item`, `org-timer-pause-or-continue`, `org-timer-stop`, `org-timer-show-remaining-time` |
| Org task clock | `org-clock-in`, `org-clock-out`, `org-clock-cancel`, `org-clock-goto`, `org-resolve-clocks`, `org-clock-in-last`, `org-clock-display`, `org-clock-report`, `org-dblock-update` |
| Org timestamp adjustment | `org-timestamp-up`, `org-timestamp-down`, `org-timestamp-up-day`, `org-timestamp-down-day`, `org-shiftup`, `org-shiftdown`, `org-shiftleft`, `org-shiftright` |
| Org calendar | `org-calendar-select`, `org-calendar-forward-day`, `org-calendar-backward-day`, `org-calendar-forward-week`, `org-calendar-backward-week`, `org-calendar-forward-month`, `org-calendar-backward-month`, `org-calendar-forward-year`, `org-calendar-backward-year`, `org-calendar-scroll-three-months-left`, `org-calendar-scroll-three-months-right`, `org-calendar-goto-today` |
| Org tasks | `org-todo`, `org-todo-next`, `org-todo-previous`, `org-todo-set`, `org-inspect-todo` |
| Org tags | `org-set-tags-command`, `org-remove-tag`, `org-inspect-tags`, `org-match-tags` |
| Org properties | `org-set-property`, `org-delete-property`, `org-inspect-property`, `org-edit-property-owner` |
| Org headings | `org-insert-heading`, `org-meta-return`, `org-next-visible-heading`, `org-previous-visible-heading`, `org-forward-heading-same-level`, `org-backward-heading-same-level`, `org-up-heading-safe` |
| Org list context | `org-metaleft`, `org-metaright`, `org-metaup`, `org-metadown` |
| Org checkboxes | `org-ctrl-c-ctrl-c`, `org-toggle-checkbox`, `org-toggle-radio-button`, `org-toggle-checkbox-presence`, `org-update-statistics-cookies`, `org-insert-item-checkbox` |
| Org lists | `org-insert-item`, `org-indent-item`, `org-outdent-item`, `org-indent-item-tree`, `org-outdent-item-tree`, `org-move-item-up`, `org-move-item-down`, `org-renumber-list` |
| Org subtrees | `org-promote-subtree`, `org-demote-subtree`, `org-move-subtree-up`, `org-move-subtree-down` |
| Org configured agendas | `org-agenda-custom-view`, `org-agenda-filter`, `org-agenda-filter-remove-all`, `org-agenda-sort`, `org-agenda-columns`, `org-agenda-month-view`, `org-agenda-year-view`, `org-agenda-span` |
| Org refile and copy | `org-refile`, `org-refile-copy`, `org-refile-results` |
| Org capture drafts | `org-capture`, `org-capture-kill`, `org-capture-finalize` |
| Org agenda source actions | `org-agenda-bulk-mark`, `org-agenda-bulk-unmark`, `org-agenda-bulk-mark-all`, `org-agenda-bulk-unmark-all`, `org-agenda-bulk-action`, `org-agenda-action-results`, `org-agenda-undo` |
| Org archive | `org-archive-subtree`, `org-toggle-archive-tag`, `org-archive-to-archive-sibling`, `org-archive-results` |
| Org links | `org-store-link`, `org-insert-link`, `org-open-at-point`, `org-mark-ring-goto`, `org-mark-ring-push`, `org-next-link`, `org-previous-link` |
| Org attachments | `org-attach`, `org-attach-attach`, `org-attach-copy`, `org-attach-move`, `org-attach-link`, `org-attach-open`, `org-attach-open-in-emacs`, `org-attach-reveal-in-emacs`, `org-attach-set-directory`, `org-attach-results` |
| Org source blocks | `org-edit-special`, `org-edit-src-exit`, `org-edit-src-abort`, `org-babel-view-src-block-info`, `org-babel-insert-header-arg` |
| Org previews | `org-link-preview`, `org-link-preview-clear`, `org-link-preview-refresh`, `org-link-preview-region` |
| Org feeds | `org-feed-update-all`, `org-feed-update`, `org-feed-goto-inbox`, `org-feed-results` |
| Org execution | `org-babel-execute-src-block`, `org-babel-execution-results` |
| Org tangling | `org-babel-tangle`, `org-babel-tangle-single-block`, `org-babel-tangle-results` |
| Org publishing | `org-publish`, `org-publish-results` |
| Org exports | `org-export-to-text`, `org-export-to-unicode`, `org-export-to-markdown`, `org-export-to-org`, `org-export-to-html`, `org-export-to-latex`, `org-export-to-beamer`, `org-export-to-latex-pdf`, `org-export-to-beamer-pdf`, `org-export-to-odt`, `org-export-to-texinfo`, `org-export-to-man`, `org-export-to-icalendar` |
| Org document editing | `org-footnote-action`, `org-footnote-new`, `org-footnote-renumber`, `org-emphasize`, `org-insert-entity`, `org-edit-affiliated-keyword` |
| Org tables | `org-table-create`, `org-table-align`, `org-table-next-field`, `org-table-previous-field`, `org-table-next-row`, `org-table-beginning-of-field`, `org-table-end-of-field`, `org-table-insert-row`, `org-table-insert-hline`, `org-return` |
| Org table structure | `org-table-kill-row`, `org-table-move-row-up`, `org-table-move-row-down`, `org-table-insert-column`, `org-table-delete-column`, `org-table-move-column-left`, `org-table-move-column-right`, `org-table-sort-lines`, `org-list-to-table`, `org-table-to-list` |
| Org property columns | `org-columns`, `org-columns-insert-dblock` |
| Org table interchange | `org-table-import`, `org-table-export` |
| Org recalculation | `org-table-recalculate`, `org-table-recalculate-cell`, `org-table-recalculate-column` |

## Native task workflows

C-c C-t (`org-todo`) cycles the current heading through its declared sequence,
including the unmarked state. Declarations with fast selection keys instead
open a state prompt. `org-todo-set` always prompts: enter a full keyword, its
selection key, or `-`/empty for unmarked. `org-todo-next` and
`org-todo-previous` cycle without fast selection; `org-inspect-todo` reports
current state and the admitted choices without editing.

TODO, SEQ_TODO and TYP_TODO declarations are data. Each declaration defines a
separate sequence, `|` begins finished states, and without `|` the last state
is finished. Keywords must be unique across the file (at most 64). Annotations
such as `WAIT(w@/!)` select a key, entry note and departure timestamp. Target
entry logging takes precedence over departure logging. Unknown leading words
remain title text. Malformed declarations reject task editing and matching.

A transition preserves priorities, tags, planning, properties and unrelated
bytes. Selected CLOSED timestamps, state logs/one-line notes and the immediate
parent's task progress cookies publish in the same transaction. Empty notes
record time only; C-g cancels the entire transition. Undo restores all its
metadata, including shared-view anchors. Read-only buffers admit inspection
and reject transitions before opening an edit prompt.

LOGGING and LOG_INTO_DRAWER inherit explicitly from source ancestors/file
properties. A nonempty LOGGING value resets defaults and annotations; `nil`,
`logdone`, `lognotedone`, and declared keyword annotations are supported.
LOG_INTO_DRAWER accepts `nil`, `t` (LOGBOOK) or an explicit drawer name.
Task timestamps capture the local operation time once, before a note prompt.
The session reads at most 1 MiB of TZif data; UTC or a TZif name/path in TZ is
supported. Unsupported or uncovered local time rejects before mutation.

[Org reference](index.md)
