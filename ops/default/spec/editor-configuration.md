# Editor Configuration

Status: accepted
Approved: product owner, 2026-10-27; unified configuration and interaction delivery authorized.
Date: 2026-10-27
Format: OpenSpec capability proposal with explicit baseline amendments
Baseline: [rev Editor](rev-editor.md)
Baseline revision: `d0b2c8dd26e1721f1413cbfc25816dc1b3441731`
Model: [Editor Configuration Model](editor-configuration-model.md)
Implementation boundary: rev-configuration-ui, after completed rev-editor delivery
Plan: [Configuration and Interaction Delivery](../plan/rev-configuration-ui.md)

## Purpose

Define declarative TOML configuration for `rev`, with built-in fallback
defaults, an exhaustive reference sample, palette role overrides, editing
preferences, language-specific settings, and context-specific keybindings.

The original 48-slice editor delivery is complete. This accepted change belongs
to the separately authorized configuration and interaction delivery. Its plan
and single tracker own implementation and completion; acceptance of this
contract does not claim that the runtime already supports configuration.

OpenSpec is the shared workspace Markdown format. Reading or validating this
proposal does not require the OpenSpec CLI or another runtime service.

## Decision state

### Confirmed direction

- User configuration is TOML under the XDG configuration directory, normally
  `~/.config/rev/config.toml`.
- XDG data, state, and cache directories have separate responsibilities.
- Built-in defaults remain available without any configuration file.
- A present valid value overrides its corresponding built-in default; an
  omitted value inherits. This applies to individual palette roles as well.
- The requested Tokyo Dark Night reference is Tokyo Night's dark/night variant.
  The model pins the VS Code and Neovim sources and exact RGB defaults. Where
  those ports differ, the resolved Neovim night palette supplies the defaults.
- The reference sample includes the complete palette explicitly, even when
  its values equal the built-in defaults.
- The sample documents the entire supported configuration surface, including
  supported optional settings as commented examples when unused.
- Configuration is organized by concern, with language-specific overrides.
  Optional file separation needs explicit load order and application-defined
  composition; it is not a native TOML import mechanism.
- Keybindings belong to configuration and refer to existing Rust commands.
- Existing bindings may be reassigned; explicit unbinding is not supported.
- Configuration loads at startup only. Reloading during a session, including
  applying edits automatically, is a potential future expansion.
- Indentation style, indentation width, and the display width of literal tabs
  are separate concepts. Four spaces generally and two for Go are the example
  used to establish inheritance, not a requirement to rewrite Go files.
- Tab advances to the next applicable stop. A literal tab remains U+0009;
  choosing spaces for indentation is a separate explicit editing policy.
  Converting existing tabs and spaces belongs to a separate future ticket.

### Selected scope

The selected delivery configures existing editor capabilities, together with
the agreed indentation and keybinding policies. The separately captured panel,
vertical command-completion, and file-path-completion tickets join its
unified delivery as explicit work items rather than implied options.

The model's broader option inventory records potential future expansions and
proposed values. It does not authorize adding their missing behavior to this
delivery. Line numbers, automatic indentation, visible-whitespace modes, cursor
blinking, and other absent capabilities remain future possibilities. They
shall not appear as usable active or commented settings in the shipped sample.

This proposal configures built-in behavior. Theme packages, script evaluation,
plugin/addon loading, session restoration, and automatic project-local
configuration are separate capabilities. Reserving a suitable directory for
possible future data does not implement or approve those capabilities.

## ADDED Requirements

### Requirement: CFG01 XDG locations and ownership

`rev` SHALL resolve the following locations through their XDG variables, using
the defaults shown when a variable is unset or empty. Relative variable values
SHALL be ignored as invalid. The configuration filename SHALL be `config.toml`.

| Responsibility | Variable | Default project directory |
| --- | --- | --- |
| User settings | `XDG_CONFIG_HOME` | `~/.config/rev/` |
| Durable user data and possible installed resources | `XDG_DATA_HOME` | `~/.local/share/rev/` |
| Restart-persistent operational state | `XDG_STATE_HOME` | `~/.local/state/rev/` |
| Regenerable nonessential data | `XDG_CACHE_HOME` | `~/.cache/rev/` |

