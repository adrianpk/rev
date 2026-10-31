# Filesystem candidate provider

The library's `rev::file::completion` module owns explicit directory observations
for path completion. It does not open editing buffers or create files.

| API | Contract |
| --- | --- |
| `Query::parse(base, home, draft)` | Split literal path text into an absolute directory and a component prefix without IO or environment access. |
| `Query::directory()`, `Query::prefix()` | Retain the requested directory component and exact UTF-8 filter text. |
| `Provider::observe(directory)` | Read one explicitly supplied absolute directory into a bounded, sorted snapshot. |
| `Provider::matching(prefix)` | Filter retained names by case-sensitive literal byte prefix, without filesystem access. |
| `Provider::refresh()` | Explicitly reread the same directory; replace the snapshot only on success. |
| `Provider::entries()`, `Provider::directory()` | Expose the retained entries and their directory identity without IO. |
| `Provider::name_bytes()`, `Provider::diagnostic()` | Report retained raw name bytes and any incomplete-list indication. |
| `Candidate::name()`, `Candidate::path()` | Expose the exact supported name and original opening path, independently of presentation. |
| `Candidate::label()` | Return a terminal-safe full label; display clipping must not change opening identity. |
| `Candidate::is_directory()`, `Candidate::is_symlink()` | Expose the entry's observed type without following child symlinks. |

## Find-file navigation

`Navigation::new(base, home)` owns a single component snapshot, editable draft,
selection and scroll origin. `update(draft, false)` changes the filter and resets
selection only after an actual text change. A directory transition observes the
new component; cursor movement and redraw do not refresh it.

`navigate(previous)` moves a bounded selection without changing the draft.
`complete()` explicitly refreshes and returns a selected/unique name or an
extended whole-grapheme shared prefix. Selected identities survive sorting shifts;
a missing selected name reports an error. Directories retain a trailing separator.
The prompt owner enforces the configured scalar bound before installing text.
Incomplete snapshots cannot imply unique/shared completion without an explicit
selection. Unsupported names never become replacement opening paths.

`requested_rows(limit)`, `reveal(rows)`, `selected()` and `top()` provide presentation
state for the common completion panel. Diagnostics reserve a visible row within
that limit; zero candidate rows retain selection and origin. The common display
owner clips labels independently of paths. `typed_path()` resolves the retained
literal draft against its explicit roots.

Native Return resolves the explicitly navigated candidate from its original path;
without navigation it uses the literal draft, including new names. Acceptance
checks only that requested identity, following symlinks there rather than during
listing. The session prepares an accepted directory's bounded tree observation
before closing the prompt and replacing its browsing context. An empty draft
accepts the prompt base. Files pass to the existing canonical visit owner. Missing selected paths, unsupported encodings, denied access and file
ingress errors retain the prompt and active buffer. Observation, completion and
acceptance never publish saves or create filesystem entries.

## Queries and identity

Relative text uses the explicit absolute base. An absolute draft supplies its own
directory. `./` and `../` retain OS path semantics, including parents reached
through symlinks. A trailing slash leaves an empty component prefix.

Only leading `~/` expands, using the caller's explicit absolute home directory.
Missing or relative home roots reject that expansion. Other tilde forms,
variables, glob characters, backticks and shell-looking text remain literal.
Spaces and Unicode retain their exact bytes. NUL and a relative base reject with
`InvalidInput`. Parsing does not require the requested directory to exist.

Candidate paths are retained independently of labels. Directory labels end in
`/`; the marker is not appended to the opening path. Control characters are
escaped in labels, while ordinary Unicode remains intact. The display owner can
clip these labels at whole-grapheme boundaries without modifying path identity.

Non-UTF-8 names remain visible with an `unsupported non-UTF-8 filename` indication.
Their `name()` is `None`, and `path()` returns `InvalidData`; no lossy replacement
path is offered. Raw bytes still participate in ordering and prefix filtering.

## Observation and limits

Every observation processes at most 4096 entries and retains at most 512 KiB of
candidate-name bytes. A name that cannot fit ends observation before its type or
candidate is retained. Reaching either exact limit also marks the list incomplete
without peeking at another entry. The diagnostic is:

`Incomplete candidate list: directory observation limit reached`

An incomplete snapshot contains the bounded portion returned by directory
enumeration, sorted after observation. It does not claim to contain the first
4096 names of a fully sorted, unbounded directory listing.

Directories precede other entries; each group sorts by case-sensitive filename
bytes. Hidden names are included. Matching is a literal prefix operation;
`minibuffer.match` remains a command-candidate preference.

Only the current directory is enumerated. Child directories are not read, and
directory, dangling and looping child symlinks remain leaves. An explicitly
requested directory path can itself contain symlink components; their
destinations are not resolved speculatively from listed children.

Filtering, label access and entry/type access use cached memory. Changes on disk
become visible only through a new observation or explicit refresh. IO failures
retain their error kind and requested-directory context. Failed refresh preserves
the previous snapshot and its directory identity. There is no watcher or
background traversal.
