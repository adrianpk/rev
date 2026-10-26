# rev

A terminal text editor written in Rust, with rope-backed buffers and editing
conventions inspired by GNU Emacs.

The product baseline is being specified before implementation:

- [Editor specification](ops/default/spec/rev-editor.md)
- [Editor model](ops/default/spec/rev-editor-model.md)

Implemented input behavior is documented in the [terminal input reference](docs/reference/terminal-input.md).
Use `M-x` to run implemented commands by name; see the [command reference](docs/reference/commands.md).

## License

Copyright (C) 2026 rev contributors.

Licensed under the GNU General Public License, version 3 only
(`GPL-3.0-only`). See [LICENSE](LICENSE).
