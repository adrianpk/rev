# Org configuration

Org settings use the same [startup loading and precedence](../defaults.md#startup-configuration)
as editor settings. The [complete sample](../config.toml) stays one file,
including every editor and Org default. [Core configuration](../defaults.md#configuration-value-catalog)
and the tables here form the complete accepted schema catalog.

Values load once at startup; editing configuration requires a restart.
Empty optional arrays do not create targets, scan sources or run programs.
See the [capability map](../org-controls.md) for the behavior each area controls.

## Startup settings

| Key | Type and bounds | Default | Scope and effect |
| --- | --- | --- | --- |
| `org.timer.default` | String | `"0"` | Countdown prompt default in unsigned minutes or H:MM:SS, 0..365 days and 64 UTF-8 bytes. Zero arms nothing. Native timers are session-local and never restored at startup. |
| `org.clock.persist` | Boolean | `false` | Opt-in checked clock state under the selected XDG state root; no state directory or startup source scan when false. Recovery asks before visiting the recorded source. |
| `org.clock.history_limit` | Integer | `32` | 0..256 checked recent task locators; zero disables recent goto/re-clock. |
| `org.clock.out_when_done` | Boolean | `true` | Complete a clocked task and close its interval in one source transaction; repeating completion also closes the selected task interval. |
| `org.clock.drawer` | String | `"LOGBOOK"` | Task-owned clock drawer. Empty places records after planning/properties. CLOCK_INTO_DRAWER overrides LOG_INTO_DRAWER, then this setting. |
| `org.clock.remove_zero` | Boolean | `false` | Retain zero-minute records by default. When true, clock-out removes the zero interval and its otherwise empty drawer. |
| `org.time.extend_today_until` | Integer | `0` | Day starts at this local hour, 0..23; date-only habit recurrence uses the extended day while logs retain actual time. |
| `org.capture.templates` | Array of typed capture templates | `[]` | Empty selects the native Task fallback; at most 64 exact keys, data-only targets and bounded text. See [capture](../org-capture.md). |
| `org.capture.default_file` | String | `""` | Empty requires explicit destination entry. Relative paths use the assigning configuration source; discovery never creates files. |
| `org.tools.viewer` | Array of argv strings | `[]` | Optional external viewer, at most 64 nonempty tokens of 4096 bytes each. One whole `{target}` token or an appended target; explicit follow and confirmation required. See [links](../org-links.md). |
| `org.tools.pdf` | Array of argv strings | `[]` | Optional explicit PDF compiler, at most 64 nonempty tokens of 4096 bytes each. Exactly one whole `{input}` token; optional `{output}` and `{output_dir}` once each. Explicit PDF command and confirmation required. See [exports](../org-export.md). |
| `org.attachments.directory` | String | `"data"` | ID-based attachment directory relative to the source file. A selected DIR property takes precedence; link following never creates it. |
| `org.archive.location` | String | `"%s_archive::"` | Checked `file::heading` relative to the source file, `%s` expands its canonical path. Empty heading appends a level-one subtree. At most 4096 bytes; only local paths and the `%s` escape. |
| `org.refile.targets` | Array of local path strings | `[]` | Explicit file/directory roots, relative to their assigning configuration source. Empty uses admitted agenda/source headings. At most 4096 roots, 4096 bytes per path; no startup observation. |
| `org.agenda.include_archived` | Boolean | `false` | Exclude ARCHIVE-tagged subtrees and their children from selected calendar, TODO, search and tag/property sources. True admits them; COMMENT subtrees remain excluded. |
| `org.agenda.sources` | Array of local path strings | `[]` | Explicit files or non-recursive directories of immediate .org files; 4096 roots, 4096 bytes per path, relative to the assigning config source. No implicit scan. |
| `org.agenda.span` | String: "day", "week", "month", "year"; or Integer 1..=366 | `"week"` | Initial calendar span; months and years align to their first date. |
| `org.agenda.views` | Array of typed records | `[]` | At most 64 named views, each with 1..32 blocks; see [agenda reference](../org-agenda.md). |
| `org.agenda.week_start` | Integer 1..=7 | `1` | Start of weekly calendars; Monday=1. |
| `org.limits.agenda_rows` | Integer 1..=100000 | `20000` | Maximum retained source facts and calendar entries; limits remain visibly incomplete. |
| `org.limits.query_bytes` | Integer 1..=65536 | `4096` | Maximum query UTF-8 bytes. |
| `org.limits.query_depth` | Integer 1..=128 | `32` | Maximum nested query forms. |
| `org.source.confirm_execution` | Boolean | `true` | Ask before each selected block job; `:eval query` always asks and no/never always reject. |
| `org.preview.enabled` | Boolean | `false` | Initial per-view preference; explicit commands own resource reads. |
| `org.preview.max_bytes` | Integer: 1..16777216 | `4194304` | Per-resource byte bound for explicit local PNG observations. |
| `org.feeds.sources` | Array of typed feed records | `[]` | At most 64 named local/HTTP feed sources with target file/headline, optional drawer and transport argv. No startup IO; see [feeds](../org-feeds.md). |
| `org.tools.fetch` | Array of argv strings | `[]` | Optional bounded feed transport, one whole `{target}` token or one appended target. |
| `org.limits.feed_bytes` | Integer 1..=16777216 | `4194304` | Maximum complete XML source bytes per feed. |
| `org.limits.feed_items` | Integer 1..=10000 | `1000` | Maximum admitted items per complete feed observation. |
| `org.source.library_files` | Array of local path strings | `[]` | At most 32 explicit files, 4096 bytes per path, relative to the assigning config source; bounded observations only on explicit named calls. |
| `org.source.programs` | Array of typed program profiles | `[]` | At most 32 named explicit language runners; arrays replace by layer. See [source execution](../org-source-blocks.md#execution) for closed fields, finite limits and argv substitution. |
| `org.publish.projects` | Array of typed project records | `[]` | At most 64 named data-only projects, explicit roots/output/formats and bounded include/exclude/asset patterns. No startup discovery or automatic cleanup; see [publishing](../org-export.md#project-publishing). |
| `org.tools.upload` | Array of argv strings | `[]` | Optional explicit upload tool, at most 64 nonempty tokens of 4096 bytes each. Whole `{output_dir}` substitution only; separate confirmation after successful project publication. |
| `org.export.default_format` | String: "text", "utf8", "org", "markdown", "html", "latex", "beamer", "odt", "texinfo", "man", "icalendar" | `"html"` | Selected format must occur in enabled backends; no automatic export. |
| `org.export.backends` | Array of 1..11 unique native formats | `["text", "utf8", "org", "markdown", "html", "latex", "beamer", "odt", "texinfo", "man", "icalendar"]` | text (ascii alias), utf8, org, markdown, html, latex, beamer, odt, texinfo, man, icalendar. |
| `org.limits.include_depth` | Integer 1..=64 | `16` | Nested includes, excluding the root. |
| `org.limits.include_files` | Integer 1..=512 | `128` | Include occurrences, including repeated files. |
| `org.limits.expansion_bytes` | Integer 1..=8388608 | `1048576` | Maximum projected bytes after include and text macro expansion. |
| `org.limits.export_bytes` | Integer 1..=134217728 | `16777216` | Maximum rendered export bytes. |
| `org.tables.significant_digits` | Integer 1..=15 | `12` | Finite binary64 result digits unless a stored fixed decimal format is explicit. |
| `org.limits.formula_cells` | Integer 1..=1000000 | `100000` | Numerical grid cells; native table shape/source limits also apply. |
| `org.limits.formula_steps` | Integer 1..=10000000 | `1000000` | Work budget for each syntax/numeric stage of selected recalculation. |
| `org.limits.formula_depth` | Integer 1..=256 | `64` | Maximum nested formula syntax levels. |
| `org.limits.formula_decimal_places` | Integer 0..=15 | `12` | Absolute signed round precision and unsigned fixed-format decimal precision. |
| `org.limits.agenda_files` | Integer 1..=4096 | `512` | Maximum unique admitted source documents; excess remains visibly incomplete. |
| `org.limits.agenda_file_bytes` | Integer 1..=33554432 | `8388608` | Whole-source byte bound for disk reads and current buffer snapshots. |
| `org.limits.agenda_total_bytes` | Integer 1..=268435456 | `67108864` | Total read/snapshot byte budget, including failed reads. |
| `org.limits.job_seconds` | Integer 1..=600 | `30` | Agenda discovery and export preparation deadline, checked at read/iteration boundaries; does not interrupt blocked kernel IO. |
| `org.limits.job_output_bytes` | Integer 1..=16777216 | `1048576` | Combined stdout/stderr byte limit for explicitly launched Org tools. Exceeding the limit stops the owned process group and reports failure. |
| `org.habit.preceding_days` | Integer | `21` | Days before today, 0..366. |
| `org.habit.following_days` | Integer | `7` | Days after today, 0..366. |
| `org.habit.graph_column` | Integer | `40` | Preferred graph column, 0..4096; clip to the pane. |
| `org.habit.today_only` | Boolean | `true` | Show graphs only on today's agenda. |
| `org.habit.show_all_today` | Boolean | `false` | Include habits not yet scheduled on today's agenda. |
| `org.habit.done_always_green` | Boolean | `false` | Color late completed days as ready when explicitly enabled. |
| `org.time.zone` | String | `"local"` | local honors captured TZ; system uses localtime; UTC or a validated relative IANA name overrides TZ. Zone data is checked when the operation is selected. |
| `org.time.ambiguous` | String: "prompt" | `"prompt"` | Require a selected observed offset for repeated local times; never guess. |
| `org.time.nonexistent` | String: "prompt" | `"prompt"` | Require a valid replacement local time for a forward gap; never normalize silently. |
| `org.priority.highest` | String | `"A"` | One uppercase A-Z character; highest <= default <= lowest. File PRIORITIES data may select the buffer range. |
| `org.priority.lowest` | String | `"C"` | One uppercase A-Z character; highest <= default <= lowest. File PRIORITIES data may select the buffer range. |
| `org.priority.default` | String | `"B"` | One uppercase A-Z character; highest <= default <= lowest. File PRIORITIES data may select the buffer range. |
| `org.duration.units.min` | Finite float, positive | `1.0` | Elapsed work minutes for the min suffix, independent of calendar arithmetic. |
| `org.duration.units.h` | Finite float, positive | `60.0` | Elapsed work minutes for the h suffix, independent of calendar arithmetic. |
| `org.duration.units.d` | Finite float, positive | `1440.0` | Elapsed work minutes for the d suffix, independent of calendar arithmetic. |
| `org.duration.units.w` | Finite float, positive | `10080.0` | Elapsed work minutes for the w suffix, independent of calendar arithmetic. |
| `org.duration.units.m` | Finite float, positive | `43200.0` | Elapsed work minutes for the m suffix, independent of calendar arithmetic. |
| `org.duration.units.y` | Finite float, positive | `525960.0` | Elapsed work minutes for the y suffix, independent of calendar arithmetic. |
| `org.todo.sequence` | Array of keywords | `["TODO", "DONE"]` | Ordered 1..64 unique task words, each 1..128 UTF-8 bytes without whitespace, controls or task delimiters. Source declarations override this sequence. |
| `org.todo.finished` | Array of keywords | `["DONE"]` | Unique subset of sequence; empty permits workflows without finished states. |
| `org.todo.log_done` | Boolean | `false` | Add CLOSED on completion. Source state annotations or LOGGING select additional logging. |
| `org.todo.log_repeat` | String: "none", "time" or "note" | `"time"` | Repeated completion logging; time/note also record LAST_REPEAT. STARTUP repeat options and LOGGING properties can override it. |
| `org.todo.log_drawer` | String | `""` | Empty means normal source placement; otherwise 1..64 ASCII letters, digits, _ or -, excluding PROPERTIES and END. LOG_INTO_DRAWER overrides locally or by inheritance. |
| `org.todo.block_children` | Boolean | `false` | Block completion when a descendant task is unfinished. ORDERED parents additionally require earlier siblings to finish; local NOBLOCKING=t exempts an entry. |
| `org.tags.inherit` | Boolean | `true` | Include ancestor tags and FILETAGS with source provenance; no inherited text is inserted. |
| `org.properties.inherit` | Array of property names | `[]` | At most 128 unique case-insensitive stored names, 1..128 UTF-8 bytes each; no +, derived fields or drawer delimiters. Arrays replace on merge, including empty. Only these names inherit ancestor and file defaults. |
| `org.enabled` | Boolean | `true` | Session; admit automatic Org file recognition and explicit org-mode. Per-buffer text-mode selects plain editing. |

## Palette roles

Each value is an RGB string in the shared [palette format](../defaults.md#palette-roles).

| Key | Type and bounds | Default | Scope and effect |
| --- | --- | --- | --- |
| `palette.org.heading` | RGB string | `"#7aa2f7"` | Heading lines. |
| `palette.org.keyword` | RGB string | `"#9d7cd8"` | Keywords and affiliated metadata. |
| `palette.org.drawer` | RGB string | `"#2ac3de"` | Drawer boundaries. |
| `palette.org.property` | RGB string | `"#2ac3de"` | Property entries. |
| `palette.org.planning` | RGB string | `"#ff9e64"` | Recognized planning lines. |
| `palette.org.block` | RGB string | `"#9d7cd8"` | Block boundaries. |
| `palette.org.code` | RGB string | `"#9ece6a"` | Literal source, examples, export and fixed-width text. |
| `palette.org.comment` | RGB string | `"#565f89"` | Comments and comment-block bodies. |
| `palette.org.list` | RGB string | `"#89ddff"` | List item markers. |
| `palette.org.table` | RGB string | `"#89ddff"` | Native table rows. |
| `palette.org.formula` | RGB string | `"#ff9e64"` | Table formula directives; never evaluated by coloring. |
| `palette.org.emphasis` | RGB string | `"#9d7cd8"` | Inline emphasis and its delimiters. |
| `palette.org.link` | RGB string | `"#7aa2f7"` | Link source. |
| `palette.org.target` | RGB string | `"#2ac3de"` | Ordinary and radio targets. |
| `palette.org.footnote` | RGB string | `"#ff9e64"` | Footnote objects. |
| `palette.org.entity` | RGB string | `"#2ac3de"` | Recognized entity source. |
| `palette.org.malformed` | RGB string | `"#e0af68"` | Malformed structural delimiters. |
| `palette.org.habit_clear` | RGB string | `"#0000ff"` | Habit consistency background, clear. |
| `palette.org.habit_clear_future` | RGB string | `"#191970"` | Habit consistency background, clear future. |
| `palette.org.habit_ready` | RGB string | `"#228b22"` | Habit consistency background, ready. |
| `palette.org.habit_ready_future` | RGB string | `"#006400"` | Habit consistency background, ready future. |
| `palette.org.habit_alert` | RGB string | `"#ffd700"` | Habit consistency background, alert. |
| `palette.org.habit_alert_future` | RGB string | `"#b8860b"` | Habit consistency background, alert future. |
| `palette.org.habit_overdue` | RGB string | `"#b22222"` | Habit consistency background, overdue. |
| `palette.org.habit_overdue_future` | RGB string | `"#8b0000"` | Habit consistency background, overdue future. |

## Keymaps

These maps retain the [Org context inheritance](bindings.md#configuration-keymap-contracts)
and the shared sequence validation rules.

| Key | Type and bounds | Default | Scope and effect |
| --- | --- | --- | --- |
| `keys.org` | Map of chord sequences to existing editor command names | Compiled map | Org editing role inherits keys.editor; all 81 entries in its local map are active in the sample. |
| `keys.org_capture` | Map of chord sequences to existing editor command names | Compiled map | Capture editing role inherits keys.editor; all 2 entries in its local map are active in the sample. |
| `keys.org_source_edit` | Map of chord sequences to existing editor command names | Compiled map | Special-edit role inherits keys.editor; all 3 entries in its local map are active in the sample. |
| `keys.org_agenda` | Map of chord sequences to admitted navigation, query and estimate commands | Compiled map | Agenda role consumes unbound text; all 47 entries are active in the sample. |
| `keys.org_calendar` | Map of chord sequences to admitted calendar and navigation commands | Compiled map | The date prompt inherits keys.minibuffer, then all 16 entries in its local map active in the sample. The host calendar role consumes unbound text. |

[Org reference](index.md)
