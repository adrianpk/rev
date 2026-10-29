# Org agenda queries

`M-x org-todo-list`, `M-x org-search-view` and `M-x org-tags-view` open native
read-only query agendas. Within an agenda, use `t`, `s` or `m` respectively.
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
