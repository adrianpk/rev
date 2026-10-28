# Editor Gutter

Status: Completed
Completed: 2026-10-28
Validated candidate: `dedd85064cb5565a5382a9d8e2e2902daae24d88`
Gate validation: `PZ4OIPXA6GUBSWDH5ZS7MJUOGX`
Approved: product owner, 2026-10-27; specification and model accepted.
Date: 2026-10-27
Format: OpenSpec capability additions
Baseline: [rev Editor](rev-editor.md), R05-R07, R09, R14-R15, R18-R20 and R35
Related contract: [Editor Interaction](editor-interaction.md), UI01
Configuration: [Editor Configuration](editor-configuration.md)
Rev baseline revision: `756e540b0830866fb0dfd59c045dff5958c11095`
Delivery: [Input and Navigation](../plan/rev-input-navigation.md); integrated gate passed
Scope clarified: product owner, 2026-10-27; native numbering now, bookmarks and LSP completion later
Controls clarified: user settings, compiled M-x commands, or both according to the concern
Model: [Editor Gutter Model](editor-gutter-model.md)

## Purpose

Provide absolute and relative logical line numbers in rev's editing views,
implemented entirely in Rust. Preserve Odra's numbering, geometry, Unicode,
per-view ownership and lifecycle guarantees within rev's smaller product scope.

Rev is a fast, CPU- and memory-efficient text editor with useful factory behavior.
The common editing path must be comfortable without additional configuration.
Every user-facing concern SHALL have an appropriate control through TOML,
runtime M-x commands, or both. Factory completeness means no required setup,
not absence of controls. M-x commands are compiled Rust handlers with the same
user-facing discovery and invocation model as a programmable editor. The
already approved configuration/UI delivery retains its own scope; this proposal
does not cancel or extend it.

## Selected scope

The current gutter capability comprises:

- absolute logical line numbers;
- relative logical line numbers using each view's point;
- Odra's independent current-line form: absolute, zero or hidden, defaulting
  to the real one-based number with distinct semantic styling;
- stable number width, correct cell accounting and protected text geometry;
- built-in Rust commands to select a mode and hide/restore numbers; and
- complete behavior with factory defaults and no required user setup; and
- documented user settings and runtime controls according to the companion
  model, using the existing TOML loader and static Rust command registry.

Related future work is captured separately in the draft specifications for
[native bookmarks](editor-bookmarks.md),
[native autocompletion](editor-autocompletion.md),
[optional LSP support](editor-language-server-support.md) and
[language-server installation](editor-language-server-installation.md).
These topics do not extend the accepted numbering implementation.

The earlier programmable-provider, arbitrary-lane, marker-inspection and popup
proposals are omitted. Search/mark annotations, clipping glyphs, VCS, debugging
and tooling markers are not part of the selected numbering implementation.
Do not add an extension registry, provider callbacks, bridge, speculative LSP
adapter or annotation framework in anticipation of future work.

The original proposal and adaptation record below retain their provenance.
The approved input/navigation plan and tracker now own implementation and
controller authority; this specification alone grants no publication authority.

## Reference and adaptation

Reference: Odra, `ops/default/spec/stable/gutter-and-line-annotations.md`,
revision `f701c830b88e2088bb8de93ba9fee2dd0cd111f1`, file SHA256
`33656dbfb012d807895f4fd4e8b25a8923addcbdfc3b3a11727fe99f8f7be0fe`.
Read together with Odra's `docs/reference/gutter/index.md`, its User Guide
chapter `interface/gutter-and-transient-details.md`, and its factory numbering
and geometry contracts. This reference identifies behavior, not copied code.

| Odra behavior | Rev treatment |
| --- | --- |
| Gutter belongs to the text view | Retain inside the existing panel content bounds. |
| Independent policies for views sharing text | Retain independent point, relative origin, visibility and mode. |
| Absolute, relative, hybrid and hidden presentation | Two calculation modes: absolute and relative. Current-line form is independently selectable; visibility is separate. |
| Logical line scope | Retain, including empty lines, CRLF and blank viewport padding. |
| Four minimum digits, right alignment, stable width | Retain; width uses buffer line count rather than cursor distance. |
| Unicode and terminal-cell accounting | Retain one checked geometry/display contract and ASCII separator fallback. |
| Narrow-panel text protection | Retain; hide numbers before consuming the protected text area. |
| Programmable providers and generic lanes | Omit; the current implementation only needs native numbering. |
| Semantic annotation families | Future bookmarks have explicit product direction; other families are not implied. |
| Theme/frontend extension surface | Reuse rev's palette and terminal renderer. |

Odra's relative and hybrid factory calculations use logical distance for
non-current rows and a separate current-row policy. Rev retains that independent
current-row policy and its absolute factory default with two user-facing
calculation modes.

