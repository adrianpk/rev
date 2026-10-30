# Native Org controls

The [command catalog](commands.md) lists every compiled M-x name once. The
[binding reference](bindings.md) describes familiar keys and focus ownership.
The [complete sample](config.toml), also embedded in the executable, contains
all 178 schema/catalog entries, 54 RGB roles and 263 local default bindings
across eight contexts. [Defaults](defaults.md) owns their types, bounds and
factory values. Empty arrays disable optional declarations; they do not hide
an implicit program, source scan or destination.

## Capability map

Names below are representative entry commands. Each linked feature reference
describes the remaining operations, required source context and refusals.
An unavailable target reports a diagnostic; naming a command does not grant
file, process or publication consent.

| Capability and reference | Native entry commands | Startup preferences or fixed policy |
| --- | --- | --- |
| [Mode, visibility and outline](org-mode.md) | `org-mode`, `text-mode`, `org-cycle`, `org-shifttab`, `org-show-all`, `org-insert-heading`, `org-promote-subtree` | `org.enabled`; views start fully visible and retain independent runtime folds. |
| [Lists and checkboxes](org-mode.md) | `org-insert-item`, `org-toggle-checkbox`, `org-update-statistics-cookies` | Native list structure and progress use canonical source. |
| [Tasks and dependencies](org-mode.md) | `org-todo`, `org-todo-set`, `org-inspect-todo` | `org.todo.sequence`, `finished`, `log_done`, `log_repeat`, `log_drawer`, `block_children`. |
| [Tags and properties](org-mode.md) | `org-set-tags-command`, `org-set-property`, `org-inspect-property`, `org-match-tags` | `org.tags.inherit`, `org.properties.inherit`; admitted document declarations affect data semantics. |
| [Priority and effort](org-mode.md) | `org-priority`, `org-set-effort`, `org-inc-effort` | `org.priority.highest`, `lowest`, `default`; `org.duration.units` defines finite work-minute scales. |
| [Time and timestamps](org-mode.md#operation-time-zones) | `org-timestamp`, `org-schedule`, `org-deadline`, `org-goto-calendar` | `org.time.zone`, `ambiguous`, `nonexistent`, `extend_today_until`; repeaters remain source data. |
| [Clocks and reports](bindings.md#native-task-clocks) | `org-clock-in`, `org-clock-out`, `org-resolve-clocks`, `org-clock-report` | `org.clock.drawer`, `persist`, `out_when_done`, `remove_zero`, `history_limit`. |
| [Timers](bindings.md#session-timers) and [habits](org-mode.md#habit-consistency-graphs) | `org-timer-start`, `org-timer-set-timer`, `org-habit-show-graph` | `org.timer.default`; all six `org.habit` window/display preferences. Timers start unarmed. |
| [Agenda and queries](org-agenda.md) | `org-agenda`, `org-todo-list`, `org-search-view`, `org-tags-view`, `org-agenda-custom-view` | `org.agenda.sources`, `span`, `week_start`, `include_archived`, `views`; finite agenda/query limits. |
| [Capture](org-capture.md) | `org-capture`, `org-capture-finalize`, `org-capture-kill` | `org.capture.templates`, `default_file`; empty templates select native Task with explicit destination entry. |
| [Refile and archive](org-files.md) | `org-refile`, `org-refile-copy`, `org-archive-subtree`, `org-refile-results` | `org.refile.targets`, `org.archive.location`; each canonical participant has its own save outcome. |
| [Links](org-links.md) | `org-store-link`, `org-insert-link`, `org-open-at-point`, `org-mark-ring-goto` | Checked source targets; `org.tools.viewer` supplies the separately confirmed external path. |
| [Attachments](org-attachments.md) | `org-attach`, `org-attach-copy`, `org-attach-results` | `org.attachments.directory`; source ID/DIR selects data, never executable authority. |
| [Document conveniences](org-document.md) | `org-footnote-action`, `org-emphasize`, `org-insert-entity` | Literal native source edits, ordinary read-only and history admission. |
| [Tables and formulas](org-tables.md) | `org-table-align`, `org-table-recalculate`, `org-table-import`, `org-columns` | `org.tables.significant_digits`; formula cell/step/depth/decimal limits. Empty numeric cells use zero; rounding is half away from zero. |
| [Exports and publishing](org-export.md) | `org-export-to-html`, `org-export-to-odt`, `org-export-to-icalendar`, `org-publish` | `org.export.default_format`, `backends`, `org.publish.projects`; include/expansion/export limits; optional `org.tools.pdf`, `upload`. |
| [Source blocks](org-source-blocks.md) | `org-edit-special`, `org-babel-tangle`, `org-babel-execute-src-block`, `org-babel-execution-results` | `org.source.confirm_execution`, `programs`, `library_files`; job time/output limits. Recognition never executes source. |
| [Feeds](org-feeds.md) | `org-feed-update-all`, `org-feed-update`, `org-feed-results` | `org.feeds.sources`, `org.tools.fetch`, feed byte/item limits. Updates are explicit. |
| [Previews](org-files.md#link-and-image-previews) | `org-link-preview`, `org-link-preview-refresh`, `org-link-preview-clear` | `org.preview.enabled`, `max_bytes`; explicit view-local observations, direct Kitty PNG presentation or readable viewer fallback. |

All configurable limits appear under `org.limits` in the sample and default
catalog. Fixed structural bounds and one owned external job remain native
invariants, not accepted TOML leaves. Source-local keywords cannot change
startup authority. Unknown keys, record fields and M-x names reject.

## Context and palette ownership

| Context map | Owner and inheritance |
| --- | --- |
| `keys.editor` | Ordinary editing; includes agenda, capture and store-link entry commands. |
| `keys.org` | Org source overlay on editor keys; TAB and contextual actions use native source handlers. |
| `keys.org_capture` | Editor overlay for the transient draft; C-c C-c finalizes, C-c C-k aborts. |
| `keys.org_source_edit` | Editor overlay for source editing; C-c ' or C-c C-c returns, C-c C-k aborts. |
| `keys.org_agenda` | Independent retained agenda; text cannot insert into source rows. |
| `keys.org_calendar` | Native date-entry overlay on minibuffer keys. |
| `keys.minibuffer` | Focused argument, command, search and decision input takes precedence. |
| `keys.tree` | Independent tree focus takes precedence over a retained Org editing role. |

C-g cancels in every context. A key and its M-x name select the same compiled
handler; named invocation still requires that handler's admitted context.
Agenda bulk controls invoked from source report that an agenda is required and
preserve the current source view.

`palette.org` contains 17 source roles: heading, keyword, drawer, property,
planning, block, code, comment, list, table, formula, emphasis, link, target,
footnote, entity and malformed. Eight habit backgrounds distinguish clear,
ready, alert and overdue states, each with a future variant. Agenda selection,
clock/status text, prompts and failures use the existing selection, status,
message, warning and error roles. NO_COLOR preserves readable source and status
without emitting RGB or unsupported graphics controls.

## Optional declarations

Commented Org examples in the complete sample show a named agenda, capture
target, publish project, local/remote feed and source program. Each example
replaces the corresponding active empty array. Optional tool argv examples
replace `[org.tools]` values; tokens are literal arguments with only their
documented whole-token substitutions. Uncommenting configuration performs no
source discovery, target creation, network access or program launch. Native
commands still require explicit selection, checked paths and tool consent.