Opening the editor SHALL NOT eagerly create empty directories or a user
configuration file. A supported feature creates its destination only when it
actually needs to write. User documents SHALL NOT be redirected into these
directories. The model defines the distinction between configuration, data,
state, and cache; no new storage feature is implied by these locations.

#### Scenario: Custom configuration home

- **GIVEN** an absolute `XDG_CONFIG_HOME` and no other location override
- **WHEN** `rev` locates user settings
- **THEN** it reads `rev/config.toml` relative to that directory
- **AND** data, state, and cache retain their independently resolved defaults.

#### Scenario: Missing user configuration

- **GIVEN** no user configuration file exists
- **WHEN** the editor starts
- **THEN** it uses built-in defaults
- **AND** it does not create a configuration file merely to start.

### Requirement: CFG02 Declarative typed TOML

Settings SHALL be UTF-8 TOML data, interpreted through a documented typed
schema. A file SHALL NOT execute Rust, Ruby, shell commands, substitutions, or
an embedded configuration language. Command bindings name the fixed Rust
command catalog; they do not define executable bodies.

The schema SHALL distinguish an absent field from an explicit false, zero,
empty string, or empty array where that value is valid. Unknown keys, invalid
types, invalid enum members, and out-of-range values SHALL have clear errors.
Schema version 1 SHALL be used. Omitting `version` SHALL imply 1; an explicit
unsupported version SHALL reject the candidate. TOML 1.1.0 SHALL be accepted
through the pinned parser selected by the plan.

#### Scenario: Explicit false survives inheritance

- **GIVEN** a built-in boolean default is true
- **WHEN** a valid override explicitly sets that field to false
- **THEN** the effective value is false
- **AND** false is not treated as an absent field.

### Requirement: CFG03 Built-in defaults and per-key overrides

The complete operational default configuration SHALL be compiled into the
editor. The editor SHALL NOT depend on a distributed sample file being present
to obtain its defaults. A valid configured leaf replaces the corresponding
default; omitted siblings retain their inherited values.

The loader SHALL validate partial configuration before resolving it against
defaults. It SHALL NOT first fill a partial language/include table with type
defaults and thereby overwrite unrelated inherited settings.

#### Scenario: Override one palette leaf

- **GIVEN** built-in values exist for every palette role
- **WHEN** the user defines only `palette.background`
- **THEN** that color replaces the built-in background
- **AND** every other palette role retains its inherited value.

### Requirement: CFG04 One configurable palette

The editor SHALL provide a fixed catalog of semantic palette roles covering
its actual UI and bundled syntax captures. Built-in role values SHALL use the
Tokyo Night dark/night defaults in the model's Palette reference and defaults
section. Users MAY replace individual role colors in TOML. This capability
SHALL NOT require a theme package, named theme selector, or theme installation
workflow.

Colors SHALL use six-digit RGB strings in `#RRGGBB` form. Malformed colors and
unknown role names SHALL reject the candidate configuration instead of silently
producing an incomplete palette.

#### Scenario: Use the complete sample palette

- **GIVEN** the sample contains every supported palette role with its built-in color
- **WHEN** the user copies it to their configuration unchanged
- **THEN** the effective palette matches the built-in palette
- **AND** editing any one color changes only its configured role.

### Requirement: CFG05 Exhaustive self-documenting sample

The distributed reference sample SHALL cover every supported configurable key
and every built-in keybinding. It SHALL include all palette defaults as active
assignments. Supported optional settings MAY be commented out when unused;
their comments SHALL explain how to enable them.

For each setting, the sample or its directly linked reference SHALL state its
meaning, type, default, allowed values or bounds, units where applicable,
scope, and inheritance behavior. Comments and example values SHALL be English.
Commented settings SHALL describe supported behavior, not advertise unfinished
features. The complete command catalog SHALL be discoverable without inventing
an example for every possible user-defined key sequence.

#### Scenario: A new supported option is added

