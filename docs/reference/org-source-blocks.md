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
with `org.limits.job_seconds`, revision checks and cancellation. Each header line
caps at 4096 bytes, combined headers and affiliated metadata at 64 KiB, effective
arguments at 256, nested value delimiters at 64, and names/languages at 128 bytes.
Header keys use lowercase ASCII letters, `-` or `_`. Source and reinserted body
bytes stay within the native 8 MiB observation limit.

The workflows follow the Org manual's [source editing](https://orgmode.org/manual/Editing-Source-Code.html),
[block structure](https://orgmode.org/manual/Structure-of-Code-Blocks.html) and
[header arguments](https://orgmode.org/manual/Using-Header-Arguments.html).
