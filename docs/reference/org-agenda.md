# Org agenda queries

`M-x org-todo-list`, `M-x org-search-view` and `M-x org-tags-view` open native
read-only query agendas. Within an agenda, use `v t`, `s` or `M` respectively.
The query prompt accepts Enter and retains invalid drafts for correction;
`C-g` cancels a draft without changing the current source or agenda.

## Sources and result identity

A query uses configured `org.agenda.sources`. With no configured roots, select
`current`, `buffers` or one literal local file/directory after entering the
query. Changing query kind from an agenda retains that agenda's source scope.
Directories include immediate `.org` files without recursive discovery.
Open unsaved canonical buffers take precedence over disk observations. Source
identities are deduplicated; missing/unreadable sources remain visible.

Rows retain a real heading byte location and its buffer revision or checked
disk observation. Enter checks the selected row before visiting canonical
source. Changed buffer rows require `g` refresh. Disk return rereads bounded
text and checks actual text equality and metadata; BOM and line endings survive
normal file ingress. The agenda is derived data and has no document history.

`g` refreshes current sources and captures time/zone once. `q` closes the agenda
and restores its source view. M-x window commands can select an editable peer
pane sharing the same buffer. Removing the agenda's owning pane closes its
projection. Source task edits remain ordinary source operations.

## TODO selection

An empty TODO query lists unfinished task states from each file's effective
workflow. Explicit states such as `TODO|NEXT` select any of those keywords;
an explicitly selected completed state is admitted. COMMENT and ARCHIVE
subtrees are excluded. Task keywords are case-sensitive.

## Tag and property grammar

| Form | Meaning |
| --- | --- |
| `work+urgent-boss` | Tag AND and negation. |
| `work|home+urgent` | AND binds before OR. |
| `(work|home)&urgent` | Parenthesized Boolean forms. |
| `work/!TODO|NEXT` | Tag match and unfinished TODO-state suffix. |
| `TEAM="red"` | Case-sensitive text comparison. |
| `LEVEL>=2` | Finite numeric comparison. |
| `EFFORT<*120` | Present effort below 120 duration minutes. |
| `boss\-prio="C"` | Escaped punctuation in a property name. |

Unary `+`, `-` and `!`, AND `&`, and OR `|` are data operators. A sign can
replace explicit AND. Comparison operators are `=`, `==`, `!=`, `<>`, `/=`,
`<`, `<=`, `>` and `>=`. A following `*` requires the property to be present;
otherwise missing text compares as empty and missing numbers as zero.

Quoted values compare as original strings and support `\"`, `\\`, `\n`, `\r`
and `\t`. Unquoted numbers must be finite. Numeric EFFORT comparison parses
native duration forms into minutes using configured duration units. Ordinary
numeric properties remain scalar numbers. Tags preserve case and obey
`org.tags.inherit`; stored properties obey the selected
`org.properties.inherit` names, including file defaults, additions and `nil`.

Derived ITEM, LEVEL, TODO, PRIORITY, TAGS, ALLTAGS and FILE values use the
observed heading and file declarations. Other unsupported derived properties
produce an incomplete diagnostic. Date-relative comparisons and regexp
property/tag predicates are not part of this numeric/string match grammar.
Expressions never evaluate functions or predicates.

## Text and regexp grammar

A plain query such as `computer equipment` searches a case-insensitive phrase,
allowing multiple spaces or line breaks between words. Each heading's own
section is searched; a child's section belongs to its own row.

A leading `+` selects Boolean terms: `+computer wifi -ethernet -{8\.11[bg]}`
requires the words computer and wifi, excludes ethernet and excludes the braced
regexp. Subsequent `+` signs are optional. Double quotes group a phrase;
braces delimit a regexp. Escape quotes/backslashes in phrases and escape a
closing brace within a regexp. Positive and negative terms combine with AND.

