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

## Run configuration and UI workflows

```sh
python3 scripts/check-configuration-ui.py --suite terminal
```

This command builds the editor and its terminal fault probe, then runs ten
owned PTY sessions with isolated HOME and XDG paths:

| Sessions | Checks |
| --- | --- |
| Configured RGB and NO_COLOR | Visible tab cells and cursor columns, palette/key overrides, M-x candidate/completion/history aliases, retained path errors, directory/Unicode completion, freely typed new filenames and an explicit BOM/CRLF save. |
| Missing and invalid configuration | Exact complete first-start configuration or a nonfatal invalid-source diagnostic, complete default tab/key/palette/format behavior and passive data/state/cache roots. |
| Workspace RGB and NO_COLOR | Configured tree and nested split geometry, shared edit/undo, distinct buffer tab policies, horizontal/page/EOF boundaries, completion through zero/tiny/growing resize, exact canceled canvas/cursor restoration and editing-pane cycling. |
| Output error and unwind | Expected nonzero fault-probe exits, ordered cursor/text color reset, termios/paste/screen restoration and complete owned-child/PTY cleanup. |

The combined RGB and NO_COLOR journeys begin without configuration and connect
first-start publication, ordered arrows/goto-line, nested shared panes/tree,
independent numbering and status, pointer wheel/drag, conversion and exact
save/history. They also verify failed/canceled prompts, search/rectangle overlays,
resize and C-g capture cleanup, runtime mouse control and complete terminal
restoration. Each of their ten stage receipts must precede its named completion.

Success prints `Configuration/UI terminal suite: 10 sessions passed, 0 skipped.`
Assertions inspect complete visible rows, candidate selection, cursor positions
and exact bytes/entries in the owned fixture. After the expected first-start configuration publication, observation, completion
and visits preserve the fixture; explicit saves change only the expected files.
Every process has the existing eight-second session deadline and 1-MiB output
limit. Fixture snapshots allow at most 32 entries and 64 KiB per file. Cleanup
reaps only the owned child and verifies both PTY descriptors are closed.

For a focused rerun, select one case, such as:

```sh
python3 scripts/check-configuration-ui.py --suite terminal --case workspace
```

Available cases are `configured`, `missing`, `rejected`, `workspace`, `io`,
`unwind` and `combined`. Configured, workspace and combined cases each run RGB
and NO_COLOR variants.

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
