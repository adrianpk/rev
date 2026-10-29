# Org source blocks

Use `C-c '` (`org-edit-special`) on a complete `#+BEGIN_SRC` block, its body,
closing line or affiliated metadata. The body opens in a transient **Org Src**
buffer. Edit with ordinary text controls and undo. Available configured language
presentation is reused: the bundled Rust grammar provides **Org Src Rust**;
unknown languages fall back to text. No grammar is installed. An `org` body
remains literal code, with no heading, agenda or other Org editing actions.

Return with `C-c '` or `C-c C-c` (`org-edit-src-exit`). The complete body changes
as one undoable source transaction. Abort with `C-c C-k`
(`org-edit-src-abort`) to discard the draft and restore the originating view.
`C-g` cancels pending observation or header entry; inside the draft it cancels
ordinary input and keeps the draft. File, buffer, window and mode commands cannot
discard the active draft. Source changes or read-only state reject reinsertion
and retain the draft for inspection or abort.

The originating document remains authoritative and unsaved. Surrounding bytes,
BOM, source newlines and shared views retain their ordinary buffer semantics.
An unchanged return preserves the original body exactly, without a history item.
Drafts remove common space/tab indentation and use LF. Changed text restores the
common prefix and the opening line's newline style. A missing final draft newline
is supplied before the closing delimiter. Org-looking `*` and `#+` lines are
comma-escaped on return, including an embedded `#+END_SRC`; one corresponding
escape is removed on entry. Editing does not run code or save files.

## Names and headers

`C-c C-v I` (`org-babel-view-src-block-info`) inspects language, optional name,
and effective raw header data. `C-c C-v h` (`org-babel-insert-header-arg`) prompts
for one `:key value` argument and appends it to the opening line. Header edits use
ordinary source undo. Empty-language blocks can be edited as text; declare a
language before adding switches or header arguments.

Resolution applies built-in defaults, file `header-args` and language-specific
`header-args:<language>` properties, ancestor drawers from outer to inner, then
affiliated `#+HEADER:`/`#+HEADERS:` lines, then opening-line arguments. At each
property owner generic headers precede language headers. Header properties inherit
independently of `org.properties.inherit`; additive `header-args+` forms follow
ordinary property ordering. The defaults are `:session none`, `:results replace`,
`:exports code`, `:cache no`, `:noweb no`, `:hlines no` and `:tangle no`.

Ordinary keys use the last applicable value. Repeated `:var` declarations retain
ordered raw data for later consumers; variable expressions are never evaluated
or interpreted by editing. Values may contain balanced delimiters, quoted strings
and escaped characters. Unknown valid header names remain data. Malformed literals,
ambiguous local names or properties and incomplete blocks are rejected.

Observation uses the existing finite native structure limits and an owned job
with `org.limits.job_seconds`, revision checks and cancellation. Each header
argument line caps at 4096 bytes, combined headers and affiliated metadata at 64 KiB, effective
arguments at 256, nested value delimiters at 64, and names/languages at 128 bytes.
Header keys use lowercase ASCII letters, `-` or `_`. The complete updated source,
including its unchanged surrounding text,
stays within the native 8 MiB observation limit. Appended arguments must fit
the combined local header line and effective argument count before publication.

The workflows follow the Org manual's [source editing](https://orgmode.org/manual/Editing-Source-Code.html),
[block structure](https://orgmode.org/manual/Structure-of-Code-Blocks.html) and
[header arguments](https://orgmode.org/manual/Using-Header-Arguments.html).

## Tangling

Use `C-c C-v t` (`org-babel-tangle`) to extract the current canonical buffer,
including unsaved text. `C-u C-c C-v t` (`org-babel-tangle-single-block`) selects
only the block at point; named fragments elsewhere in the same document remain
available. Blocks without `:tangle` or with `:tangle no` are skipped, as are
COMMENT subtrees. Full source structure and metadata must be complete before
any output is admitted.

Declare `:tangle filename` explicitly. Relative paths use the associated Org
file's directory; absolute paths are allowed. `:tangle yes` derives the associated
source filename with a language extension: el for emacs-lisp/elisp, py for python,
rs for rust, js for javascript, c/cpp for C/C++, sh for bash/shell and the declared
language otherwise. Relative/default targets require an associated source file.
No block or file is executed.

Blocks for the same destination concatenate in source order. `:padline yes`
(default) inserts a blank line before each additional block; `no` disables it.
Bodies remove common space/tab indentation and one Org literal escape, preserve
relative indentation and Unicode, and use LF without a source BOM. Nonempty
outputs receive a final newline. `:mkdirp yes` permits missing destination
directories; the default `no` requires existing parents. Declarations for one
file must agree on this choice.

`:noweb yes`, `tangle`, `no-export` and `strip-export` expand literal `<<name>>`
references. `no` (default), `eval` and `strip-tangle` retain them literally when
tangling. A unique `#+NAME:` takes precedence over a `:noweb-ref` group. Groups
join matching blocks in document order using `:noweb-sep` (default newline).
Nested named references expand recursively; `:no-expand` keeps the selected
body or referenced fragment literal. Characters before a reference repeat on
inserted continuation lines, unless `:noweb-prefix no` disables that prefix.
One final fragment newline is removed before substitution, preserving the
reference line's suffix and newline. Unknown, ambiguous, cyclic, unclosed or
evaluated references such as `<<name(args)>>` reject the entire plan.

Native header consumers accept literal paths, identifiers, yes/no choices and
double-quoted strings with `\n`, `\r`, `\t`, `\\` and `\"` escapes. Executable
forms are rejected. Nonempty `:comments`, `:shebang` or `:tangle-mode`
transformations are unsupported and reject before writing. Root `:var` expansion
requires a language adapter and rejects unless `:no-expand` keeps the body
literal. Referenced fragments do not carry variable assignments into their
caller.

The whole plan is expanded and all targets are checked before any file write.
Symlink/non-regular destinations, symlink parents, parent traversal and aliases
of live source buffers or other outputs reject. Destinations cap at 4096 bytes.
Tangling admits at most 512 source blocks, 256 output files, 8 MiB of combined
output, 64 expansion levels and 100000 expansion steps. Protecting live input
files caps at 4096 distinct paths and 1 MiB of path bytes. The existing source
and header limits and configured owned-job deadline also apply.

Each existing output asks `yes/no` separately. An invalid answer repeats that
file's prompt; `no` records a declined replacement and continues. Source editing
and buffer/window changes remain held while tangling owns the source snapshot.
`C-g` cancels remaining work, waits for an in-flight write's actual outcome and
keeps completed files. Each file uses checked atomic publication, retains
existing permissions and rejects a destination changed since preflight. Files
are independent: a later failure does not roll back an earlier publication.
`org-babel-tangle-results` reports Complete, Uncertain or NotPublished with its
path and diagnostic. Tangling never acknowledges the source as saved or changes
its text, undo history, BOM or file newline format. Read-only source can be
extracted. Idle tangling adds no deadline.

See the Org manual's [tangling](https://orgmode.org/manual/Extracting-Source-Code.html)
and [noweb reference syntax](https://orgmode.org/manual/Noweb-Reference-Syntax.html).
