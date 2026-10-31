# Register grammar assets

Use a compatible, user-trusted precompiled native library and two matching
Tree-sitter queries. Native libraries execute code in the editor process;
ABI and query validation do not sandbox that code. Rev does not download,
compile or install grammars.

## Place a versioned asset set

Before starting rev, place the library and UTF-8 queries in a new versioned
directory, such as `~/.local/share/rev/grammars/python-0.25.0/`:

```text
python-0.25.0/
├── parser.so
├── highlights.scm
└── completions.scm
```

When `XDG_DATA_HOME` is set, use `rev/grammars` beneath that directory. Relative
registration paths resolve there. Use regular files and keep relative paths
and their symlinks within that root. Absolute file paths are also accepted.

Prepare queries using the supported
[external query profile](../reference/syntax.md#external-query-profile).
Highlights and completion queries are both required; arbitrary upstream query
files may use captures or operations outside that finite profile.

## Register and check

Add this complete registration to your startup `config.toml`:

```toml
[language.python.external]
parser = "python-0.25.0/parser.so"
symbol = "tree_sitter_python"
highlights = "python-0.25.0/highlights.scm"
completions = "python-0.25.0/completions.scm"
```

Use the exported symbol and paths belonging to your actual assets. Restart rev
and open a `.py` file, whose existing language association selects `python`.
First use admits the library and both queries on a temporary worker. Successful
admission shows `External` in the language status. Try
`M-x completion-at-point` beside a declaration observed by your query.

An invalid library, ABI, symbol or query preserves bundled syntax when available;
an unbundled identity remains editable as `Text`. A completed failure is cached
for that session. Correct the assets or registration and restart to retry.

For an upgrade, install another versioned directory, change all relevant paths
and restart. Never overwrite a library mapped by a running editor. Loaded native
mappings remain for the process lifetime. See
[External grammar assets](../reference/syntax.md#external-grammar-assets)
for path/query limits, fallback and replacement details.