- **GIVEN** the schema adds a configurable key or palette role
- **WHEN** the change is reviewed for delivery
- **THEN** the matching sample/reference entry is present and accurate
- **AND** consistency checks reject missing or stale documentation/defaults.

### Requirement: CFG06 Commented examples preserve fallback behavior

A commented assignment SHALL have no configuration effect. Commenting out an
override SHALL restore the next inherited value. It SHALL NOT disable the
corresponding built-in feature or remove a built-in binding. This delivery
SHALL NOT provide an explicit unbinding or disable action. Configured binding
values SHALL resolve to existing command names.

The reference sample SHALL separate active defaults from commented optional
overrides clearly enough that users can predict this behavior.

#### Scenario: Comment out a custom binding

- **GIVEN** a configured chord overrides its built-in binding
- **WHEN** the user comments out that override
- **THEN** the built-in binding becomes effective again
- **AND** the chord is not implicitly unbound.

### Requirement: CFG07 Explicit ordered file composition

If configuration is split into files, the root file SHALL name its includes in
an explicit ordered list. Includes SHALL be resolved relative to the file that
declares them unless an explicitly supported absolute path is supplied.
Configuration directories SHALL NOT acquire an implicit load order from an
unspecified filesystem enumeration.

The order SHALL be built-ins, included files in declared order, the root
file's own values, then matching language overrides. Tables merge by supported
leaf key; arrays replace instead of implicitly concatenate. A later value
replaces an earlier value at the same scope. Duplicate TOML keys in one source
remain syntax errors, not a file-composition mechanism.

#### Scenario: Root file overrides included defaults

- **GIVEN** two ordered includes and the root define the same editor setting
- **WHEN** the complete configuration is resolved
- **THEN** the root value wins over both includes
- **AND** unrelated included values remain available.

#### Scenario: Explicit empty array

- **GIVEN** an earlier layer supplies a nonempty supported array
- **WHEN** a later layer explicitly supplies an empty array for that key
- **THEN** the effective array is empty where permitted by the schema
- **AND** it is neither concatenated with the old array nor treated as absent.

### Requirement: CFG08 Bounded include graph

Include resolution SHALL detect cycles using actual resolved file identities,
including aliases through symlinks. An explicitly requested missing or
unreadable include SHALL be an error. Include traversal SHALL allow at most 16 edges from root to leaf, 64 source
occurrences including the root, 256 KiB per source and 2 MiB total bytes read.
Repeated includes SHALL count toward the limits; active canonical path and
filesystem identities SHALL detect symlink and hard-link cycles. The root has
depth zero. A file SHALL be read under its byte limit, not read unbounded and
checked afterward.
The loader SHALL NOT fetch remote configuration or evaluate include paths as
shell expressions.

#### Scenario: Include cycle through an alias

- **GIVEN** included files reach an earlier source through a symlink alias
- **WHEN** the loader traverses the include graph
- **THEN** it reports the cycle with its relevant source paths
- **AND** no partial candidate becomes effective.

### Requirement: CFG09 Global and language-specific scope

Concern tables SHALL group global settings. Language tables SHALL override only
settings explicitly permitted at language/buffer scope. Global-only settings,
such as application storage roots and overall palette ownership, SHALL NOT
silently acquire per-language semantics.

For a buffer, the effective editing policy SHALL combine built-ins, resolved
global settings, and the selected language's partial override. Language
identification SHALL be independent of Tree-sitter grammar availability.
The same Go indentation preference can apply to plain text without adding a
Go parser. User associations SHALL override built-in associations. The model's
`files.associations` rules SHALL resolve overlapping matches without treating
shared extensions as invalid configuration: exact basename wins over suffix,
and the longest suffix wins within one source class.

After a successful rename or file reassociation, each affected buffer SHALL
re-evaluate its language-specific preferences from the settings loaded at
startup. This includes visited descendants of a renamed directory. All shared
views SHALL use the resulting buffer policy coherently. A canceled or failed
path change SHALL NOT change that policy. Re-evaluation SHALL NOT reload TOML,
convert existing text, or change its undo history or saved-state identity.