## Factory behavior and controls

Numbers are initially visible on the left in absolute mode. The number field
is right-aligned, reserves at least four digits, has zero left padding and one
right padding cell, and uses `│` with `|` fallback. The current line defaults to
its real absolute number. Both modes are usable without a configuration file.

Every exposed concern has one control owner. Mode, visibility, side and current-
line form have both startup TOML defaults and active-view M-x operations. Digit
width, padding and palette roles are startup settings. The companion model
specifies the complete control matrix, default values and command identities.
A fixed internal safety limit is not automatically a user setting.

The user can search and invoke native commands through M-x and bind their names
through the existing validated keymaps. Every command resolves to a concrete
Rust handler in the binary. No dynamic command layer or scripting API is needed
to provide that interaction. This specification assigns no factory keybinding that
would replace an existing editor shortcut.

## ADDED Requirements

### Requirement: GUT01 View-owned presentation

The gutter SHALL be derived presentation within an editing view's panel content
bounds. The buffer SHALL remain the canonical text/history owner. Panel borders,
status rows and prompt/candidate surfaces SHALL retain their existing owners.

Numbers and separators SHALL NOT enter copied/killed/rectangular
text, search, replacement, save output, syntax offsets, undo data or reported
logical text columns. Changing gutter presentation SHALL NOT change rope bytes,
text revision, dirty state or saved-state identity.

#### Scenario: Copy and save with gutters enabled

- **GIVEN** a buffer displayed with numbers in two editing panes
- **WHEN** the user copies a region or rectangle and saves ordinary text edits
- **THEN** only canonical document text is copied and written
- **AND** gutter changes create no edit transaction or dirty-state change.

### Requirement: GUT02 Absolute and relative numbering

Absolute mode SHALL display each logical line's one-based index. Relative mode
SHALL display the nonnegative logical-line distance from the point in that
specific view. Inactive views SHALL use their own point, never the focused
pane's point. Relative values SHALL NOT count terminal cells or visible rows.

The current-line form SHALL be independently selectable as `absolute`, `zero`
or `hidden`, with distinct semantic styling and absolute as the factory default,
following Odra. Visibility SHALL be separate from the selected calculation;
hiding and restoring numbers SHALL retain the mode.

#### Scenario: Relative numbers preserve the real current line

- **GIVEN** relative mode and point on line 42
- **WHEN** the viewport shows lines 40 through 44
- **THEN** its number labels are 2, 1, 42, 1 and 2
- **AND** choosing zero or hidden changes only the current row's label.

#### Scenario: Shared views have independent relative origins

- **GIVEN** two views share a buffer with their points on lines 20 and 60
- **WHEN** both use relative mode and focus switches between them
- **THEN** each view's numbers remain relative to its own point
- **AND** changing one view's mode does not move either point.

### Requirement: GUT03 Logical lines and real document rows

Numbering SHALL use the existing buffer's logical line model. An empty buffer
SHALL expose its first editable logical line as line 1. CRLF SHALL be one line
separator; tabs, combining marks and wide graphemes SHALL NOT create line
numbers. A trailing editable empty logical line follows that same buffer model.
Blank viewport padding beyond the last logical line SHALL have no line number
.

The delivered no-wrap presentation SHALL remain authoritative. A future wrapped
continuation cannot acquire another logical line number merely because it
occupies another display row. Unsupported visual numbering SHALL be rejected
explicitly if requested, without reinterpreting it as logical numbering.

#### Scenario: Empty document and padding rows

- **GIVEN** an empty editable buffer in a taller viewport
- **WHEN** absolute numbering is displayed
- **THEN** its real first logical line shows 1
- **AND** rows used only to pad the viewport remain unnumbered.

### Requirement: GUT04 Stable number geometry

The number field SHALL be right-aligned and reserve at least four digits by
default. Its digit width SHALL derive from the complete buffer's logical line
count using checked integer metadata, with an admitted upper bound sufficient
for every representable buffer line index. Cursor movement in relative mode
SHALL NOT shrink or grow the field. Width changes at digit boundaries or a
numbering-policy change SHALL trigger layout before rendering.

Padding and separators SHALL have explicit reserved cell widths. The default
separator SHALL be `│`, with `|` fallback. Width SHALL be measured in terminal
cells under the same display policy used by the renderer, never byte length or
Unicode scalar count. Admitted bounds SHALL NOT permit silently truncated or misleading line labels.

#### Scenario: Cross a decimal boundary

- **GIVEN** a four-digit field and a buffer with 9999 logical lines
- **WHEN** an edit creates logical line 10000
- **THEN** the field grows coherently to five digits before the next frame
- **AND** undo restores the matching width and valid view positions.

#### Scenario: Move within relative mode

