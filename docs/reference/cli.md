# Command-line interface

```text
rev [file-or-directory]
```

| Invocation | Result |
| --- | --- |
| `rev` | Open an unnamed `*scratch*` buffer. Saving asks for a filename. |
| `rev notes.txt` | Open an existing UTF-8 file, or an empty visiting buffer if it does not exist. |
| `rev src/main.rs` | Open a file relative to the shell's current directory. |
| `rev .` | Browse the current directory with its immediate tree entries expanded. |
| `rev notes` | Browse an existing directory, or visit an absent file named `notes`. |

Absolute file and directory paths are also accepted. The parent directory must exist when
saving a new file. Opening an absent file creates no disk entry until save.

Only zero or one argument is accepted. There are no command-line options:
`--help` and `--version` are treated as filenames, not flags. Multiple arguments
produce `usage: rev [file-or-directory]` and exit with failure.

Directory arguments show and focus the [file tree](file-tree.md), retaining
the editing buffer. Child directories stay collapsed until explicitly opened.
The accepted directory root remains selected across later file visits. In
`C-x C-f`, Tab completes a directory and lists its children inside the prompt;
Enter accepts it and opens its tree. Neither route changes the process working
directory or loads a directory as document text. See [path completion](prompts.md).

rev requires an interactive terminal. It does not read a document from piped
standard input. Normal operation is offline. Startup configuration is discovered
from the environment; there is no configuration-path command-line option.
See [startup configuration](defaults.md#startup-configuration).

File input must be UTF-8 without NUL bytes. Existing BOM and line-ending details
are preserved; see [text and files](defaults.md#text-and-files).

For installation and the Linux utility with the same name, see
[Install rev](../how-to/install-rev.md).