#### Scenario: Go changes only indentation width

- **GIVEN** global spaces, indentation width 4, and tab display width 4
- **WHEN** a matching Go override defines only indentation width 2
- **THEN** that buffer uses indentation width 2
- **AND** it inherits spaces and tab display width 4.

#### Scenario: Rename selects the new language preferences

- **GIVEN** a text buffer and different Go preferences loaded at startup
- **WHEN** its visiting path is successfully renamed from `notes.txt` to `notes.go`
- **THEN** its effective editing policy uses the Go overrides in every shared view
- **AND** existing text, undo history and saved-state identity remain intact.

### Requirement: CFG10 Indentation and literal tab display

The schema SHALL represent indentation style, indentation width, and literal
tab display width independently. With tab indentation, Tab SHALL insert one
U+0009 tab character. Its visual advance SHALL reach the next `tab_width`
display-column stop. With space indentation, Tab SHALL insert enough spaces
to reach the next `indent_width` display-column stop, rather than always
inserting a fixed number. These policies SHALL NOT confuse a key press, a
stored tab character, and its visual width.

Changing these settings SHALL NOT convert existing document whitespace,
normalize file content, or rewrite pasted text. A two-cell tab display width
is not a request to replace every tab with two stored spaces.

#### Scenario: Change tab display width without editing

- **GIVEN** a buffer contains literal tabs and has a known saved state
- **WHEN** its effective tab display width changes
- **THEN** projection and display-column calculations use the new width
- **AND** rope bytes, text revision, undo content, and dirty state do not change.

### Requirement: CFG11 Coherent display and rectangle policy

Every view of a buffer SHALL use that buffer's effective display policy for
tabs, graphemes, columns, selection, and rectangle commands. Applying a policy
change SHALL invalidate dependent geometry/caches and retain valid text
positions. It SHALL NOT reuse display-derived ranges from the previous policy.

A rectangle SHALL use the source buffer's display policy for extraction and
the destination buffer's display policy for insertion columns. Tab handling
SHALL follow the observed GNU Emacs 31.1 distinction: rectangle copy expands
selected tabs into the retained text without changing the source; rectangle
kill MAY retain complete tabs inside the extracted rows while materializing
partial tab edges as needed.

Yank SHALL insert retained rows at the same left display column on successive
destination lines. It SHALL NOT retabulate each row to force the original
right edge. Retained literal tabs therefore use the destination's tab stops.
The R20 replacement below records this decision while preserving rev's
grapheme, short-line and history guarantees.

#### Scenario: Reproject shared views

- **GIVEN** two views show a buffer containing tabs and wide Unicode clusters
- **WHEN** its display policy is replaced by a valid configuration
- **THEN** both views reproject against that policy
- **AND** independent points and canonical document text remain intact.

### Requirement: CFG12 Context-specific keybindings

User bindings SHALL map supported key sequences to existing hyphenated command
names. At minimum, editor, minibuffer, and tree contexts SHALL be distinguishable.
An override in one context SHALL NOT accidentally change another context's
input. Unspecified bindings SHALL inherit the built-in map for that context.

An override SHALL name an existing command. Disabling a built-in chord through
an empty, boolean, sentinel, or removal value SHALL NOT be supported.

Bindings SHALL use the common Rust command owner, with the same command-name
identity used by `M-x`. The loader SHALL NOT interpret a command string as a
shell command, function body, script, or new plugin operation.

#### Scenario: Add an editor alias

- **GIVEN** `next-line` is a supported Rust command
- **WHEN** `keys.editor` binds `M-j` to `next-line`
- **THEN** that chord invokes the existing command in editor context
- **AND** other contexts and unmodified built-in bindings retain their behavior.

### Requirement: CFG13 Validate key sequences before activation

The complete effective keymap SHALL be validated before activation. Unknown
commands, invalid chord syntax, conflicting normalized sequences, and an
ambiguous command/prefix collision SHALL produce actionable errors. A later
exact override of the same inherited chord is intentional replacement, not an
illegal duplicate across configuration layers.