- **GIVEN** a large buffer with relative numbering
- **WHEN** point moves through distances with different digit counts
- **THEN** the gutter width remains stable
- **AND** work remains bounded by the visible projection.

### Requirement: GUT05 One coordinate and clipping contract

Panel layout SHALL reserve gutter cells before determining the final text
viewport. Cursor placement, horizontal reveal, selection, rectangles, search
and syntax style ranges SHALL use that same final text origin and extent.
A literal tab's logical stops SHALL NOT shift merely because a gutter becomes
visible or changes sides. Gutter cells SHALL NOT scroll horizontally with text.

The number field SHALL have explicit reserved bounds and order within its
editing panel. It SHALL NOT overwrite borders, status, the prompt or another
pane. Layout SHALL resolve before the text and number rows are drawn.

#### Scenario: Tabbed text in a nested panel

- **GIVEN** nested splits, a visible tree and text containing tabs and wide clusters
- **WHEN** number visibility changes and point is revealed horizontally
- **THEN** all display overlays and the cursor use the new text bounds
- **AND** logical columns, tab stops and rectangle extraction remain unchanged.

### Requirement: GUT06 Protect text under limited geometry

The gutter SHALL yield to the panel's admitted minimum text area. Numbers SHALL
hide as a complete field
rather than consume protected text columns or render a false truncated label.
A temporarily hidden lane SHALL retain its requested policy and restore it
when geometry permits.

Zero/tiny dimensions SHALL preserve view state through the baseline suspended
presentation path. File tree, prompts and completion panels SHALL NOT receive
source-editor line numbers merely because they use the panel abstraction.

#### Scenario: Shrink and restore shared panes

- **GIVEN** two independently configured gutters and an active completion prompt
- **WHEN** the terminal shrinks to unusable dimensions and grows again
- **THEN** gutter lanes recover their prior requested policy within valid bounds
- **AND** document text, independent points, prompt draft and focus remain intact.

### Requirement: GUT10 Safe number rendering and semantic palette

Numbers, current-line emphasis, background and separator SHALL use semantic
roles in rev's existing Tokyo Night palette. Factory values SHALL be readable
without installing a theme or editing a configuration file. Ordinary and
current-line labels SHALL remain meaningful under reduced color capability.

Separators and labels SHALL use the same terminal-safe display boundary and
cell measurement as document rendering. Wide or combining representations
SHALL be measured consistently or rejected before layout; control/escape text
SHALL NOT become terminal markup. An unavailable separator glyph SHALL select
the printable ASCII fallback without shifting logical document coordinates.

#### Scenario: Render with the separator fallback

- **GIVEN** the primary separator cannot be displayed reliably
- **WHEN** rev composes the gutter using the ASCII fallback
- **THEN** number and text bounds remain coherent and terminal-safe
- **AND** ordinary text, syntax, selection and search remain correctly aligned.

### Requirement: GUT11 Per-view lifecycle

A new view SHALL use the startup default. A split SHALL inherit its source
view's effective gutter policy and may then diverge. Switching the buffer in
one view SHALL retain that view's presentation policy while recomputing numbers for the new buffer. Closing a view SHALL release
its gutter state;
closing a buffer SHALL leave no gutter reference to that buffer.

Session overrides SHALL NOT be persisted implicitly or copied into document
metadata. Shared views SHALL retain one canonical text owner while keeping
separate
projection, point, visibility and numbering state.
Pointer input remains optional; if it exists, a gutter hit SHALL NOT be routed
as ordinary text selection or dragging merely because it is adjacent to text.

#### Scenario: Rebind and close a split view

- **GIVEN** a split with its own numbering policy showing buffer A
- **WHEN** it switches to buffer B and is later closed
- **THEN** its policy survives the switch but its labels now describe B
- **AND** closing it releases only its own derived state and leaves the other view intact.

### Requirement: GUT12 Configuration, native commands and complete defaults

The selected numbering capability SHALL work with no user configuration. Every
public control SHALL be available through startup TOML, runtime M-x or both as
specified by the model. Optional settings SHALL extend the existing typed
configuration catalog and complete reference sample only in the later gutter
delivery. Unknown-key rejection, per-leaf layering and whole-candidate validation
SHALL remain unchanged. There SHALL be no separate settings store.

M-x SHALL discover and invoke concrete compiled Rust command handlers through
the same registry used by configured keybindings. Commands SHALL affect the
active editing view, preserve normal context/cancellation behavior and report
unavailable contexts clearly. Hiding/restoring numbers SHALL retain mode;
explicit mode selection SHALL show that mode. Mode, side and current-line-form
changes SHALL create view-local overrides; reset SHALL restore the startup
snapshot defaults for that view. None of these operations SHALL reload TOML,
modify the startup snapshot, edit text or persist the override implicitly.

