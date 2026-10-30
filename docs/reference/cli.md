# Command-line interface

```text
rev [file]
```

| Invocation | Result |
| --- | --- |
| `rev` | Open an unnamed `*scratch*` buffer. Saving asks for a filename. |
| `rev notes.txt` | Open an existing UTF-8 file, or an empty visiting buffer if it does not exist. |
| `rev src/main.rs` | Open a file relative to the shell's current directory. |

An absolute file path is also accepted. The parent directory must exist when
saving a new file. Opening an absent file creates no disk entry until save.

Only zero or one argument is accepted. There are no command-line options:
`--help` and `--version` are treated as filenames, not flags. Multiple arguments
produce `usage: rev [file]` and exit with failure.

Directory arguments, including `rev .`, are not supported in this version.
Use `C-x t t` for the [file tree](file-tree.md). In `C-x C-f`, accepting a
directory continues [path completion](prompts.md) rather than opening a directory
as an editing buffer.

rev requires an interactive terminal. It does not read a document from piped
standard input. Normal operation is offline. Startup configuration is discovered
from the environment; there is no configuration-path command-line option.
See [startup configuration](defaults.md#startup-configuration).

File input must be UTF-8 without NUL bytes. Existing BOM and line-ending details
are preserved; see [text and files](defaults.md#text-and-files).

For installation and the Linux utility with the same name, see
[Install rev](../how-to/install-rev.md).