Terminal-library limitations SHALL remain visible. Configuration SHALL NOT
make the editor claim it can distinguish chords the target terminal cannot
report. Complete sequences still dispatch once, and cancellation remains an
effective operation in transient contexts.

#### Scenario: Leaf command conflicts with a longer prefix

- **GIVEN** an effective binding exists for `C-x C-s`
- **WHEN** a new binding would also make `C-x` an immediately invoked command
- **THEN** ambiguous activation is rejected with the involved sequences
- **AND** the prior complete keymap remains usable.

### Requirement: CFG14 Startup errors keep usable defaults

A missing optional root configuration SHALL select built-ins without error.
A present malformed, unreadable, or semantically invalid candidate SHALL be
reported with useful source, key, and location context where available.
Startup SHALL keep the editor usable with complete built-ins and a visible
configuration diagnostic; the rejected candidate SHALL have no partial effect.

The loader SHALL NOT repair, rewrite, or overwrite user configuration as a
side effect of reading it. Secrets or unrelated environment contents SHALL NOT
be copied into diagnostics. An explicitly requested broken include is not an
absent optional root file.

#### Scenario: Bad color in an otherwise valid file

- **GIVEN** a configuration changes editing settings but contains an invalid color
- **WHEN** startup validates the whole candidate
- **THEN** the error identifies the bad key/source
- **AND** no mixture of accepted and rejected file settings becomes effective.

### Requirement: CFG15 Startup-only application

The editor SHALL read and validate configuration at startup. The resulting
effective configuration SHALL remain fixed for that session; changing source
files SHALL NOT trigger manual or automatic reload in this delivery.

Startup application SHALL be complete and coherent. It SHALL NOT silently save
files, execute a bound command, create filesystem entries, or treat preferences
as document edits. Runtime reload and automatic change watching MAY be
considered later, but SHALL NOT appear as available commands or settings now.

#### Scenario: Configuration edits wait for the next startup

- **GIVEN** an editor session uses valid settings and contains unsaved text
- **WHEN** the configuration file changes on disk
- **THEN** the running session retains its settings and unsaved text
- **AND** a later startup reads and validates the changed configuration.

### Requirement: CFG16 Preserve file-format and save guarantees

Configured newline/BOM preferences SHALL describe new files or explicit future
format-changing commands. Reading or applying configuration SHALL NOT rewrite
the existing line separators, BOM, final-newline state, or encoding of a visited
file. The UTF-8 baseline and safe file-publication contract remain in force.

Required confirmations, truthful publication outcomes, read-only enforcement,
and preservation of unsaved text SHALL NOT become casual disable switches in
the general preference catalog.

#### Scenario: Global LF preference and an existing CRLF file

- **GIVEN** a visited CRLF file without a final newline and a global LF preference
- **WHEN** configuration loads and the user saves an unrelated word edit
- **THEN** unaffected CRLF separators and final-newline absence remain preserved.

### Requirement: CFG17 Bounded history and ring preferences

If history retention and ring capacity are configurable, their bounds, units,
and eviction policy SHALL be documented. Reducing history retention SHALL
preserve whole reversible units and truthful saved-state identity. A soft
payload target SHALL be labeled as soft; it SHALL NOT be presented as a strict
process-memory maximum. Ring capacity remains positive and finite.

A valid policy change SHALL NOT silently make a partly applied transaction
irreversible. If runtime policy replacement is supported, it SHALL be admitted
coherently or leave the prior policy unchanged.

#### Scenario: Lower retention below a compound operation

- **GIVEN** the newest retained undo unit is a multi-line rectangle operation
- **WHEN** a valid policy lowers the soft retention target
- **THEN** pruning respects the documented complete-unit policy
- **AND** it does not retain only a subset of that operation's reversible rows.

### Requirement: CFG18 Configure bundled syntax behavior

Configuration MAY enable/disable highlighting and select among supported
bundled grammar associations. It SHALL NOT imply language servers, language
completion, remote grammar installation, or loading arbitrary native code.
Unsupported syntax continues as editable plain text under the editor baseline.

