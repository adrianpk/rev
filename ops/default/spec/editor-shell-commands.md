# Shell Commands and Text Filters

Status: Completed
Date: 2026-11-03
Completed: 2026-11-03
Validated candidate: `736d631947d4d379cd65dd695ff2ab00fd6c964b`
Gate validation: `5SCPQH64UEI7UZ2VFXVFZMGKWH`
Reviewed: 2026-11-03
Version: 0.1
Format: OpenSpec capability proposal
Source: owner request to separate interactive terminals from piping and begin the agent-selected first capability
Implementation authority: owner, 2026-11-03; selected first capability under the delegated choice and established Emacs interaction guide
Model: [Shell Commands Model](editor-shell-commands-model.md)
Delivery: [Plan](../plan/rev-shell-commands.md), [tracker](../tracker/rev-shell-commands.md#final-gate-and-closure)
Baseline: [rev Editor](rev-editor.md), [Configuration](editor-configuration.md)
Related: [Interactive Terminals](editor-integrated-terminal.md), [External Extensions](editor-external-extensions.md)
Delivery boundary: native one-shot commands and text filters first; interactive terminals and optional extension protocols remain separate

## Purpose

Run an explicitly entered shell command, send a captured region to its standard
input when requested, and display or apply its complete output. Users do not
register a program in TOML before using it. Ordinary shell pipelines and quoting
belong to the selected shell; selected document text is input data, never an
automatically constructed command.

This is a native editing capability with fixed commands. It does not adopt the
optional extension draft's configured program registry, JSON protocol or command
discovery. Interactive sessions, lower panels and terminal-backed buffers belong
to the independent terminal specification.

## Selected profile and baseline amendment

The owner authorized two specifications and starting the more relevant capability.
This profile selects one-shot shell commands first because they provide direct
text transformation without a terminal emulator or another dependency.

The baseline target is the rev-editor.md paragraph beginning "The baseline
excludes a GUI" and its following Org-adapter boundary, which exclude embedded
shells and a general build/test runner. This additive
delta admits explicit user-entered, one-shot shell subprocesses and their text
results. It does not admit an embedded interactive shell, startup execution,
automatic task orchestration or a public extension interface. Historical editor
delivery records retain their original scope.

The interaction reference is GNU Emacs Single Shell Commands, verified against
the installed GNU manual on 2026-11-03. M-! uses null input; M-| sends the region;
a prefix selects insertion or replacement. rev exposes each outcome as a named
command rather than introducing a general universal-argument engine. Execution
is nonblocking so native editing, cancellation and file observation continue.
A completed result can only modify its originally captured target.

## Native commands

| Name | Default non-modal key | Input | Output |
| --- | --- | --- | --- |
| shell-command | M-! | No document text; immediate stdin EOF | Show stdout |
| shell-command-on-region | M-\| | Captured linear region | Show stdout |
| shell-command-insert | C-u M-! | No document text; immediate stdin EOF | Insert at captured point |
| shell-filter-region | C-u M-\| | Captured linear region | Replace that region |
| shell-command-cancel | None; contextual C-g also cancels | Current owned invocation | Cancel and clean up |
| shell-command-output | None | Last retained result identity | Visit its output or diagnostic buffer |

All six are native shared commands through M-x and modal colon, and may receive
user bindings. Modal Normal/Visual preserve their motions and C-u behavior;
the non-modal C-u chords are specific existing keymap sequences, not general
numeric arguments. Preserve Org's other local C-u routes. A modal Visual linear
selection remains the region captured before colon entry. A rectangle/block
selection is rejected explicitly rather than flattened silently.

A region command requires a nonempty admitted linear selection. It never falls
back to the complete document. Selecting the whole document with ordinary
mark-whole-buffer permits whole-buffer filtering.

## ADDED Requirements

### Requirement: SHL01 Explicit invocation and command input

An accepted native command SHALL capture its originating buffer, view, point or
region and visiting-file context before opening the command prompt. Return SHALL
launch only the command text the user accepts. Empty, whitespace-only, NUL-bearing
or over-4096-byte UTF-8 command strings SHALL reject without starting a process.
Prompt cancellation SHALL leave source and history unchanged.

The prompt SHALL use ordinary native editing and cancellation and retain at most
32 accepted command strings in session-local history. Loading configuration,
listing commands and opening files SHALL never execute a shell command.

#### Scenario: Run a simple command

- **GIVEN** an editing view
- **WHEN** the user invokes shell-command and accepts printf 'hello\n'
- **THEN** one invocation runs with no document text on stdin
- **AND** its complete stdout is available through native output presentation.

#### Scenario: Cancel before launch

- **GIVEN** an open shell-command prompt
- **WHEN** the user presses C-g or contextual modal Escape
- **THEN** no child is started and source remains unchanged.

### Requirement: SHL02 Shell and directory resolution

The subprocess SHALL use shell.program when nonempty, otherwise the nonempty
SHELL environment value captured at editor startup, otherwise /bin/sh. Bare names
use ordinary executable resolution, without directory scans. The accepted command
string SHALL be appended as one final argument after shell.args. The default is
-c; do not add interactive/login flags or inject selected text into that string.

The child SHALL start in the captured visiting file's containing directory.
For unnamed buffers use the editor's explicitly opened directory, then the launch
directory. Reuse current native path identities; no workspace abstraction,
implicit cd, directory registration or automatic save is required.
A later focus or tree-root change SHALL not redirect that invocation.
The child inherits the ordinary editor process environment; rev does not perform
project-local configuration discovery.

#### Scenario: Execute beside the visiting file

- **GIVEN** a file in a nested directory and unsaved source edits
- **WHEN** shell-command executes pwd
- **THEN** the child uses that file's containing directory
- **AND** rev does not save the buffer or change its current directory globally.

### Requirement: SHL03 Exact bounded input and complete output

Region input SHALL be an immutable snapshot of the selected canonical text,
including its actual newline content, supplied to stdin followed by EOF.
No-input commands SHALL receive immediate EOF. Input delivery and concurrent
stdout/stderr draining SHALL avoid a write/read pipe deadlock.

Output collection SHALL retain stdout and stderr separately under one aggregate
shell.max_output_bytes bound. A result SHALL require process exit and complete
stream observation; truncation SHALL never become a successful edit.
Text admitted for native presentation or insertion SHALL satisfy native UTF-8
and document-text admission. No trimming, markdown extraction, ANSI evaluation
or automatic formatting may change stdout used as edit content.

#### Scenario: Filter unsaved text

- **GIVEN** the active linear selection contains three unsaved lines
- **WHEN** shell-filter-region accepts sort
- **THEN** sort receives those selected lines rather than the visiting file
- **AND** the successful output replaces only the captured region.

#### Scenario: Drain simultaneous streams

- **GIVEN** a command consumes input while writing stdout and stderr
- **WHEN** all admitted streams make progress
- **THEN** rev drains both output streams without waiting for stdin completion
- **AND** aggregate excess output fails and cleans up the invocation.

### Requirement: SHL04 Checked atomic edit publication

Insertion and replacement SHALL require successful exit, complete valid stdout
and an unchanged original buffer identity, revision, target range and writable
state. Validate the captured target before launch and again before publication.
A changed buffer, external reload, closed target or later read-only transition
SHALL prevent automatic application. Switching panes or buffers SHALL never
redirect output to the newly selected source.
Pending external-file observations and conflicts for that source SHALL be
settled or rejected under the existing reload policy before edit publication;
completion must not bypass a queued replacement or consent decision.

A successful edit SHALL use the ordinary native transaction and one undo step,
including all existing source, newline and view-position conventions.
Empty successful replacement output SHALL delete the region; empty insertion
SHALL be a no-op. The selected input snapshot is not a save operation.

#### Scenario: Reject a stale filter

- **GIVEN** a filter is running for one captured region
- **WHEN** the source is edited or externally reloaded before completion
- **THEN** its output is retained for inspection without modifying source
- **AND** rev explains that the original target changed.

#### Scenario: A command writes the visiting file

- **GIVEN** a command changes its source file on disk while collecting stdout
- **WHEN** native file observation and command completion become ready
- **THEN** the existing external-change policy retains authority over that file
- **AND** stdout does not silently overwrite a conflicting or reloaded target.

#### Scenario: Undo a replacement

- **GIVEN** a successful region replacement
- **WHEN** the user invokes ordinary undo
- **THEN** the exact selected source is restored in one step.

### Requirement: SHL05 Results and diagnostics

Complete stdout SHALL remain available as native plain-text read-only output.
Short single-line output may also use the echo area; multi-line output SHALL
have an inspectable buffer. Never splice status text, command metadata or stderr
into edit payloads. Native rendering, navigation, search and copy SHALL treat
output as text rather than terminal control authority.

Nonzero exit SHALL retain bounded diagnostics and available stdout for inspection,
report the exit status and leave source unchanged. Invalid text, timeout,
cancellation and output overflow SHALL not expose a partial result as applicable
source. Older output buffers SHALL not be overwritten if their checked native
result ownership has changed. No result archive or implicit disk persistence is
introduced.

Completion SHALL not steal focus from another prompt, newer user interaction or
different selected source. shell-command-output SHALL provide an explicit way
to visit the last retained result; a closed result reports unavailable rather
than resurrecting a stale target.

#### Scenario: Inspect failure without editing

- **GIVEN** a replacement command writes stdout and stderr and exits nonzero
- **WHEN** rev observes completion
- **THEN** source remains unchanged and bounded diagnostics are inspectable
- **AND** stdout is not mistaken for an accepted transformation.

### Requirement: SHL06 One owned cancellable invocation

The first profile SHALL admit one running shell invocation per editor instance.
Another attempt SHALL report the active job without queuing or replacing it.
Input, rendering, source editing and external-file observation SHALL remain
responsive while the command runs.

A normal C-g in the originating editing context, or shell-command-cancel through
either command selector, SHALL cancel the invocation. Existing inner prompt,
search, popup and modal Escape ownership wins before job cancellation.
Quitting SHALL cancel and reap the owned process group only after the ordinary
save/quit decisions succeed; cancelling quit SHALL retain the running job.

Cancellation, timeout, output failure, leader exit and editor shutdown SHALL
close streams, stop owned descendants and reap the leader under the existing
bounded Unix process-group cleanup discipline. Never signal an unowned process
or reuse stale PID authority. Waiting for a job SHALL not introduce a persistent
worker or periodic input/render wake after that job ends.

#### Scenario: Cancel a waiting command

- **GIVEN** a shell child is waiting without producing output
- **WHEN** shell-command-cancel is invoked
- **THEN** it and owned descendants are stopped and reaped
- **AND** source remains unchanged and the editor resumes idle waiting.

### Requirement: SHL07 Complete startup preferences

All settings SHALL participate in built-in defaults, complete sample TOML,
typed admission, include precedence/provenance and whole-candidate fallback.
They are startup-only, global, and do not silently rewrite the user's file.

| Setting | Default | Admission |
| --- | --- | --- |
| shell.program | "" | Empty selects SHELL then /bin/sh; otherwise one executable string, <=4096 UTF-8 bytes, no NUL. |
| shell.args | ["-c"] | Whole-array replacement; 1..32 strings, <=4096 bytes each, no NUL; <=32768 bytes total. |
| shell.timeout_seconds | 600 | Integer 1..600. Explicit finite one-shot profile; not an interactive session. |
| shell.max_input_bytes | 8388608 | Integer 1..16777216, checked before snapshot/staging. |
| shell.max_output_bytes | 8388608 | Integer 1..16777216, aggregate stdout plus stderr. |

The 4096-byte command limit and single-active-invocation profile are explicit
native bounds. A shell pipeline remains within the same invocation and process
group. A detached/background child is not a retained session; this profile
cleans up its owned group when the invocation completes. M-& and a streaming
background-jobs interface are not claimed by these four editing operations.

#### Scenario: Override a limit

- **GIVEN** valid user configuration changes only shell.max_input_bytes
- **WHEN** rev starts
- **THEN** that value overrides its built-in leaf and other shell defaults remain
- **AND** invalid fields reject the candidate through the ordinary loader.

### Requirement: SHL08 Integration and representative acceptance

The delivery SHALL verify real local shell commands and filters in both editing
profiles, through native sessions and Linux PTYs, including resize, Unicode,
literal newlines, readonly/stale targets, cancellation, noisy streams, source
immutability on failure and one-step undo. Exercise existing Org prefixes,
modal C-u, path context, concurrent external reload and owned child cleanup.

Measure a finite large-region fixture for snapshot/application cost, retained
bytes and idle CPU after completion. Use real system sh, printf, sort and tr;
no model provider, language server, interpreter package or network is required.
Any low-level process-helper reuse SHALL preserve existing Org consent,
argv, resource and lifecycle contracts. Add no crate by default.

#### Scenario: Accept the complete native interaction

- **GIVEN** isolated native and Linux PTY fixtures with sh, sort and tr
- **WHEN** command, region, insertion and replacement journeys run in both profiles
- **THEN** source/undo, cancellation, resize and owned cleanup satisfy SHL01-SHL07
- **AND** a completed job leaves no worker or periodic idle input deadline.

## Completion evidence

SHL01-SHL08 are delivered through canonical PRs #242-#245 and repair #246.
The unchanged controller gate passed on the immutable candidate above, covering
native commands, checked text publication, both profiles, owned process cleanup,
finite resources, real Linux PTYs and guide projection. The
[Slice 4 closure](../report/slices/rev-shell-commands/slice-4-acceptance.md#delivery-set-closure)
records exact evidence. The baseline's existing additive shell paragraph is
reconciled with this completed capability; earlier gates retain their identities.

## References

- [GNU Emacs shell command overview](https://www.gnu.org/software/emacs/manual/html_node/emacs/Shell.html)
- [Single Shell Commands](https://www.gnu.org/software/emacs/manual/html_node/emacs/Single-Shell.html)
- [GNU Emacs reference card](https://www.gnu.org/software/emacs/refcards/pdf/refcard.pdf)