Regexps admit literals, `.`, character classes/ranges and negated classes,
`^`/`$` line anchors, grouping, alternation, and `*`, `+`, `?` repetition.
Grouping/alternation accept parentheses/`|` and their escaped Org spellings.
Use character classes for literal parentheses or pipes. `\n`, `\r`, `\t`,
`\s`, `\w`, `\<` and `\>` represent newline, carriage return, tab, whitespace,
word character and word edges. Backreferences, lookaround, counted repetition,
unknown escapes and malformed operators reject before discovery.

## Bounds and completeness

`org.limits.query_bytes` defaults to 4096 with a maximum of 65536 UTF-8 bytes;
`org.limits.query_depth` defaults to 32 with a maximum of 128 nested forms.
Match plans and regexp automata each retain at most 512 nodes/states, and text
queries admit at most 64 Boolean terms. Heading/declaration text is at most
4096 bytes; effective tag metadata admits at most 8192 names. The existing
property model admits at most 4096 entries and 8192 property records per file.

Source discovery uses the file, per-file byte and aggregate byte limits.
`org.limits.agenda_rows` bounds retained results, and `org.limits.job_seconds`
bounds cooperative analysis. Cancellation retains worker ownership until it
finishes. A blocked kernel operation cannot be forcibly timed out. Partial,
canceled, stale and malformed observations remain visible; an unavailable
source cannot become a complete empty result. No idle query polling runs.

## Configured views

`org.agenda.views` is a replacement array of at most 64 records. Each record
has a unique `key` (1–16 bytes), a `title` (at most 256 bytes), and 1–32 `blocks`.
Unknown fields, kinds and sort keys reject. Records cannot evaluate code.

A block has these fields; omitted fields use the shown defaults:

| Field | Default | Meaning |
| --- | --- | --- |
| `kind` | `"todo"` | `calendar`, `todo`, `search`, `tags`, `properties`, `columns`, or `clock`. |
| `title` | `""` | Up to 256 bytes; empty uses the block number and kind. |
| `query` | `""` | Existing native TODO/search/tag-property grammar; empty tag/property/calendar/clock queries select all headings; search requires nonempty text. |
| `filter` | `""` | Additional tag/property expression, intersected with the query. |
| `span` | absent | Inherit `org.agenda.span`; otherwise day/week/month/year or 1–366 days. |
| `sources` | `[]` | Inherit selected sources, or select explicit file/directory roots. |
| `columns` | `[]` | At most 16 stored or native property names, each 1–64 ASCII letters/digits/underscores. |
| `sort` | `[]` | At most eight ordered keys; empty preserves native source/calendar order. |
| `row_limit` | `20000` | 1–100000 rows; the global agenda row budget also applies across blocks. |

Sort keys are `title-up`, `title-down`, `todo-up`, `priority-down`, `effort-up`,
`effort-down`, `date-up`, `clock-down` and `source-up`. Priority uses the source's
admitted priority declaration. Effort uses native duration minutes; absent effort
sorts after defined effort in ascending order. Equal keys use source/heading
order. Calendar blocks provide `DATE` and `TIME`; clock blocks provide integer
`CLOCKSUM` minutes and `CLOCKOPEN` counts. Clock intervals use the captured zone
and civil date window; malformed/gap/fold records remain explicit failures.
Columns display observations and never create an editable task database.

Month and year spans align to their first date. Custom day counts start at the
requested date. Month/year movement advances calendar units, including leap
years. Date overflow rejects without changing the existing view.

All blocks share one discovery/read budget and cancellation deadline. Source
aliases retain selected-root membership while canonical snapshots are deduplicated.
Retained property text across a block result is bounded to 8 MiB. Row, record,
source or time exhaustion is explicit; it never reports a complete empty result.

