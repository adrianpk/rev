# Org bindings

These keys share the [terminal conventions](../bindings.md) and
[M-x invocation](../commands.md#matching-and-invocation) used by everyday editing.
Entries without an Org-only context remain available from ordinary editing;
source, agenda, calendar and draft actions require their stated owner.

| Keys | Command | Result |
| --- | --- | --- |
| `C-c C-x C-a` in Org | `org-archive-subtree` | Move the complete subtree to the checked archive location with origin metadata. |
| `C-c C-x a` in Org | `org-toggle-archive-tag` | Toggle the local ARCHIVE tag without removing source. |
| `C-c C-x A` in Org | `org-archive-to-archive-sibling` | Move the entry below its checked Archive sibling, creating it when absent. |
| M-x | `org-archive-results` | Inspect archive participants and independent save outcomes. |
| `C-c l` | `org-store-link` | Store the checked current location for later insertion. |
| `C-c C-l` in Org | `org-insert-link` | Insert or edit a literal link and description. |
| `C-c C-a` in Org | `org-attach` | Dispatch explicit heading attachment operations. |
| M-x | `org-attach-attach`, `org-attach-copy`, `org-attach-move`, `org-attach-link` | Confirm selected file methods and destinations. |
| M-x | `org-attach-open`, `org-attach-open-in-emacs`, `org-attach-reveal-in-emacs` | Open or list checked heading attachments. |
| M-x | `org-attach-set-directory`, `org-attach-results` | Set DIR without creating folders, or inspect retained outcomes. |
| `C-c C-x f` in Org | `org-footnote-action` | Follow or create a checked footnote. |
| `C-c C-x C-f` in Org | `org-emphasize` | Insert or change source emphasis. |
| M-x | `org-footnote-new`, `org-footnote-renumber`, `org-insert-entity`, `org-edit-affiliated-keyword` | Create notes, renumber numeric labels and edit literal inline or affiliated source. |
| `C-c C-o` in Org | `org-open-at-point` | Resolve and visit the exact link at point. |
| `C-c &` / `C-c %` in Org | `org-mark-ring-goto` / `org-mark-ring-push` | Return to a checked source point or record one. |
| `C-c C-x C-n` / `C-c C-x C-p` in Org | `org-next-link` / `org-previous-link` | Move between recognized links; repeat at the boundary to wrap. |
| `C-c C-w` in Org | `org-refile` | Select a checked heading and move complete subtrees into its canonical buffer. |
| `C-c M-w` in Org | `org-refile-copy` | Copy complete subtrees without removing the source. |
| M-x | `org-refile-results` | Inspect each participant and its ordinary save outcome. |
| `C-c c` | `org-capture` | Select a template and prepare an editable transient draft. |
| `C-c C-c` in capture | `org-capture-finalize` | Revalidate, insert and save once; retain the draft on failure. |
| `C-c C-k` in capture | `org-capture-kill` | Discard the draft and return to its mapped origin view. |
| `C-c .` in Org | `org-timestamp` | Insert or edit an active timestamp through a native date/calendar prompt. |
| `C-c !` in Org | `org-timestamp-inactive` | Insert or edit an inactive timestamp. |
| `C-c C-s` in Org | `org-schedule` | Set the current heading's SCHEDULED timestamp. |
| `C-c C-d` in Org | `org-deadline` | Set the current heading's DEADLINE timestamp. |
| `C-c >` in Org | `org-goto-calendar` | Inspect the calendar and retain a selected date. |
| `C-c <` in Org | `org-date-from-calendar` | Insert the previously selected calendar date. |
| `S-Left` in Org | `org-shiftleft` | Move a timestamp back one day, or select the previous heading task state. |
| `S-Right` in Org | `org-shiftright` | Move a timestamp forward one day, or select the next heading task state. |
| M-x org-timestamp-up | `org-timestamp-up` | Increase the selected timestamp field. |
| M-x org-timestamp-down | `org-timestamp-down` | Decrease the selected timestamp field. |
| M-x org-timestamp-up-day | `org-timestamp-up-day` | Move the selected timestamp forward one civil day. |
| M-x org-timestamp-down-day | `org-timestamp-down-day` | Move the selected timestamp back one civil day. |
| M-x org-priority-up | `org-priority-up` | Cycle the current heading priority upward. |
| M-x org-priority-down | `org-priority-down` | Cycle the current heading priority downward. |
| Return in date prompt | `org-calendar-select` | Accept the interpreted date/time with any required offset choice. |
| S-Right in date prompt | `org-calendar-forward-day` | Select the next day. |
| S-Left in date prompt | `org-calendar-backward-day` | Select the previous day. |
| S-Down in date prompt | `org-calendar-forward-week` | Select one week later. |
| S-Up in date prompt | `org-calendar-backward-week` | Select one week earlier. |
| M-S-Right or > in date prompt | `org-calendar-forward-month` | Select one month later, clamping its day. |
| M-S-Left or < in date prompt | `org-calendar-backward-month` | Select one month earlier, clamping its day. |
| M-S-Down in date prompt | `org-calendar-forward-year` | Select one year later, clamping its day. |
| M-S-Up in date prompt | `org-calendar-backward-year` | Select one year earlier, clamping its day. |
| C-v in date prompt | `org-calendar-scroll-three-months-left` | Advance by three months. |
| M-v in date prompt | `org-calendar-scroll-three-months-right` | Move back by three months. |
| C-. in date prompt | `org-calendar-goto-today` | Select the operation's captured current date. |
| `C-c C-x p` in Org | `org-set-property` | Set a local property through name/value prompts. |
| `M-x org-delete-property` | `org-delete-property` | Remove the named local base or additive form. |
| `M-x org-inspect-property` | `org-inspect-property` | Inspect effective data and source owners. |
| `M-x org-edit-property-owner` | `org-edit-property-owner` | Explicitly edit a selected effective source owner. |
| `C-c ,` in Org | `org-priority` | Set a priority character; empty/space removes the cookie. |
| `S-Up` in Org | `org-shiftup` | Increase the timestamp field at point, or cycle heading priority. |
| `S-Down` in Org | `org-shiftdown` | Decrease the timestamp field at point, or cycle heading priority. |
| `M-x org-inspect-priority` | `org-inspect-priority` | Show explicit or implicit default priority. |
| `C-c C-x e` in Org, `e` in agenda role | `org-set-effort` | Store a validated Org duration in local EFFORT. |
| `M-x org-inc-effort` | `org-inc-effort` | Cycle declared EFFORT_ALL choices. |
| `M-x org-inspect-effort` | `org-inspect-effort` | Show original effort spelling and elapsed minutes. |
| `C-c C-t` in Org | `org-todo` | Cycle task state or select an annotated fast key. |
| `M-x org-todo-next` | `org-todo-next` | Next state in the current sequence, including unmarked. |
| `M-x org-todo-previous` | `org-todo-previous` | Previous state in the current sequence, including unmarked. |
| `M-x org-todo-set` | `org-todo-set` | Select an admitted state or clear it. |
| `M-x org-inspect-todo` | `org-inspect-todo` | Inspect task state and choices without editing. |
| `C-c C-q` in Org | `org-set-tags-command` | Replace locally stored tags; empty clears them. |
| `M-x org-remove-tag` | `org-remove-tag` | Remove exactly one local, case-sensitive tag. |
| `M-x org-inspect-tags` | `org-inspect-tags` | Inspect local, ancestor and file memberships with source lines. |
| `M-x org-match-tags` | `org-match-tags` | Test a bounded tag/property expression against the current heading. |
| M-x org-mode | `org-mode` | Select native Org for the current buffer. |
| TAB in Org | `org-cycle` | Cycle heading or drawer visibility in native Org. |
| S-TAB in Org | `org-shifttab` | Cycle overview, contents and show-all in this view. |
| M-x org-overview | `org-overview` | Show top-level headings. |
| M-x org-content | `org-content` | Show every heading without section bodies. |
| M-x org-show-all | `org-show-all` | Show all source in this view. |
| M-x org-hide-subtree | `org-hide-subtree` | Hide the containing heading body and descendants. |
| M-x org-show-children | `org-show-children` | Show direct child headings. |
| M-x org-show-subtree | `org-show-subtree` | Show the entire containing subtree. |
| M-x org-hide-drawer | `org-hide-drawer` | Hide only the current drawer body. |
| M-x org-show-drawer | `org-show-drawer` | Show the current drawer body. |
| M-x org-toggle-drawer | `org-toggle-drawer` | Toggle only the current drawer body. |
| M-x org-insert-heading | `org-insert-heading` | Insert a heading at the current level through one checked edit. |
| M-RET in Org | `org-meta-return` | Insert a heading or continue the precise current list item. |
| M-S-Left in Org | `org-outdent-item-tree` | Outdent the whole item and its nested source. |
| M-S-Right in Org | `org-indent-item-tree` | Indent the whole item and its nested source. |
| M-S-RET in Org | `org-insert-item-checkbox` | Insert a new unchecked list item. |
| C-c C-x C-d in Org | `org-clock-display` | Compare task/subtree/file clock totals and effort. |
| C-c C-x C-c in Org | `org-clock-report` | Insert or explicitly update a clocktable. |
| C-c C-x C-u in Org | `org-dblock-update` | Refresh a recognized clocktable or columnview body. |
| C-c C-x 0 in Org | `org-timer-start` | Start or reset the relative timer. |
| C-c C-x h in Org | `org-habit-show-graph` | Open or refresh the native habit agenda for this buffer. |
| K in habit agenda | `org-habit-toggle-habits` | Toggle habit graph visibility. |
| C-u K in habit agenda | `org-habit-toggle-habits-only-for-today` | Toggle graphs only for today. |
| A in habit agenda | `org-habit-toggle-show-all-today` | Include all habits on today's agenda. |
| d in habit agenda | `org-habit-graph-date` | Select a view date without editing source. |
| G in habit agenda | `org-habit-set-graph-window` | Set graph days and preferred column. |
| C-c C-x ; in Org | `org-timer-set-timer` | Enter an explicit countdown. |
| C-c C-x . in Org | `org-timer` | Insert the current timer value. |
| C-c C-x - in Org | `org-timer-item` | Insert or continue a timed description list. |
| C-c C-x , in Org | `org-timer-pause-or-continue` | Pause or continue the session timer. |
| C-c C-x _ in Org | `org-timer-stop` | Stop and discard the session timer. |
| C-c a | `org-agenda` | Select and observe bounded canonical agenda sources. |
| M-x | `org-timer-show-remaining-time` | Inspect the session timer without source changes. |
| C-c C-x C-i in Org | `org-clock-in` | Start a task-owned source interval. |
| C-c C-x C-o in Org | `org-clock-out` | Close the active interval with elapsed time. |
| C-c C-x C-q in Org | `org-clock-cancel` | Remove the active interval. |
| C-c C-x C-x in Org | `org-clock-in-last` | Re-clock the most recent qualified task. |
| C-c C-x C-z in Org | `org-resolve-clocks` | Explicitly resolve incomplete intervals in the selected entry. |
| C-c C-x C-j in Org | `org-clock-goto` | Reveal the active task heading. |
| C-c C-c in Org | `org-ctrl-c-ctrl-c` | Apply a selected TBLFM line, align table data, normalize a timestamp weekday, edit heading tags, or toggle a precise list checkbox and verify numbering. |
| C-c C-x C-b in Org | `org-toggle-checkbox` | Toggle checked source and update progress. |
| C-c C-x C-r in Org | `org-toggle-radio-button` | Select a radio checkbox at the current level. |
| C-c # in Org | `org-update-statistics-cookies` | Update current item/heading statistics. |
| C-c C-n in Org | `org-next-visible-heading` | Visit the next visible heading. |
| C-c C-p in Org | `org-previous-visible-heading` | Visit the previous visible heading. |
| C-c C-f in Org | `org-forward-heading-same-level` | Visit the next sibling without crossing a parent boundary. |
| C-c C-b in Org | `org-backward-heading-same-level` | Visit the previous sibling without crossing a parent boundary. |
| C-c C-u in Org | `org-up-heading-safe` | Visit the parent heading. |
| M-Left in Org | `org-metaleft` | Promote a subtree or outdent the precise list item. |
| M-Right in Org | `org-metaright` | Demote a subtree or indent the precise list item. |
| M-Up in Org | `org-metaup` | Move a complete subtree or list item before its previous sibling. |
| M-Down in Org | `org-metadown` | Move a complete subtree or list item after its next sibling. |
| M-x org-promote-subtree | `org-promote-subtree` | Promote a complete heading subtree. |
| M-x org-demote-subtree | `org-demote-subtree` | Demote a complete heading subtree. |
| M-x org-move-subtree-up | `org-move-subtree-up` | Move a complete heading subtree before its previous sibling. |
| M-x org-move-subtree-down | `org-move-subtree-down` | Move a complete heading subtree after its next sibling. |
| M-x org-insert-item | `org-insert-item` | Split or continue an admitted list item. |
| M-x org-indent-item | `org-indent-item` | Indent the item while retaining child indentation. |
| M-x org-outdent-item | `org-outdent-item` | Outdent an admitted item without detaching children. |
| M-x org-move-item-up | `org-move-item-up` | Move complete item source before its previous sibling. |
| M-x org-move-item-down | `org-move-item-down` | Move complete item source after its next sibling. |
| M-x org-renumber-list | `org-renumber-list` | Renumber ordered siblings with retained explicit resets. |
| M-x org-toggle-checkbox-presence | `org-toggle-checkbox-presence` | Add or remove checkbox source explicitly. |
| M-x text-mode | `text-mode` | Select ordinary text editing. |
| v t in agenda / M-x org-todo-list | `org-todo-list` | Select unfinished tasks or explicit states. |
| s in agenda / M-x org-search-view | `org-search-view` | Search entry text with phrases or signed terms. |
| M in agenda / M-x org-tags-view | `org-tags-view` | Select Boolean tag/property matches. |
| g in calendar/habit agenda | `org-agenda-redo` | Refresh current sources and captured time. |
| v d in calendar agenda | `org-agenda-day-view` | Show one captured day. |
| v w in calendar agenda | `org-agenda-week-view` | Show one configured week. |
| f / Right in calendar agenda | `org-agenda-later` | Move a span forward. |
| b / Left in calendar agenda | `org-agenda-earlier` | Move a span backward. |
| . in calendar agenda | `org-agenda-goto-today` | Show captured effective today. |
| j in calendar agenda | `org-agenda-goto-date` | Choose an explicit date. |
| RET in calendar agenda | `org-agenda-switch-to` | Check and visit selected source. |
| v v in agenda / M-x org-agenda-custom-view | `org-agenda-custom-view` | Select a typed configured block view by key. |
| / in agenda | `org-agenda-filter` | Apply a native tag/property visibility filter. |
| Pipe in agenda | `org-agenda-filter-remove-all` | Restore configured preset filters. |
| S in agenda | `org-agenda-sort` | Set ordered native sort keys. |
| V in agenda | `org-agenda-columns` | Set observed property columns. |
| v m in agenda | `org-agenda-month-view` | Show an aligned calendar month. |
| v y in agenda | `org-agenda-year-view` | Show an aligned calendar year. |
| v s in agenda | `org-agenda-span` | Select a named span or 1–366 days. |
| m in agenda | `org-agenda-bulk-mark` | Mark one qualified source entry. |
| u in agenda | `org-agenda-bulk-unmark` | Unmark the selected source entry. |
| Asterisk in agenda | `org-agenda-bulk-mark-all` | Mark all visible entries, up to 256 distinct targets. |
| U in agenda | `org-agenda-bulk-unmark-all` | Clear explicit marks. |
| B in agenda | `org-agenda-bulk-action` | Select a checked bulk action and common value. |
| M-x org-agenda-action-results | `org-agenda-action-results` | Inspect every retained target outcome. |
| C-_ in agenda | `org-agenda-undo` | Undo the last unchanged source action transaction. |
| Create table in Org | `org-table-create` | Create a bounded native table on a blank line. |
| `RET` in Org | `org-return` | Move to the next table row or insert an ordinary newline in other source contexts. |
| `C-c -` in Org | `org-table-insert-hline` | Insert a table separator below the current row. |
| M-x | `org-table-align`, `org-table-next-field`, `org-table-previous-field`, `org-table-next-row` | Align the selected table or navigate fields and rows. |
| M-x | `org-table-beginning-of-field`, `org-table-end-of-field` | Visit field edges. |
| M-Up / M-Down in a native table | `org-table-move-row-up`, `org-table-move-row-down` | Move the current data row or separator. |
| M-Left / M-Right in a native table | `org-table-move-column-left`, `org-table-move-column-right` | Move the selected column in every data row. |
| M-S-Left / M-S-Right in a native table | `org-table-delete-column`, `org-table-insert-column` | Remove the column or insert an empty column before it. |
| M-S-Up in Org | `org-table-kill-row` | Remove the selected table row or separator. |
| M-S-Down in Org | `org-table-insert-row` | Insert a row above the selected table row. |
| C-c ^ in Org | `org-table-sort-lines` | Sort the selected data rows by a field. |
| C-c * in Org | `org-table-recalculate` | Recalculate the selected table explicitly, as one undo unit. |
| M-x | `org-table-recalculate-cell`, `org-table-recalculate-column` | Recalculate only the selected field or column. |
| M-x | `org-list-to-table`, `org-table-to-list` | Convert contiguous single-line items and one-column tables without separators. |
| M-x | `org-table-import`, `org-table-export` | Read or write checked CSV/TSV table data; existing output files require confirmation. |
| C-c ' / C-c C-c / C-c C-k in source draft | `org-edit-src-exit`, `org-edit-src-abort` | Return the checked source draft or discard it. |
| C-c ' | `org-edit-special` | Edit the selected source block in a retained draft. |
| C-c C-v I | `org-babel-view-src-block-info` | Inspect declared language, name and effective header data. |
| C-c C-v h | `org-babel-insert-header-arg` | Insert one checked header argument on the opening line. |
| C-c C-v e / C-c C-c on a source block | `org-babel-execute-src-block` | Select a configured runner and explicitly confirm a bounded source job. |
| C-c C-x C-v | `org-link-preview` | Explicitly preview the local image at point, in a region or in the current section. |
| C-c C-x C-M-v | `org-link-preview-refresh` | Refresh bounded local previews in the current view. |
| M-x | `org-link-preview-clear` | Remove this view's disposable previews. |
| M-x | `org-link-preview-region` | Preview the active region, or the buffer when no region is active. |
| C-c C-x g | `org-feed-update-all` | Explicitly update declared feeds with checked native imports. |
| C-c C-x G | `org-feed-goto-inbox` | Open a declared canonical inbox without fetching. |
| M-x | `org-feed-update` | Select one declared feed for an explicit update. |
| M-x | `org-feed-results` | Inspect retained per-feed import and publication outcomes. |
| M-x | `org-babel-execution-results` | Inspect retained stdout, stderr and source without inserting results. |
| C-c C-v t | `org-babel-tangle` | Tangle declared blocks from the current canonical document into checked files. |
| C-u C-c C-v t | `org-babel-tangle-single-block` | Tangle only the block at point, retaining its declared named references. |
| M-x | `org-babel-tangle-results` | Inspect the actual outcome of each file from the last tangling operation. |
| M-x | `org-publish`, `org-publish-results` | Publish a configured document project and inspect retained per-file outcomes. |
| M-x | `org-export-to-text`, `org-export-to-unicode`, `org-export-to-markdown`, `org-export-to-org`, `org-export-to-html`, `org-export-to-latex`, `org-export-to-beamer`, `org-export-to-latex-pdf`, `org-export-to-beamer-pdf`, `org-export-to-odt`, `org-export-to-texinfo`, `org-export-to-man`, `org-export-to-icalendar` | Export a checked buffer, subtree or region to an explicit local file; confirm observed replacements. |
| M-x | `org-columns`, `org-columns-insert-dblock` | Inspect source property columns or explicitly insert/update their native report. |

## Configuration keymap contracts

See the shared [keymap rules](../bindings.md#configuration-keymap-contracts)
and [sequence vocabulary](../bindings.md#sequence-vocabulary).

Org editing, capture and special-edit roles use `keys.org`, `keys.org_capture`
and `keys.org_source_edit`, inheriting the admitted editor map before local
overrides. Capture adds `C-c C-c` to finalize and `C-c C-k` to abort its transient draft; special editing adds `C-c '` and `C-c C-c` to return, and `C-c C-k` to abort. Its named controls are `org-edit-src-exit` and `org-edit-src-abort`; see [source blocks](../org-source-blocks.md). Agenda
uses an independent `keys.org_agenda` map. The native date prompt inherits the
minibuffer map before `keys.org_calendar` controls. Host agenda/calendar roles
consume unbound text and paste. They admit implemented motion, scroll, quit, command-entry
and cancellation targets. The agenda role also admits the implemented priority/effort
source commands: comma selects priority, e sets effort and semicolon starts
the session countdown prompt. Switching roles clears pending chords and Escape timing
without accepting a prompt. The native habit agenda is read-only; priority/effort source editing requires
returning to source. Feature owners activate their role when delivered;
role selection alone does not create an agenda or draft.

### Org agenda defaults and availability

| Keys | Target | Behavior |
| --- | --- | --- |
| `v t` | `org-todo-list` | Select TODO states; empty means unfinished. |
| `s` | `org-search-view` | Enter a text or Boolean search. |
| `M` | `org-tags-view` | Enter a tag/property expression. |
| `t` | `org-todo` | Change the checked source task state. |
| `m` | `org-agenda-bulk-mark` | Mark entry and advance. |
| `u` | `org-agenda-bulk-unmark` | Unmark entry. |
| Asterisk | `org-agenda-bulk-mark-all` | Mark visible distinct targets. |
| `U` | `org-agenda-bulk-unmark-all` | Clear marks. |
| `B` | `org-agenda-bulk-action` | Select a bulk operation/value. |
| `:` | `org-set-tags-command` | Edit local source tags. |
| `C-c C-s` | `org-schedule` | Schedule the source heading. |
| `C-c C-d` | `org-deadline` | Set the source deadline. |
| `I` | `org-clock-in` | Clock into the checked task. |
| `O` | `org-clock-out` | Close the selected task's active clock. |
| `X` | `org-clock-cancel` | Cancel the selected task's active clock. |
| `C-_` | `org-agenda-undo` | Undo the retained exact source edit. |
| `g` | `org-agenda-redo` | Explicitly refresh from current source and time. |
| `K` | `org-habit-toggle-habits` | Toggle graph visibility. |
| `C-u K` | `org-habit-toggle-habits-only-for-today` | Toggle today's agenda only. |
| `A` | `org-habit-toggle-show-all-today` | Toggle all habits on today's agenda. |
| `d` | `org-habit-graph-date` | Select a view date or today. |
| `G` | `org-habit-set-graph-window` | Set preceding/following days and preferred column. |
| `Down` | `next-line` | Scroll the read-only habit rows forward. |
| `Up` | `previous-line` | Scroll the read-only habit rows backward. |
| `;` | `org-timer-set-timer` | Enter an explicit session countdown. |
| `e` | `org-set-effort` | Set the checked source entry estimate. |
| `,` | `org-priority` | Set the checked source heading priority. |
| `C-g` / `q` | `keyboard-quit` | Cancel current input or return to canonical source. |
| `M-x` | `execute-extended-command` | Enter the compiled command query. |
| `v d` | `org-agenda-day-view` | Show one captured calendar day. |
| `v w` | `org-agenda-week-view` | Show the configured calendar week. |
| `v v` | `org-agenda-custom-view` | Select a configured view key. |
| `/` | `org-agenda-filter` | Apply a tag/property visibility filter. |
| Pipe | `org-agenda-filter-remove-all` | Restore preset filters. |
| `S` | `org-agenda-sort` | Select ordered sort keys. |
| `V` | `org-agenda-columns` | Select property columns. |
| `v m` | `org-agenda-month-view` | Show an aligned month. |
| `v y` | `org-agenda-year-view` | Show an aligned year. |
| `v s` | `org-agenda-span` | Select a named span or 1–366 days. |
| `f`, `Right` | `org-agenda-later` | Move one displayed span forward. |
| `b`, `Left` | `org-agenda-earlier` | Move one displayed span backward. |
| `.` | `org-agenda-goto-today` | Return to the captured effective today. |
| `j` | `org-agenda-goto-date` | Choose YYYY-MM-DD in the retained span. |
| `RET` | `org-agenda-switch-to` | Check and visit the selected real source. |

### Org calendar defaults and availability

The active date prompt inherits the admitted minibuffer map before these local
calendar controls. Local overrides admit calendar/navigation commands; inherited
prompt-editing aliases belong in `keys.minibuffer`. Host calendar roles without
an active date prompt cannot dispatch inherited deletion or prompt-only actions.

| Keys | Target | Behavior |
| --- | --- | --- |
| `RET` | `org-calendar-select` | Accept the interpreted date/time; offset selection may follow. |
| `S-Right` | `org-calendar-forward-day` | Select the next day. |
| `S-Left` | `org-calendar-backward-day` | Select the previous day. |
| `S-Down` | `org-calendar-forward-week` | Select one week later. |
| `S-Up` | `org-calendar-backward-week` | Select one week earlier. |
| `M-S-Right`, `>` | `org-calendar-forward-month` | Select one month later. |
| `M-S-Left`, `<` | `org-calendar-backward-month` | Select one month earlier. |
| `M-S-Down` | `org-calendar-forward-year` | Select one year later. |
| `M-S-Up` | `org-calendar-backward-year` | Select one year earlier. |
| `M-v` | `org-calendar-scroll-three-months-right` | Move back three months. |
| `C-v` | `org-calendar-scroll-three-months-left` | Advance three months. |
| `C-.` | `org-calendar-goto-today` | Select the captured current date. |
| `C-g` | `keyboard-quit` | Discard the draft. |
| `M-x` | `execute-extended-command` | Command entry in a host calendar role; active date prompts retain input focus. |
| `TAB` | `minibuffer-complete` | Expand a valid date draft to timestamp spelling. |
| Backspace | `backward-delete-char` | Delete the previous prompt grapheme. |
| Delete, `C-d` | `delete-char` | Delete the next prompt grapheme. |
| Left, `C-b` | `backward-char` | Move back in prompt text. |
| Right, `C-f` | `forward-char` | Move forward in prompt text. |
| Home, `C-a` | `move-beginning-of-line` | Move to the draft start. |
| End, `C-e` | `move-end-of-line` | Move to the draft end. |
| Up, `C-p` | `minibuffer-previous` | Retain the argument-stage candidate policy. |
| Down, `C-n` | `minibuffer-next` | Retain the argument-stage candidate policy. |
| `M-p` | `minibuffer-history-previous` | History applies only to command prompts. |
| `M-n` | `minibuffer-history-next` | History applies only to command prompts. |
| `C-s` | `isearch-forward` | Search repetition applies only to search prompts. |
| `C-r` | `isearch-backward` | Search repetition applies only to search prompts. |

### Native mode selection

`M-x org-mode` selects native Org behavior for the current canonical buffer.
`M-x text-mode` selects ordinary text editing and releases retained Org indexes.
Both commands retain source, history, saved state and the mode choice across
shared views and file reassociation. Visiting an admitted `.org` basename enables
Org through the existing association resolver; user rules can override it.
`org.enabled = false` rejects explicit entry and keeps source editable as text.
Mode activation starts no analysis worker or idle polling.

### Org date entry and calendar

| Keys in Org | Target | Behavior |
| --- | --- | --- |
| `C-c .` | `org-timestamp` | Insert or edit an active timestamp with a native date/calendar prompt. |
| `C-c !` | `org-timestamp-inactive` | Insert or edit an inactive timestamp. |
| `C-c C-s` | `org-schedule` | Set the current heading's SCHEDULED fact. |
| `C-c C-d` | `org-deadline` | Set the current heading's DEADLINE fact. |
| `C-c >` | `org-goto-calendar` | Inspect a calendar and retain its selected date. |
| `C-c <` | `org-date-from-calendar` | Insert the retained calendar date at point. |

The transient date prompt inherits `keys.minibuffer` text editing and completion.
`keys.org_calendar` supplies the following context actions. Its local overrides
admit calendar/navigation commands; prompt editing aliases belong in
`keys.minibuffer`. A host calendar role without a prompt cannot edit document text.

| Keys in the date prompt | Target | Behavior |
| --- | --- | --- |
| `RET` | `org-calendar-select` | Accept the interpreted date/time; an offset choice may follow. |
| `TAB` | `minibuffer-complete` | Expand a valid draft into timestamp spelling. |
| `S-Right`, `S-Left` | `org-calendar-forward-day`, `org-calendar-backward-day` | Select the next/previous day. |
| `S-Down`, `S-Up` | `org-calendar-forward-week`, `org-calendar-backward-week` | Select the next/previous week. |
| `M-S-Right`, `M-S-Left`, `>`, `<` | `org-calendar-forward-month`, `org-calendar-backward-month` | Select the next/previous month. |
| `M-S-Down`, `M-S-Up` | `org-calendar-forward-year`, `org-calendar-backward-year` | Select the next/previous year. |
| `M-v`, `C-v` | `org-calendar-scroll-three-months-right`, `org-calendar-scroll-three-months-left` | Move backward/forward by three months. |
| `C-.` | `org-calendar-goto-today` | Select the command's captured current date. |
| Left click | Calendar date cell | Select a visible day without accepting the source edit. |
| `C-g` | `keyboard-quit` | Discard the entire draft and return to the original workspace. |

### Org timestamp context actions

| Keys in Org | Target | Behavior |
| --- | --- | --- |
| `S-Up`, `S-Down` | `org-shiftup`, `org-shiftdown` | Adjust the timestamp's selected year/month/day/hour/minute; otherwise cycle the current heading priority. |
| `S-Right`, `S-Left` | `org-shiftright`, `org-shiftleft` | Adjust a timestamp by one day; otherwise select the next/previous task state on its heading. |
| `C-c C-c` on a timestamp | `org-ctrl-c-ctrl-c` | Repair the selected weekday, retaining other source fields. |
| `M-x org-timestamp-up`, `org-timestamp-down` | Same name | Adjust the selected timestamp field without a priority fallback. |
| `M-x org-timestamp-up-day`, `org-timestamp-down-day` | Same name | Adjust the selected timestamp by one civil day. |

At a timestamp bracket, field adjustment toggles active/inactive status for the
connected range. Moving the first clock moves its end equally; moving an end
clock changes the range length. Literal contexts cannot select an ancestor task.
Explicit priority commands retain their priority meaning. On other contexts,
`C-c C-c` retains its heading-tag and list-checkbox behavior. Two successive
`C-c .` commands, or two `C-c !` commands, enter a checked date range; another
editing command or cancellation retires that succession.

### Native task clocks

Native task clocks insert `CLOCK: [YYYY-MM-DD Ddd HH:MM]` in LOGBOOK by
factory policy. CLOCK_INTO_DRAWER overrides LOG_INTO_DRAWER; either accepts a
selected drawer, t for LOGBOOK or nil for normal source placement. Clock-out
adds `--[end] =>  H:MM`. Both endpoints have minute precision. Elapsed minutes
subtract captured UTC minute endpoints, including across offset transitions.
The interval retains the zone selected at clock-in for consistent source dates.

Clock commands publish one ordinary undoable transaction each. Clock-out/cancel
can address the active source from another buffer without changing the current
view. Goto selects and reveals its heading. A changed source revision rejects
these commands and retains the open source; it never relocates by guessed text.
An active clock adds no idle deadline, repaint loop or periodic source write.

`org.clock.drawer` selects default source placement. `org.clock.remove_zero`
defaults false: a same-minute out retains a `0:00` record. Selecting true removes
that record. Cancel removes its open record. Either removal also removes an
otherwise empty clock drawer; unrelated logs and drawer text remain exact.

Clocking a different heading closes the prior interval and opens the selected
heading using one captured instant. Both source buffers and all shared views are
prepared before either changes. Files are saved only by normal explicit saves.
Clocking the same active heading again leaves its interval unchanged.

`org.clock.out_when_done` defaults true. Selecting a finished task state closes
that task's clock together with the keyword, planning and selected note/history
edits. Note cancellation leaves the clock and source intact. A successful note
uses the captured completion instant, excluding time spent typing the note.

Quit and closing the clock's buffer first ask `close` or `keep`. Close records
clock-out; keep detaches tracking and preserves the open source interval. Normal
save/discard/cancel questions then retain their existing per-file meaning. C-g
at the clock question leaves the clock running. A failed clock-out stops departure.

`org-resolve-clocks` observes at most 64 incomplete intervals in the selected
262144-byte entry. Select an interval number, then resume, close, cancel or keep.
Resume adopts its checked source instant without writing text. Close asks for
the accepted end date/time, so rejected idle time is not counted. Repeated local
times show their observed offsets; missing local times cannot be forced through.
Invalid end input remains editable. End times cannot exceed the captured operation
time. Cancel removes only the selected interval/otherwise empty drawer; keep
leaves open source unresolved. C-g and changed source/view cannot publish edits.

Undo/redo retains ordinary source/history behavior. A previously captured clock
locator becomes stale when its source revision changes, including history replay.
Use explicit resolution to adopt a freshly observed incomplete interval; similar
text in another revision never authorizes an automatic clock edit.

Recent goto uses the last qualified task when no clock is active. C-c C-x C-x
re-clocks that task. `org.clock.history_limit` defaults 32 and admits 0..256;
zero disables recent actions. Closed buffers are removed, and stale revisions
require fresh task selection instead of guessed relocation.

`org.clock.persist` defaults false. When selected, application startup reads only
`org-clock.toml` under the resolved XDG state namespace and asks before visiting
its recorded source. The state directory is created only by an explicit clock
checkpoint. Source files still require ordinary explicit saving. A missing or
changed saved interval remains unresolved; recovery never recreates source text.
Recovery checks file identity/metadata, canonical path, heading/line positions,
actual source bytes, native Org context and the selected start instant separately.

State contains one bounded clock locator, its original source, captured start and
last acknowledged operation time. State admits 65536 UTF-8 bytes, 4096-byte path/
source fields and a 1048576-byte source visit. Checkpoints use checked atomic
replacement and report published-but-unresolved completion separately. A failed
state checkpoint retains canonical source/tracking and never rolls back a real
clock operation. Active clocks add no state polling or periodic source writes.

Native hosts initialize a selected state path explicitly with
`Session::restore_clock_state`; library constructors perform no implicit state IO.
Accepting an empty clock end uses the captured operation instant. A minute-valued
end within the starting UTC minute records zero; backwards operation clocks still
reject independently.

### Clock totals and reports

`org-clock-display` reads actual task, subtree or file intervals without editing.
Enter `task`, `subtree` (the empty default) or `file`, optionally followed by
`YYYY-MM-DD YYYY-MM-DD` date bounds; the end date is exclusive. Each operation
captures its zone once. Totals clip actual minute endpoints to that window,
exclude open/unresolved records, and disclose both counts and overlapping
intervals. Overlapping recorded intervals sum separately; overlap is visible.
Stored duration spelling never substitutes for actual resolvable endpoints.
Fold/gap source timestamps remain unresolved rather than inventing an offset.
A task/subtree compares its root EFFORT; a file compares top-level estimates.
Reports admit 262144 source bytes, 4096 CLOCK candidates and the native 4096
heading/drawer bound. Incomplete observations reject. Read-only inspection is
available; C-g or a changed source/view cancels the request without edits.

`org-clock-report` inserts a compatible `#+BEGIN: clocktable` / `#+END:` block
on a blank line or empty EOF, or updates the selected existing block.
`org-dblock-update` and C-c C-c in a dynamic block explicitly refresh that body.
The opening/closing lines, other blocks and unrelated source remain exact.
A successful change uses one ordinary transaction; an unchanged result adds no
history entry. Read-only source rejects before publication. Rendering alone
never refreshes a report.

Clocktable options are data only: `:scope file` (or nil) / subtree,
`:maxlevel 1..256` (default 2), `:tstart` and `:tend` (quoted Org date/time
strings or YYYY-MM-DD; end exclusive), or `:block` with today, yesterday,
thisweek, lastweek, thismonth, lastmonth, thisyear, lastyear, untilnow or one
YYYY-MM-DD day. Default scope is file and default range is the captured current
year. `:block untilnow` selects all recorded closed time. Duplicate, unsupported,
conflicting or incomplete options reject without rewriting the original body.
No Lisp, formatter, formula or implicit source discovery is evaluated.
Tables show subtree totals up to maxlevel, root effort and record diagnostics;
deep descendant time contributes to its visible ancestors and the grand total
counts each interval once. Output admits 1048576 UTF-8 bytes.

Clock interval calculation and table rendering share the native Org worker
slot. C-g cancels pending work; compute checks cancellation and yields at 10ms
boundaries. A completed result must retain the initiating source revision,
selected view and editing focus. Changed source, moved point or canceled work
cannot publish a report. Completion retires the worker deadline; idle reports
add no polling or source writes. `org::agenda::ClockScan` exposes the same
bounded native snapshot/result contracts for explicit host analysis without
file discovery or edit authority.

### Session timers

Native timers use one session-owned monotonic observation, independent of task
clocks, civil zones and file changes. Start/reset selects zero relative time.
Insertion starts a relative timer only after its source edit succeeds when no
timer exists. H:MM:SS values and `- H:MM:SS :: ` description items use ordinary
checked edits and single-step undo. M-RET in a timed item continues that list;
ordinary lists retain their existing behavior and explicit timer-item rejects a
non-timer list. Literal/unclosed source and read-only edits reject. Undo changes
source without rewinding the session timer.

Countdown entry accepts unsigned whole minutes or H:MM:SS, at most 365 days and
64 UTF-8 bytes. `org.timer.default` is "0" and arms nothing at startup. Enter
accepts a configured positive default; zero/invalid input stays editable. A
running relative timer must stop before selecting a countdown, and vice versa.
Replacing a countdown asks yes/no; C-g preserves the previous timer. Pause keeps
its precise elapsed/remaining value; continue excludes paused time. Stop prevents
resumption of the old timer. Timers have no persisted state or implicit restart.

Only an active countdown arms a native deadline. Relative and paused/stopped
states add no wait or repaint loop. Ready input is drained before due work, so
a stop queued at the deadline cancels expiry. A delayed wake signals once and
clears the timer. The selected pane shows a disposable timer value on ordinary
frames; narrow status lines retain their normal tokens. Expiry shows an in-editor
notice, including during a prompt or save question without replacing that draft
or question. The done marker remains until input. Clean exit and terminal I/O
failure discard the session timer; reopening the editor starts unarmed.

### Org agenda source discovery

C-c a and M-x org-agenda use the same explicit discovery handler. Configured
org.agenda.sources replace by layer and resolve relative to the assigning file.
With no roots, enter current for the selected native Org buffer, buffers for
all eligible open Org buffers, or one literal local path. Merely opening the
prompt observes no sources. C-g cancels selection or owned work.

Directories select immediate .org children only. Symlinks and hard links share
real file identity; current buffer text wins over disk, even when unsaved.
Conflicting live hard-link buffers require explicit buffer selection. Missing,
unreadable, binary, changed or over-limit sources remain reported as incomplete;
available whole sources remain retained. Source discovery does not save, create
or edit files. Normal editing remains available during the one owned worker.

### Native day and week calendars

C-c a shows a read-only native calendar after explicit source selection. Use
v d/v w for day/week, f/b or Right/Left for span movement, . for the captured
effective today, and j for a date. These controls reuse captured source facts
and zone rules. g explicitly refreshes sources and captures time once again.
The same g binding refreshes a habit agenda.

Up/Down and page commands select/scroll derived lines. RET visits an entry only
after checking its source revision. Disk sources are reread within the owned
IO budget and their actual text must agree before normal canonical ingress.
Changed sources require refresh. q/C-g closes the calendar; prompt cancellation
returns to the calendar. Ordinary text, paste, editing commands and pointer
input cannot mutate source through the calendar. Other-window permits editing
an independent peer pane while the calendar keeps its own pane.

Planning sections show scheduled/deadline dates, unfinished overdue reminders
and approaching deadlines. Explicit source warning/delay suffixes apply; the
default deadline warning is fourteen days. Time grids retain wall-clock times;
ambiguous/nonexistent local times remain marked unresolved. Partial sources,
malformed data and limits stay visible as incomplete results. Date navigation
and rendering never advance repeating source or poll a clock while idle.

See [Org agenda queries](../org-agenda.md) for source scopes, query grammar, limits and checked return.

Configured and runtime agenda controls use the same compiled handlers. Filters
change visibility over retained snapshots; `g` alone refreshes sources. `|`
restores preset filters while keeping selected sorting and columns. Empty sort
or column entry restores native order or the ordinary text presentation.


`C-c C-x f` runs `org-footnote-action`; `C-c C-x C-f` runs `org-emphasize`. Other document controls are available through M-x. See [Org document editing](../org-document.md).

In a native table, TAB and S-TAB align and visit the next/previous field; C-c C-c aligns, and M-RET inserts a row below. C-c followed by the vertical-bar key creates a table. See [Org tables](../org-tables.md).

[Org reference](index.md)
