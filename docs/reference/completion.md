# Native completion

Use `M-TAB` or `M-x completion-at-point` to request declarations from the current
Rust buffer, including unsaved text. Terminal or desktop shortcuts may intercept
M-TAB; the named command uses the same handler. The editor uses its retained
Rust syntax tree and explicit declaration queries. No server or file scan runs.

The compact, single-border list appears below or above the source cursor within
its pane. The first candidate is selected. Up/Down or C-p/C-n changes selection;
Tab or Return inserts it. Escape or C-g cancels. These keys keep their ordinary
behavior outside an active list. Unrelated editing, movement, prompts and pane
changes dismiss the list before their normal action. Resizing retains a current
selection; a pane too small for the border and a row hides the overlay.
Source focus and document layout stay in place. Selected rows use the selection
palette; the border uses the active border role. NO_COLOR keeps reverse video.

Candidates are observed declaration names, with their kind shown separately.
The display annotation is never inserted. Lowercase input matches without case
sensitivity; uppercase input requires matching case. Prefix matches precede
ordered subsequences, with nearer declarations and stable lexical ties next.
Equal insertions are deduplicated. A single candidate still requires acceptance.
No matches produce a notice and no popup. Manual invocation also works in comments
and strings, subject to the same declaration coverage.

Acceptance replaces the full identifier around point with literal text through
the ordinary edit owner, as one undo operation. Read-only buffers and obsolete
view, point, revision or invocation identities cannot publish that edit.
Cancellation and failed admission create no history entry. Native completion
does not expand snippets, execute commands or save a file.

This manual Rust path observes direct function, type, trait, module, field,
constant, simple local and parameter declarations. Destructured bindings,
scope-aware coverage and other open buffers belong to the broader native
completion delivery. An observed name does not imply type or member resolution.
Automatic invocation and its controls are not part of this manual path.

Extraction admits at most 2 MiB and 4096 symbols in the current buffer, with
512-byte labels and at most 200 displayed candidates. Parse/query callbacks
share a 50 ms work deadline. Exceeding a work bound reports unavailable or
limited assistance without editing source. Extraction occurs on explicit
invocation; rendering and cursor movement only observe the retained list.

See [Syntax highlighting](syntax.md) for Rust grammar admission and fallback,
and [Bindings](bindings.md) for the complete command catalog.
