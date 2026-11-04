# Run terminal journeys

Use Linux and the repository's Cargo toolchain. Bookmark and completion journeys
use Bash, util-linux `script`, `stty` and GNU `timeout`. Completion also needs
`rg` and a C compiler (`cc`) for its owned native grammar fixtures. The Python
suites below use Python 3 with its standard library. Run commands from the
repository root.

## Run native completion acceptance

```sh
bash scripts/check-completion.sh
```

This builds the normal release editor, native acceptance example and existing
source checker with two build jobs. It applies the unchanged source-quality
rules to every production Rust file, marks actual crate roots and checks the
acceptance example modules. Isolated HOME/XDG roots and source files preserve
user data. Locked Rust/Python grammar fixtures are compiled before editor
construction; the running editor never compiles or downloads an asset.

Native journeys check all ten bundled parsers, Unicode replacement, six Org
contexts, actual external Python admission and bundled fallback. They connect
manual selection/cancel/acceptance, automatic typing, resize, one-step undo/redo
and unchanged disk bytes. A real PTY checks the same editing interaction,
idle silence and terminal restoration using normal editor arguments. It keeps
the existing eight-second child deadline and 1 MiB output limit. Cleanup reaps
only owned children and removes the temporary fixture.

The output reports actual release bytes, delta and ratio against the recorded
13,326,584-byte normal-release baseline, bounded cross-buffer/symbol/source
observations, a 128-event burst over 1024 declarations, peak Linux RSS and idle
CPU ticks. Timing observations depend on the machine; they do not establish a
latency guarantee. The historical grammar-only ratio is comparative evidence,
not a release-size ceiling. Strict limits remain those in
[Native completion](../reference/completion.md).

## Run bookmark restart acceptance

```sh
bash scripts/check-bookmarks.sh
```

This builds the repository's release editor and native acceptance example.
An owned temporary HOME and XDG roots isolate all files and stores. Native
sessions connect set/refusal, edits and undo, save, alias identity, shared views,
list deletion, restart and checked context, folded targets and protected failure
states. Two real PTY editor processes verify persisted jumps and deletion across
a restart, independent markers and numbers, resize, cancellation, idle silence
and restored terminal attributes.

Each PTY child has an eight-second deadline and 1 MiB output limit. Read-only
list frames are complete screens with a hidden cursor. Cleanup reaps only owned
children and readers and removes their temporary roots. Installed executables
and user stores are not used.