#### Scenario: Select relative mode without setup

- **GIVEN** two ordinary editing views and no user configuration file
- **WHEN** the user finds and invokes the native relative-number command through M-x
- **THEN** only that view changes and both views remain usable
- **AND** no setup, plugin, external process or configuration write is required.

#### Scenario: Runtime override and reset

- **GIVEN** startup TOML selects absolute mode and a native command selects relative in one view
- **WHEN** the user resets that view's gutter controls
- **THEN** it returns to the resolved startup settings
- **AND** other views, text, undo and the configuration file remain unchanged.

### Requirement: GUT13 Bounded CPU, memory and redraw work

Numbering SHALL use existing buffer line-count/index metadata and visible-row
projection. Cursor movement SHALL NOT enumerate every buffer line or allocate
a retained number object per possible line. Width calculation SHALL use checked
integer arithmetic and support the largest admitted line index without a false
truncated label.

Hidden numbers SHALL do no number-row materialization. Numbering SHALL add no
idle repaint loop, background scanner, timer or external process. Edits, view
movement, mode changes and resize SHALL invalidate only the required geometry
and visible presentation through the existing redraw path.

#### Scenario: Move point through a large document

- **GIVEN** a large buffer, bounded viewport and relative numbering
- **WHEN** point moves rapidly and the viewport scrolls
- **THEN** numbering performs bounded visible-range work
- **AND** hiding numbers removes their row-materialization work without adding idle activity.

### Requirement: GUT14 Acceptance through real editor paths

Future implementation SHALL verify both numbering modes, per-view state,
geometry and terminal presentation at their real boundaries. A terminal journey
SHALL exercise the native commands with factory defaults and no config file, and verify supported startup settings.
Existing text, history, safe-save and terminal cleanup guarantees remain
required. CPU/memory measurements SHALL use representative large-buffer,
cursor/scroll and idle workloads; no runtime result is claimed by this specification.

#### Scenario: Numbered editing preserves normal behavior

- **GIVEN** no user configuration, a tabbed Unicode file and nested shared views
- **WHEN** the user changes numbering, copies a rectangle, searches, scrolls,
  resizes, saves, switches buffers and closes the session
- **THEN** labels and text overlays agree with each view's current projection
- **AND** canonical text, file fidelity, undo truth and terminal restoration remain correct.

## Related future specifications

- [Native Bookmarks](editor-bookmarks.md): real bookmark ownership and later
  marker integration; location, navigation and persistence choices remain open.
- [Native Autocompletion](editor-autocompletion.md): compiled Rust interaction,
  preferred operational LSP and Tree-sitter-only fallback; no ctags.
- [Language Server Support](editor-language-server-support.md): optional client,
  document synchronization, completion integration, controls and resource use.
- [Language Server Installation](editor-language-server-installation.md): simple
  optional installation/discovery, Rust priority, secondary languages including
  Odin and easy additional server connections.

These drafts preserve the discussed direction and remaining questions. None
adds runtime work or dependencies to this accepted gutter specification.

## Validation obligations

- Both modes across empty, one-line, multiline, CRLF and Unicode buffers;
  real final empty lines and blank viewport padding beyond EOF.
- Number width across 9/10, 99/100 and 9999/10000 boundaries, edits, undo and
  redo; stable width while moving the relative origin.
- Independent points and modes, inherited splits, buffer switching, close and
  teardown without stale or foreign view/buffer references.
- Left/right placement, number and separator cell accounting, horizontal reveal,
  Unicode tab widths,
  selection, rectangles, search and syntax alignment inside nested panels.
- Tiny/zero/growing geometry, file tree and completion panels; protect the
  admitted text area and restore hidden requested state.
- Safe ASCII fallback, current-line role and no escape/control leakage.
- Factory-default operation without a config file, all documented M-x commands, startup-setting inheritance and
  preservation of the existing validated keymap behavior.
- A real PTY editing journey, file fidelity, terminal cleanup, bounded large-
  buffer CPU/memory work and event-driven idle behavior.

These are future implementation obligations, not completed tests. Odra's
cell-accounting and sanitization audit findings inform boundary coverage;
they are not asserted as reproduced rev bugs.

## State and delivery boundary

The companion model adds preferences to the existing TOML schema. It creates no
independent settings store or persistent annotation entity. Runtime overrides
and geometry remain ordinary view/session state. Future bookmark or LSP
contracts will define their own actual state and persistence when needed.

The accepted specification and model are versioned. The product owner selected
this bounded capability for the combined input/navigation/presentation delivery.
Its plan and tracker own slicing, implementation and controller authority; the
completed configuration/UI validation remains historical evidence. Reconcile
current configuration/view contracts without expanding gutter behavior.
