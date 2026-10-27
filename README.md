# rev

A terminal text editor written in Rust, with direct Unicode editing and Emacs
bindings. It supports multiple buffers and nested windows, guarded file saves,
linear and rectangular selections, literal search/replacement, a keyboard file
tree and bundled Rust highlighting. Normal editing works offline.

## Build and run

Use Linux, a terminal, Rust 1.98 or newer, Cargo and a C compiler for the bundled
grammar. Building needs the pinned Cargo dependencies; running needs no network,
interpreter, language server or grammar installation.

```sh
cargo build --release --locked
./target/release/rev
./target/release/rev notes.txt
```

`rev` accepts zero arguments or one file path. An absent file opens an empty
visiting buffer; saving creates it. Install the built executable on your PATH
when you want to invoke it as `rev`.

## First commands

Type ordinary text directly. `C-` means Control; `M-` means Alt/Meta or a
compatible Escape prefix. A space between chords means press them in order.

| Keys | Action |
| --- | --- |
| `C-x C-f` | Find or revisit a file. |
| `C-x C-s` | Save; unnamed text asks for a path. |
| `C-x C-w` | Write to another path. |
| `C-x b`, `C-x C-b` | Switch buffers or select from the buffer list. |
| `C-x 2`, `C-x 3`, `C-x o` | Split below/right and cycle editing windows. |
| `C-SPC`, `M-w`, `C-w`, `C-y` | Set mark, copy, kill and yank linear text. |
| `C-s`, `C-r`, `M-%` | Search forward/backward or query-replace. |
| `C-x u`, `M-x undo-redo` | Undo and redo. |
| `M-x` | Find and run a command by name. |
| `C-g` | Cancel the current interaction or clear the active region. |
| `C-x C-c` | Exit after save/discard decisions for modified buffers. |

Read the [edit/save procedure](docs/how-to/edit-files.md),
[complete bindings](docs/reference/bindings.md) and
[terminal input requirements](docs/reference/terminal-input.md). Save conflicts
need explicit `y` or `n`; Return has no affirmative default. `C-g` cancels.

## Documentation

- [Documentation index](docs/index.md)
- [Scope and defaults](docs/reference/defaults.md)
- [Commands and M-x](docs/reference/commands.md)
- [Recover a failed operation](docs/how-to/recover-edits.md)
- [Change history](CHANGELOG.md)

For contributors, the [accepted specification](ops/default/spec/rev-editor.md)
and [state model](ops/default/spec/rev-editor-model.md) define the implementation
contracts. [Terminal journeys](docs/how-to/terminal-journeys.md) describe native
and real-terminal validation. Python 3 is needed for those scripts.

## License

Copyright (C) 2026 rev contributors.

Licensed under the GNU General Public License, version 3 only
(`GPL-3.0-only`). See [LICENSE](LICENSE).