Language editing settings SHALL remain usable independently of syntax support.
Any size cutoff affects syntax work only, not whether ordinary text can be
opened or edited.

#### Scenario: Go editing preferences without a Go grammar

- **GIVEN** Go-specific editing preferences exist but only Rust syntax is bundled
- **WHEN** a Go file opens
- **THEN** its editing preferences apply
- **AND** it remains editable plain text without a grammar installation step.

### Requirement: CFG19 Tree and layout preferences preserve state

Tree visibility, width, root policy, sorting, and hidden-entry presentation MAY
be configured when supported. Window/display preferences MAY affect initial
geometry and presentation. They SHALL respect minimum geometry, symlink-cycle
safety, live buffer ownership, and deterministic focus behavior.

Preference changes SHALL NOT delete buffers, rename files, recursively traverse
an unbounded tree, or override pending user decisions.

#### Scenario: Requested tree width does not fit

- **GIVEN** the configured preferred tree width exceeds available terminal space
- **WHEN** layout computes actual geometry
- **THEN** it clips or hides the optional pane under the documented policy
- **AND** editing buffers and pending input remain intact.

### Requirement: CFG20 Catalog scope and capability truthfulness

The model's inventory SHALL distinguish confirmed configuration capabilities,
future controls, and any later product choices. A setting SHALL enter the
shipped schema/sample only when its underlying behavior is actually supported.
Defaults SHALL NOT change silently because a candidate setting was listed in
this proposal.

Every accepted setting SHALL have one owning component and a clear scope.
Configuration SHALL NOT become a means to bypass canonical edit/history/file
owners or expose unstable internal implementation details as permanent user API.

#### Scenario: Proposed cursor feature is not implemented

- **GIVEN** cursor blinking is listed as a candidate but is not delivered
- **WHEN** the reference sample is prepared for a release
- **THEN** it does not advertise blinking as a usable active or commented setting.

### Requirement: CFG21 Explicit source selection

Initial discovery SHALL use the user XDG configuration and its explicit
includes. Merely opening a project SHALL NOT discover and activate a repository
configuration file. Project-local discovery, environment interpolation, and
additional configuration scopes require an explicit future decision.

This boundary does not prevent includes from referring to a directory chosen
by the user. The file must be part of the explicitly selected configuration graph.

#### Scenario: Open a repository containing a configuration-looking file

- **GIVEN** a project contains its own `config.toml` not included by the user
- **WHEN** the user opens a source file in that project
- **THEN** that project file does not become an implicit configuration layer.

### Requirement: CFG22 Acceptance and reference consistency

Implementation SHALL verify schema parsing, per-leaf precedence,
include errors/cycles, language scope, keymap validation, and sample completeness
at terminal-independent boundaries. Representative terminal checks SHALL prove
that palette, tab display, custom binding, cancellation, and invalid-config
recovery work through the actual executable.

Required regression checks SHALL preserve canonical text, undo state, file
fidelity, view positions, and safe publication during preference changes.
Schema/sample checks alone SHALL NOT count as runtime acceptance evidence.

#### Scenario: Sample is syntactically valid but incomplete

- **GIVEN** the sample parses as TOML but omits a supported palette role or option
- **WHEN** catalog consistency is checked
- **THEN** the missing documentation/default is detected
- **AND** syntax success alone does not establish configuration acceptance.

## MODIFIED Requirements

The complete accepted replacements below retain the baseline identities and
existing scenarios. They are reconciled into the current baseline in Slice 30;
the original validated baseline remains identified separately. The R17 insertion policy follows the next-stop decision in CFG10.

### Requirement: R01 Product identity and runtime boundary

Source: sections 1-3, 16, 23-24. Acceptance: A02, A17.

The executable MUST be named `rev`. It MUST run in a terminal, accept ordinary
text without an insert-mode transition, and implement commands directly in
Rust. Normal execution MUST work without a network connection. Temporary
prompt, prefix, search, and tree focus states are part of non-modal editing.
The baseline MUST respect the exclusions under Scope and priority.

