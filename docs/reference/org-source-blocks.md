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

## Execution

`C-c C-v e` (`org-babel-execute-src-block`) selects the block or named call at point;
`C-c C-c` has the same meaning on either construct. Native execution requires an
explicit matching profile in `org.source.programs`; no language runner is
installed or inferred. If several profiles match the exact declared language,
select one by name. `org.source.confirm_execution` defaults true. Each launch
asks yes/no with the profile, language and executable. Invalid answers repeat
confirmation, and decline or `C-g` before launch creates no process/workspace.
`:eval no` and `never` always reject, including inherited declarations.
`:eval query` always asks. `yes`, `no-export`, `never-export` and `query-export`
permit interactive execution according to configuration. Other values reject.
Visiting, coloring, tangling, exporting and recalculating never execute blocks.

A minimal explicit shell profile is:

```toml
[[org.source.programs]]
name = "shell"
language = "sh"
action = "execute"
executable = "/bin/sh"
argv = []
delivery = "stdin"
cwd = "workspace"
result = "output"
seconds = 30
output_bytes = 1048576
```

Records reject unknown fields. Names/languages use 1..128 ASCII letters, digits,
`_`, `+` or `-`; names are unique. `action` supports `execute`. `delivery` is
`stdin` (default) or `file`; `cwd` is `workspace` (default) or `source`, which
requires an associated Org file. Relative executables containing a slash resolve
against the configuration file that assigned the program array; bare executable
names use PATH. There is no shell interpolation of profile tokens. The executable
can also be the whole `{output}` token for an explicitly compiled owned program.

Arguments accept whole `{input}`, `{output}` and `{workspace}` substitutions for
an owned source file, compiled program path and private workspace. File delivery
requires `{input}` in the final argv or an explicit compile step. `steps` is an
optional array of up to three `{ executable, argv }` records, run sequentially
before the final executable. Every step shares the operation deadline and
combined output budget. There are at most 64 executable/argv tokens per step,
4096 bytes per token and 64 KiB of executable/argument data per profile.
`seconds` admits 1..600 and `output_bytes` 1..16777216; the effective bound is the
smaller profile/global `org.limits.job_seconds` or `job_output_bytes` value.
`result` declares `output`, `value` or `table`; this operation retains raw bytes
and publishes successful results as a checked source transaction.

The selected unsaved body uses native indentation, LF and literal-unescaping
rules. `:noweb yes`, `eval`, `no-export`, `strip-export` and `strip-tangle` expand
bounded literal references; `no` and `tangle` retain them literally. `:no-expand`
keeps literal code. Cycles, evaluated references and unresolved names reject
before launch. Expansion retains the existing 512-block, 64-level and 100000-step
bounds and must fit `org.limits.expansion_bytes`. Persistent sessions, `:dir`,
prologue/epilogue transformations, executable post-processing and cache requests
reject before launch.

The operation owns one worker, a mode-0700 workspace, mode-0600 input and all
child process groups. Staged input is delivered by a regular stdin file or a
literal argv path. HOME, all XDG home/runtime directories and TMPDIR point inside
the private workspace. Programs retain normal host authority; these settings
provide lifetime and file ownership rather than a sandbox. Source input and
buffer/window changes remain held until owned cleanup finishes. `C-g` stops
and reaps the direct child and stops its owned group; time/output failures do
the same. Only the owned workspace is removed. No idle polling remains.

`org-babel-execution-results` inspects the last state, exit code, byte counts,
stdout, stderr or retained source. Inspection displays an escaped 1024-character
prefix; the public native result retains bounded actual bytes. Nonzero exits,
missing programs, cancellation, timeout, cleanup failures and stale contexts
remain distinct from successful execution. Failed or cancelled work retains the
selected source definition and leaves existing RESULTS, buffer history and file
saved state untouched. Source context is revalidated before confirmed launch
and after completion. Programs can perform their explicitly confirmed host
operations; result publication changes the unsaved source through ordinary undo
without saving the file.

See the Org manual's [evaluation and eval restrictions](https://orgmode.org/manual/Evaluating-Code-Blocks.html).

### Calls and results

A named call uses `#+CALL: name(arguments)` or
`#+CALL: name[inside headers](arguments) end headers`. Inline
`call_name(arguments)[end headers]` at point uses a verbatim results macro.
Local named blocks take precedence. `org.source.library_files` declares at most
32 explicit Org files, resolved relative to the assigning configuration file.
They are read only during an explicit unresolved call, through bounded canonical
source observations; open unsaved text takes precedence over disk. A qualified
`file.org:name` call must match a declared file. Missing or ambiguous definitions
reject. There is no startup ingest, search path or arbitrary file loading.

`:var name=literal` defaults and call `name=literal` arguments admit quoted
strings and finite numbers. Inside headers override target headers; call
arguments override defaults by variable name. Shell-family blocks receive safely
quoted assignments. Other adapters consume each variable through an explicit
whole `{var:name}` argv token, preserving one argument without shell evaluation.
Expressions, recursive calls and executable argument forms reject. The selected
target's no/never restriction cannot be removed by a call override. Query
restrictions and default confirmation remain effective for library calls.

A profile's `result` declares the external adapter contract: `output` returns
stdout text; `value` returns an externally collected scalar; `table` returns
rectangular tab-delimited rows. Rev does not synthesize a language interpreter
or infer a function-return wrapper. A collection header must agree with the
selected adapter. Only the final step's stdout becomes canonical result data;
all step streams remain available for inspection.

`:results` supports output/value collection, table/vector or scalar/verbatim
format, drawer/html/latex/org/code wrapping, and replace/append/prepend handling.
Silent/none/discard retain inspected output without changing source. Call end
headers support result formatting/handling. Unsupported options reject before
execution. UTF-8/control admission, rectangular tables (256 columns, 10000 rows),
wrapping delimiters and the native 8 MiB updated-source ceiling are checked before
publication. Inline publication requires one verbatim scalar.

The editor replaces only recognized adjacent RESULTS content: fixed-width text,
tables, drawers or delimited blocks. Ambiguous or unclosed old content is retained
with a diagnostic. Replacement drops obsolete result hashes. A successful edit
is one undo transaction and preserves unrelated bytes, BOM, line endings and
shared source views. Source and library observations are checked again before
launch and publication; failed, cancelled, stale and read-only publication keeps
prior results. See the Org manual's [result options](https://orgmode.org/manual/Results-of-Evaluation.html)
and [Library of Babel](https://orgmode.org/manual/Library-of-Babel.html).
