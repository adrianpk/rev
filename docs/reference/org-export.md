# Checked Org export snapshots

The native export preparation layer captures one canonical buffer revision,
including unsaved text, BOM metadata and exact newlines/EOF. Its request selects
a native backend, whole buffer, checked region or subtree at a byte position,
optional typed options, and one captured operation time/zone. Native forms,
links, tables, footnotes, blocks, names, captions and affiliated attributes stay
available to renderers. Projected byte offsets belong to export text; original
source evidence retains its own real buffer revision or disk observation.
Preparation has no source edit, save, destination publication or execution effect.

## Formats and declarations

`org.export.default_format` defaults to `html`. `org.export.backends` selects
1..11 unique native formats: text (also accepted as ascii), utf8, org, markdown,
html, latex, beamer, odt, texinfo, man and icalendar. The default must be enabled.
These are fixed typed choices. PDF/tools and loadable backend names are not
native choices. Changing configuration does not start an export.

File-preamble TITLE, AUTHOR, EMAIL, DATE and LANGUAGE values are retained as
metadata, at most 4096 UTF-8 bytes each before and after macro expansion.
OPTIONS admits `toc`, `num`, `author`, `title`, `todo` and `tags`, each with
`t`, `nil`, `1` or `0`. All default true. Multiple OPTIONS lines compose by leaf;
the last assignment wins. An explicit request options value overrides the whole
file options selection. Unknown options and unsupported value forms reject.

## Includes

Only native INCLUDE keyword forms resolve files. Source/example/export/comment
blocks, drawers, fixed-width text and comments do not activate directives.
A quoted local path resolves against the containing source's canonical directory;
an unnamed buffer requires an absolute path. Nested includes resolve against
each included file. No environment expansion, network fetch or implicit scan
occurs. Existing canonical buffers supply their captured unsaved text, including
physical aliases. Otherwise the regular file is read through checked UTF-8/NUL
and BOM ingress, with cancellation and consistency checks.

The admitted forms are:

```org
#+INCLUDE: "chapter.org"
#+INCLUDE: "chapter.org" :lines "2-5"
```

Line numbers are one-based; the end is exclusive. An empty start means one and
an empty end means EOF. Outside ranges and selections that split literal native
forms reject. Other include options, search suffixes, escaped path syntax,
missing/nonregular/dangling sources, physical cycles and incomplete or malformed
native structure reject the entire preparation. Repeated noncyclic files count
as separate include occurrences; their original source evidence is retained once.

Only includes within the selected export scope are read. File metadata and
macro declarations remain file-wide inputs. Includes retain original content
newlines; an included result without a final newline is separated from
following source by the original directive's newline.

## Text macros

Native MACRO definitions contain a name and text replacement. Names use 1..64
ASCII letters, digits, underscores or hyphens. Definitions and invocations
admit at most 4096 UTF-8 bytes each; at most 128 distinct names are retained.
Later declarations replace earlier declarations of the same name. File-wide
root declarations remain available to selected regions and subtrees; included
native declarations can replace them in document order.

```org
#+MACRO: greet Hello $1
{{{greet(World)}}}
```

Arguments are comma-separated, at most nine; backslash escapes a comma or
backslash. `$1`..`$9` substitute arguments and `$$` supplies a literal dollar.
Missing arguments, unsupported escapes/parameters, malformed invocations,
cycles, nesting beyond 16 active calls and more than 100000 invocations reject.
Text expansion can nest. Definitions beginning with `(eval` are unsupported.
No source block, file-local Lisp or text replacement is evaluated as code.

Unknown names remain literal with a diagnostic, at most 128 distinct diagnostics.
Code/verbatim objects, links and literal structural contexts remain unchanged.
Recognized definition lines are omitted from the projection. Metadata text values
expand independently within their 4096-byte bound. Expansion occurs after include
resolution; generated text cannot initiate another file read.

## Bounds and freshness

`org.limits.include_depth` defaults to 16, maximum 64; the root is excluded.
`include_files` defaults to 128 occurrences, maximum 512. `expansion_bytes`
defaults to 1048576, maximum 8388608; the complete include/macro projection and
aggregate prepared macro replacements must fit. Original sources also obey
`agenda_file_bytes` and `agenda_total_bytes`, counting a leading BOM. Native
analysis admits at most 8 MiB per source and 200000 aggregate observed source
nodes. Generated native structure must fit the same analysis profile.
`export_bytes` defaults to 16777216, maximum 134217728; the immutable snapshot
retains this separate limit for its renderer. Limits reject without truncation.

One `job_seconds` deadline and cancellation cover all preparation stages. Work
uses the ordinary owned Job slot; cancellation does not release that slot until
its worker is reaped. Already blocked kernel IO has no hard interruption promise.
No output file or parent directory is created by preparation.

Before publication, the editor owner rechecks every canonical buffer revision
and file association. A newly visited canonical source supersedes retained disk
authority, including aliases. The owned worker separately rechecks disk source
observations. Stale evidence requires a new snapshot; it cannot be relabeled as
current text or permission to write. Destination selection, confirmation and
publication remain with the ordinary file owner and renderer.

## Native text and Org file commands

Run `org-export-to-text`, `org-export-to-utf8`, `org-export-to-markdown` or
`org-export-to-org` through M-x. Enter `buffer`, `subtree` or `region` scope;
empty selects the buffer. Region requires an active region. Enter an OPTIONS
selection or leave it empty to use file declarations, then enter the literal
local destination path. Relative output paths use the source directory. No
parent directory is created. C-g cancels prompts or requests cancellation of
owned work; completed file publication remains visible.

Existing output files require an explicit `yes` or `no` decision tied to the
observed target. Any live canonical file or input source, including physical
aliases, is unavailable as a destination. Sources and target evidence are
rechecked before atomic publication. Export never changes source association,
modified state, undo history or editing positions. Read-only sources can export.

ASCII rejects any unrepresentable Unicode scalar with its code point and output
byte offset; it never substitutes replacement characters. UTF-8 and Markdown
retain Unicode. Markdown converts native headings, markup, lists, tables, links,
notes and literal code blocks; code fences cannot be closed by embedded shorter
runs. Literal paragraph markup is escaped. The first data row becomes a
Markdown table header; all data rows and cell values remain present. Org retains
the prepared native syntax and source newlines/EOF. Includes and text macros
have already been resolved by preparation.

The title, author, numbering, table of contents, TODO and tag options apply to
the selected output. Region text has no generated title/contents header. A
subtree starts at heading level one. Explicit Org option overrides and inherited
subtree options are written as a data-only OPTIONS declaration. Unknown keyword
or dynamic forms retain a literal representation with diagnostics. Named and
inline footnotes retain references and bodies; missing or duplicate definitions
reject the output rather than dropping content.