**Design default:** Linux terminals are the primary target. Other platforms
MAY work through the selected crates; broad platform certification is not
required. The interface uses compiled defaults and one semantic palette whose
roles MAY be overridden through the accepted declarative configuration contract.
Configuration SHALL NOT introduce a scripting or theme-package runtime.

#### Scenario: Ordinary offline editing

- **GIVEN** a supported terminal with no network connection
- **WHEN** the user launches `rev` and types ordinary text
- **THEN** an editable terminal buffer receives the text directly
- **AND** no interpreter, language server, or external editor service is required.

### Requirement: R17 Direct insertion and deletion

Source: sections 9.2-9.3. Acceptance: A02, A03, A09.

Printable input MUST insert at point. Enter MUST insert a line break using the
document's applicable format policy. Tab MUST insert under the effective,
documented indentation policy: one literal tab for tabs, or spaces through the
next indentation stop for spaces. Applying that policy MUST NOT convert existing
content or literal pasted text. Backspace MUST delete the preceding
character/grapheme without crossing buffer start. `C-d` (`delete-char` in the
built-in keymap) MUST delete the following character/grapheme without crossing
buffer end.

`C-k` / `kill-line` MUST kill to line end, or the line separator when already
at line end. `M-d` / `kill-word` and `M-DEL` / `backward-kill-word` MUST kill in
their respective directions. These sequences identify the default keymap;
explicit user overrides may choose other bindings for the same commands.
Region deletion MUST update point and mark consistently. Every affected view
MUST retain valid positions after an edit. Control/Meta combinations MUST
enter the effective keymap rather than insert their printable representation
or unknown control bytes.

#### Scenario: Kill a CRLF line separator

- **GIVEN** point at the end of a line followed by CRLF
- **WHEN** `C-k` invokes the default `kill-line` binding
- **THEN** the logical separator is removed coherently and retained for yanking
- **AND** all views remain at valid positions.

### Requirement: R20 Rectangle selection and block editing

Source: section 10.2. Acceptance: A10.

The following rectangle commands MUST be implemented. Rectangle columns MUST
be display columns, not byte offsets. Operations MUST NOT divide a grapheme,
combining sequence, or double-width character into invalid text. Copying a
short source line MUST NOT invent trailing text. Insertion MAY add spaces only
where needed to reach the target column. The rectangle store MUST be separate
from the linear kill ring. Each rectangle mutation MUST undo as one coherent
operation over all affected lines.

| Keys | Command | Behavior |
| --- | --- | --- |
| `C-x SPC` | `rectangle-mark-mode` | Toggle rectangle selection for the active interaction. |
| `C-x r k` | `kill-rectangle` | Remove and retain the selected rectangle. |
| `C-x r M-w` | `copy-rectangle-as-kill` | Retain the rectangle without editing. |
| `C-x r y` | `yank-rectangle` | Insert the retained rows at point across successive lines. |

These keys describe defaults under R22. Extraction MUST use the source
buffer's effective display policy. Rectangle copy MUST materialize selected
tab cells as spaces in its retained rows without changing source text. Kill
MUST retain complete selected tabs and handle partially selected tab edges
through the source policy. Yank MUST align each row's left edge to the
destination display column and insert the retained row contents. It MUST NOT
retabulate retained rows to force their original source width; literal tabs
use the destination buffer's tab stops.

This tab/column behavior follows the observed GNU Emacs 31.1 copy, kill and
yank operations. Rev retains its existing whole-grapheme, short-source-line
and atomic-history rules. Additional clear/open/number/register operations
remain outside this change.

#### Scenario: Rectangle spans short and mixed-width lines

- **GIVEN** selected display columns across a tab, a wide grapheme, and a short line
- **WHEN** the user copies, kills, and yanks the rectangle
- **THEN** whole Unicode text and adjacent unselected content remain valid
- **AND** copy adds no absent trailing content and insertion pads only as needed.

#### Scenario: Undo a multi-line rectangle

- **GIVEN** a rectangle command changed several lines
- **WHEN** the user invokes undo once
- **THEN** every affected line is restored as one transaction.

