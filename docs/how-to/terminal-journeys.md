# Run terminal journeys

Use Linux, Python 3 with its standard library, and the repository's Cargo
toolchain. Run commands from the repository root.

## Run acceptance workflows

```sh
python3 scripts/check-pty.py --suite acceptance
```

The command builds the editor and runs four connected journeys through real
controlling PTYs. Each session uses only the editor's normal launch arguments,
key bindings and implemented M-x commands.

| Journey | Checks |
| --- | --- |
| Document | New Unicode/CRLF text, safe save, nested shared views, bound/named motion, forward/backward search, literal replacement, undo aliases and reopen in another editor process. |
| Blocks | Linear copy/kill/yank and yank-pop, separate rectangle rows, cross-buffer padding, atomic history, buffer switch/close and exact BOM/separator bytes. |
| Workspace | Shared Rust views, tree rename/create/open/delete, visited-path saves, buffer inventory, guarded close and explicit recreation of retained deleted text. |
| Failure | Decision and path focus through zero/tiny resize, refused paste/default consent, canceled quit, failed required save, continued editing, independent save and accepted exit. |

Success prints `Acceptance PTY suite: 4 journeys passed, 0 skipped.` File checks
compare actual bytes; display checks inspect completed frames, bounded cursor
coordinates and observable selected-buffer state. Each accepted exit verifies
termios, bracketed-paste disable, visible cursor, attribute reset and alternate
screen restoration.

## Run terminal boundary regressions

```sh
python3 scripts/check-pty.py --suite terminal
```

This suite also builds the terminal fault probe and exercises individual input,
rendering, filesystem and cleanup boundaries. Both suites use owned temporary
files, finite eight-second session deadlines and bounded output. They wait for
observable conditions rather than fixed delays. A failed assertion or deadline
returns a failing exit status; cleanup reaps only the fixture's own process.

The acceptance suite sends CSI-u `ESC [ 47 ; 5 u` for C-/, legacy byte `0x1f`
for the shared C-_ undo report, NUL for C-SPC, and compatible Escape-prefixed
Meta reports including M-< and M->. These checks prove the reports handled by
the editor. Physical chords also depend on terminal and multiplexer delivery;
see [Terminal input](../reference/terminal-input.md). Key press/repeat/release
semantics and canonical state variants remain covered by native Rust tests.