The same entrypoint reports measured admission/rendering for 1024 bookmarks,
large-source reversal and resize, cooperative relocation, peak Linux RSS and
idle behavior. These are local observations; the strict resource and store
bounds are documented in the [bookmark reference](../reference/commands.md#named-bookmarks).

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

## Run joint completion acceptance

```sh
bash scripts/check-lsp.sh
```

This requires the installed `rust-analyzer`, local Rust toolchain and cached
locked dependencies. The wrapper builds the normal release editor, checks every
production Rust source and the acceptance roots, then runs isolated real-server
and controlled-server completion journeys. It also reruns native language/Org/
external-grammar completion, resource bounds and the native PTY journey without
changing their historical evidence.

The real Rust fixture has a 35-second semantic readiness deadline and a
60-second outer child-process deadline. It requires a semantic field from
unsaved text, one undo and unchanged disk bytes. Controlled peers prove resolved
imports, empty/failure fallback, exceptions, sharing, restart and cleanup. The
native and LSP terminal sessions each retain the eight-second and 1 MiB output
limits and verify terminal restoration. Output records executable bytes, native
editor resources and installed-server resources separately. Only owned sources,
HOME/XDG roots, processes and PTYs are created and cleaned up.

## Run guided server management acceptance

```sh
bash scripts/check-lsp-installation.sh
```

This Linux x86-64 journey uses the installed Rust toolchain, rust-analyzer,
gopls, Go, Bash, gzip/SHA-256 tools, `rg`, util-linux `script` and GNU `timeout`.
It builds the normal release editor and new driver with two jobs, then checks
the affected acceptance sources. It needs cached locked Cargo dependencies;
it uses no Python tooling or package network acquisition.

One controlled asset is compressed from the real installed Rust server and
verified through the actual managed acquisition/probe path. Its local digest
establishes the fixture bytes, not an upstream release identity. Native journeys
in both profiles install/update/remove that payload, preserve unsaved text and
settings, and use actual Rust/Go servers for unsaved-only semantic fields, one
provider and one-step undo. A separate protocol peer establishes routing for an arbitrary
language without a grammar; it does not establish that language's
semantic quality. Explicit upstream package probes remain separate from this gate.

The real management PTYs cover prompt cancellation, setup, saved selection,
inventory, disabled analysis, idle silence, unchanged source and terminal
restoration. Each retains the existing eight-second/1-MiB boundary. Native
cases have a 90-second outer guard and finite management/semantic observations.
Output separates driver/server RSS, idle CPU ticks, payload/editor bytes, fixture
disk usage and runtime. Only owned roots, children and PTYs are removed after
their checks finish. The script does not run the combined delivery-set gate.

## Run joint Pi and reload acceptance

Pin the installed native Pi 1.3.0 executable, then run the offline journey:

```sh
export REV_PI_EXECUTABLE=/absolute/path/to/installed/pi
bash scripts/check-ai-pi.sh
```

The script refuses an updater or shell wrapper and never installs Pi. It needs
Linux, cached locked Cargo dependencies, Bash, util-linux `script`, `stty` and
GNU `timeout`. JavaScript resources run inside Pi's installed runtime; no
separate Python, Node or npm build step is required. The script builds the
normal release editor and Rust driver with two jobs.

Owned HOME/XDG/Pi roots isolate both startup profiles. A validation-only local
provider emits controlled model events without HTTP, while the actual Pi
runtime performs RPC, read/list/search tools and history reconstruction. Checks
compare actual tool bytes/results, unsaved snapshot precedence, rejected private,
traversing, escaping, missing, binary and oversized reads, compatible-child reuse,
restart/reset, private limits, idle expiry and owned exit. Native publication
checks one undo, unchanged disk bytes and stale-result refusal after a real
atomic notification reload. Missing Pi leaves ordinary edit/undo usable.

Real PTYs preserve another command's input across a kernel-triggered reload,
reject the stale AI proposal and restore terminal attributes on owned exit.
Each retains the existing eight-second/1-MiB bound. Native journeys have a
35-second outer guard. Output records startup/reload timing, editor RSS,
external Pi RSS and idle CPU separately. Voluntary context switches are labeled
as such; they are not a claim to measure every kernel wakeup. Cleanup removes
only completed owned fixtures. These local observations establish no machine
independent latency or RSS guarantee.

A real-provider observation is deliberately separate from the repeatable gate.
The Rust driver's explicit `--live` mode requires `REV_PI_LIVE=1`, an unused toy
root and existing working Pi authentication/default model. It submits one fresh
input-only toy explanation, disables context tools/history, caps model turns at
four and tool calls at eight, and uses a 45-second request deadline with bounded
cleanup within a minute. It reports model, counts and timing without printing
authentication or the response. Do not put that mode in automatic gate retries.
Record the actual authentication form separately; offline fixtures do not prove
OAuth or API-key provider availability.

## Run heading presentation acceptance

```sh
env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-org-headings.sh
```

This journey needs Linux, the repository toolchain, cached locked Cargo
dependencies, Bash, `rg`, util-linux `script`, `stty`, `kill` and GNU
`timeout`. It builds the normal release editor and Rust driver with one job
and checks the heading policy, projection, view and command owners, crate roots
and heading/shared acceptance sources.
It uses isolated HOME/XDG roots without a language server, provider or font
installation.

Native cases and four real PTYs cover both editing profiles at 40x12 and
80x24, then resize to the other dimensions. They check emitted level-1-to-8
markers, explicit source/pretty/reset, canonical star copy, unchanged source
point, command cancellation, unchanged disk bytes and terminal restoration.
Native cases also preserve BOM/CRLF bytes and empty history. A finite large
Org fixture checks a heading-level edit and one undo, and measures startup,
repeated projection, refresh, peak RSS and idle CPU;
idle work has no pending deadline. Each PTY retains eight seconds and 1 MiB
of output; native/resource cases have 20-second outer guards and the four-PTY
group has 35 seconds. Only owned completed fixtures are removed.

These measurements describe the local run. Emitted Unicode characters establish
terminal protocol behavior; glyph appearance and ligatures belong to the
hosting terminal.

## Run shell command acceptance

```sh
env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-shell-commands.sh
```

This journey needs Linux, the locked repository toolchain and cached dependencies,
Bash, installed sh/sort/tr/cat/sleep, `rg`, util-linux `script`, `stty`, `kill`
and GNU `timeout`. It builds the release editor, driver and source checker with
one job, after requiring 1 GiB free on the private target filesystem. HOME/XDG
stores and sources belong to one temporary fixture. No provider or language
server is used.

Native journeys exercise both profiles with real commands, unsaved Unicode/CRLF,
checked filters/insertion and ordinary undo, readonly and stale targets, an
atomic external reload while a child is still active, and owned cancellation.
Source quality covers shell configuration/prompt/publication, the shared process
helper, activity status, crate roots and acceptance modules. Its safe-Rust,
five-parameter, three-level nesting and complexity limits remain unchanged.

The finite resource case supplies exactly the default 8 MiB input bound to real
tr, verifies the complete retained stdout and native inserted/removed history
bytes, then undoes once. Output records snapshot and collection/publication time,
undo observation, peak RSS, thread counts and 350 ms idle CPU. It requires no
post-job deadline, no remaining shell worker and at most one idle CPU tick.
These are local observations, not hardware-independent latency or RSS promises.

Two real PTYs cover non-modal sequences and modal Visual/colon invocation,
canonical Unicode selection, sort/tr output, one-step undo, inner prompt
cancellation before source job cancellation, descendant reaping and resize from
100x24 to 72x18. Each retains the existing eight-second/1-MiB limit and verifies
unchanged disk bytes, restored termios, cursor visibility, alternate-screen exit
and bracketed-paste shutdown. The driver checks post-job terminal silence and CPU.
Native, resource and two-PTY groups have 20-second outer guards. Only completed
owned fixtures are removed.

For focused development, the same script accepts `--native`, `--resources` or
`--pty`; the ordinary command above runs all three and is the canonical gate.

## Run integrated terminal acceptance

```sh
env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-integrated-terminal.sh
```

This journey needs Linux, cached locked Cargo dependencies, Bash, Git, installed
lazygit, `rg`, util-linux `script`, `stty` and GNU `timeout`. It builds the normal
release editor and Rust driver with one job and requires 1 GiB free. Disposable
Git/HOME/XDG stores remove inherited Git selectors; lazygit fetch/update are
disabled. No network or user Git state is needed.

Both native profiles run actual Bash and lazygit. Four host PTYs cover tool
relocation and multiple-session input/copy/closure, plus one real-PTY injected
host write fault. They assert process/state retention, exact literal/mouse/paste
bytes, protected source, fresh active-shell readiness, actual exit status and
full terminal restoration. Each case retains eight seconds and 1 MiB of output;
the five-case group has a 45-second outer guard. Native/resource groups each
have 20-second guards. Live commands have no runtime cap outside these fixtures.

The resource case runs a real 1 MiB flood beside a responding peer and a blocked
input owner. It records bounded service/retention/queues, CPU ticks at the printed
clock rate, RSS, exact FD/thread return and owned cleanup. The normal-release
binary delta uses Slice 1's explicitly recorded pre-terminal baseline, not a
fresh baseline rebuild. These local observations do not establish portable
latency, CPU or memory guarantees. The write-fault case proves restoration with
subsequent writes available, not recovery from a physically unavailable terminal.

Source checking covers the terminal engine/session/input/copy/shutdown owners,
crate roots and shared/new acceptance modules. Only owned completed fixtures
are removed. Use `--native`, `--pty` or `--resources` for focused development.
The ordinary command runs all three; the combined delivery gate remains a
separate controller operation.