Behavior references: [custom views](https://orgmode.org/manual/Custom-Agenda-Views.html),
[filter visibility and reset](https://orgmode.org/manual/Filtering_002flimiting-agenda-items.html),
and [sorting](https://orgmode.org/manual/Sorting-of-agenda-items.html).

Use `M-x org-agenda-custom-view` and enter the configured key. Inside any native
calendar/query agenda, `v v` selects another view; `/` accepts a tag/property
filter, `|` restores preset filters, `S` accepts comma-separated sort keys and
`V` accepts comma-separated column names. Empty sort/column entry restores
native order/text presentation. `v m`, `v y`, and `v s` select month, year or an
explicit span. The corresponding compiled commands appear in the command
catalogue and use the same handlers.

Filters, sorting, columns and date movement use retained snapshots in the
existing owned job. `g` reads sources again, retaining runtime presentation and
preset identity. Closing or selecting another configured view drops runtime
overrides. Prompts can be cancelled without changing the previous result.
Every block, including an empty block, has a scrollable section header. Enter
on a section header never visits a different entry.

For example, this typed view selects a day calendar and unfinished work tasks:

```toml
[[org.agenda.views]]
key = "w"
title = "Work"
blocks = [
  { kind = "calendar", span = "day" },
  { kind = "todo", filter = "work", columns = ["ITEM", "EFFORT"], sort = ["effort-up"] },
]
```

Directory alias memberships share the unchanged 65536-entry discovery budget;
source contents remain deduplicated. Retained selected-path aliases are limited
to 4096 per document and 1 MiB overall. Limits are reported explicitly.


## Checked source actions

The agenda stays a read-only projection. `t`, `:`, comma and `e` invoke the
ordinary task, local-tag, priority and effort handlers on the selected source.
`C-c C-s` and `C-c C-d` schedule or set a deadline; `I`, `O` and `X` clock in,
out or cancel. These keys call the same compiled commands available in source.
Clock-out/cancel require the selected heading to own the active interval.

Before an edit, exact buffer revisions or bounded disk text/metadata checks
admit the real canonical buffer and source position. Changed evidence rejects
visibly; refresh and select again. Titles and cached row numbers do not grant
mutation authority. Original and peer points/scroll map through the ordinary
transactions. Changes remain unsaved canonical text; entering an agenda action
never saves a file implicitly. Source logging, blocking, DST choices and
cancellable notes retain their native behavior.

Use `m` to mark an entry and advance, `u` to unmark it, `*` to mark visible
entries and `U` to clear marks. Selection admits at most 256 distinct source
headings; duplicate calendar/block rows select the same target once. Section
headers and failure lines cannot select a different heading. Rebuilding or
changing the projection clears marks.

`B` selects `todo`, `tags`, `remove-tag`, `schedule`, `deadline`, `priority` or
`effort`, then accepts one common value in the ordinary source grammar. The
short choices `t`, `+`, `-`, `s`, `d`, comma and `e` are also accepted. Bulk tags
append local tags, preserving existing and inherited owners. Preparation checks
all targets before the first edit. Each accepted target uses a mapped canonical
anchor; earlier edits cannot redirect a later action to another heading.
Mandatory per-source notes and local-time choices can still ask for input.

Bulk completion opens a scrollable result panel with changed, unchanged,
rejected, cancelled or changed-with-failure evidence for every selected target.
One failure does not erase other outcomes. `q` returns to the agenda; `g`
refreshes sources. `org-agenda-action-results` reopens the retained results.
Cancellation preserves completed changes, cancels remaining targets, and never
claims rollback of already changed source. Marks clear after bulk completion.

Each source operation retains its ordinary history unit. `C-_` or
`org-agenda-undo` undoes the last retained source transaction only while its
final revision still matches; subsequent source changes require ordinary source
history inspection. Undo does not combine a multi-file bulk action into a false
atomic disk operation.

Reference: [remote editing and bulk selection](https://orgmode.org/manual/Agenda-Commands.html).

## Archived sources

`org.agenda.include_archived` defaults to `false`: calendar, TODO, search and
matching views omit ARCHIVE-tagged headings and their descendants. Set it to
`true` to inspect those selected subtrees. COMMENT trees remain excluded.
Changing archive tags makes retained source observations stale; refresh with
`g` to rebuild from canonical text under the selected policy.

External archive files are selected explicitly, like other agenda files.
There is no implicit archive-directory scan. Standard `.org_archive` files open
in native Org mode; explicit source selection uses unsaved canonical buffers
before disk text. Origin properties remain available to property matching and
source inspection after an external archive. A directory root still admits only
its immediate `.org` children; name an archive file directly to include it.