#### Scenario: Copy a tabbed rectangle into a different tab policy

- **GIVEN** source rows `a\tb` and `c\td` with tab width 4
- **WHEN** their existing columns are copied and yanked into a buffer with tab width 8
- **THEN** retained rows are `a   b` and `c   d`, without source mutation
- **AND** both rows start at the selected destination display column.

#### Scenario: Kill retains complete tabs for destination interpretation

- **GIVEN** the same source rows and a rectangle containing each complete tab
- **WHEN** that rectangle is killed and yanked into a buffer with tab width 8
- **THEN** retained tabs stay literal tabs and use the destination's tab stops
- **AND** row starts remain aligned without forcing the original right edge.

### Requirement: R22 One static command registry

Source: sections 12.1-12.2, 18.3. Acceptance: A11, A17.

Bindings and `M-x` MUST invoke the same Rust command handler. The registry
SHOULD be a direct static mapping to functions or a small enum. Every listed
command MUST work; a functional-looking stub is not an implemented command.
Public names MUST be lowercase and hyphenated, preferring the corresponding
Emacs name. Internal identifiers use idiomatic Rust naming under R36.

Key tables in the editor baseline describe built-in bindings. Explicit valid
user configuration MAY override those sequences or add aliases while retaining
the existing named operations and context rules. Without an override, the
documented built-in mapping MUST remain effective.

The registry MUST NOT introduce a command interpreter or public plugin
protocol. Names MUST NOT imply absent Emacs subsystems.

#### Scenario: Equivalent named and bound command

- **GIVEN** equivalent sessions with the same active buffer, layout, and built-in keymap
- **WHEN** one invokes `C-x 3` and the other invokes `M-x split-window-right`
- **THEN** both execute the same operation and produce equivalent split behavior.

#### Scenario: User alias retains command identity

- **GIVEN** a valid editor alias names `split-window-right`
- **WHEN** that alias or `M-x split-window-right` is invoked
- **THEN** both reach the same supported Rust handler.

## Baseline reconciliation before implementation

During delivery, reconcile the full accepted editor baseline with this change
without rewriting historical implementation evidence. In addition to the named replacements,
the baseline's decision that no user configuration system is selected and
its fixed-palette summary must be updated explicitly. Existing key tables must
be consistently described as defaults under the accepted R22 amendment.

The configuration model adds a TOML file format and configuration state. It
does not add on-disk session restoration or addon records. The existing editor
model remains authoritative for document bytes, buffer/view identities,
transactions, dirty state, history, and file-publication guarantees.

The existing rectangle reference describes retaining complete tabs during
copy. Reconcile that description and its checks with the selected R20 copy
expansion rule; keep the explicit distinction from kill and destination yank.

## Closed decisions

- Startup-only load; no reload command or file watcher.
- Association defaults plus user precedence; overlapping patterns are resolved
  by the model rather than requiring users to allocate extensions uniquely.
- Schema version 1, with omission accepted as version 1; TOML 1.1.0 input.
- Explicit include order and finite graph/input limits under CFG08.
- Reassignment to supported commands; no explicit unbinding.
- Next-stop Tab semantics; conversion commands remain a separate future ticket.
- Exact accepted catalog and bounds are in the model. The candidate inventory
  is a roadmap boundary and cannot add implementation scope.

No product decision remains open for slicing. Runtime evidence remains pending.

## References

- [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir/latest/)
- [TOML 1.1.0](https://toml.io/en/v1.1.0)
- [GNU Emacs rectangle behavior](https://www.gnu.org/software/emacs/manual/html_node/emacs/Rectangles.html)
- [GNU Emacs rectangle operations](https://github.com/emacs-mirror/emacs/blob/master/lisp/rect.el),
  cross-checked through isolated copy/kill/yank observations in GNU Emacs 31.1.
- OpenSpec document-format reference in the shared workspace specification contract,
  pinned there to version 1.15.0 and revision
  `6d04a1b58dedc95729d34e4f223d9d483aae210a`.
