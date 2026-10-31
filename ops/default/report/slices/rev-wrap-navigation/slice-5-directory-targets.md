# Slice 5: Directory targets and initial tree expansion

Status: delivered
Delivery set: rev-wrap-navigation
Plan: [Wrap and Navigation](../../../plan/rev-wrap-navigation.md)
Tracker: [Wrap and Navigation Tracker](../../../tracker/rev-wrap-navigation.md)
Branch: `feat/rev-wrap-navigation-05-directory-targets`
PR: [#178](https://forge.adrianpk.com/adrianpk/rev/pulls/178)

## Purpose

Deliver NAV01, NAV02, NAV03 and NAV06 through shared CLI/find-file target
admission, explicit root intent and one-time first-level tree initialization.

## Delivered Behavior

- Startup accepts files and directories, including `rev .`, relative/absolute
  paths, Unicode and spaces. A directory opens a visible expanded tree without
  creating a document or changing launch cwd. Empty roots remain valid.
- Return accepts a navigated candidate or the typed draft; an empty draft opens
  the prompt base. A directory closes the prompt and preserves live documents,
  dirty text, history and shared editing views. Tab stays a completion action.
- Explicit directory roots survive nested and outside file visits until replaced
  by another directory choice. Otherwise auto/file-parent roots follow the file
  parent, while explicit cwd policy retains launch cwd. Files keep tree visibility.
- First manual or configured tree presentation initializes immediate entries.
  Child directories remain lazy. Later hide/show retains selection, viewport and
  deliberate collapse. A new visible automatic root initializes once again.
- Narrow geometry retains root, initialization and focus intent. Widening can
  restore tree focus; explicit editing or cancellation releases that intent.
- Find-file works through the actual tree keymap and M-x. Failed target/list
  admission retains the prior usable root, prompt and canonical document state.
  Existing tree symlink leaf actions retain regular-file admission.

## Implementation Notes

File and directory targets share checked component and alias resolution,
including symlink-parent semantics. Regular-file Location/Observation and
canonical buffer reuse stay authoritative. Directory and new visible automatic
root candidates complete fallible observation before registry/view publication.
A directory-only session retains its existing scratch identity. Startup defers
presentation until its target is admitted, avoiding an unrelated cwd listing.

Each directory observation processes at most 4096 entries and 512 KiB of raw
name bytes. These reuse the native path provider's existing ceilings. Tree
initialization previously had no explicit ceiling; it now uses bounded reads
for initialization, expansion and refresh, including names hidden by policy.
Exact-limit observations are conservatively incomplete without another read.
Retained incomplete entries show `[incomplete]` and the action reports
`Incomplete tree listing: directory observation limit reached`.

Initialization succeeds once per root, independently of later collapse state.
Failed first observation does not set that evidence. Root changes reset it;
renames retain the existing root's intent through its normal reconciliation.
Rendering and visible selection movement never enumerate directories. There
is no recursive discovery, watcher, persisted root record or new configuration
knob. The first observed subset is sorted under existing tree preferences.

The factory tree map admits `C-x C-f`; the packaged English sample matches it.
Other editor mutations retain their focused admission rules. Product references,
User Guide examples and the existing Unreleased entry describe the new behavior.

## Contracts Added or Changed

Return accepts a directory as a complete browsing target. Empty acceptance uses
the current visiting file's parent, otherwise the chosen directory or launch cwd.
Explicit root intent takes precedence over automatic/configured roots. First
presentation observes immediate children once, with bounded incomplete-list
feedback. Buffer text, source identities and undo owners remain unchanged.

Selected-candidate Tab behavior and clicked-entry activation retain their
separate approved slice owners.

## Files of Interest

- `src/session/file.rs`: shared target admission and prepared publication.
- `src/tree/root.rs`: explicit and automatic root choices.
- `src/tree.rs`, `src/tree/entry.rs`: initialization, focus intent and bounds.
- `src/file/location.rs`: shared component/alias resolution.
- `src/input/keymap/`: native tree find-file dispatch and availability.
- `tests/files/directories.rs`, `tests/tree/initialization.rs`: canonical state,
  root precedence, first presentation, permissions, aliases and exact limits.
- `scripts/check-directory-targets.py`: focused existing-PTY driver for actual
  CLI input, cwd/byte preservation, failed startup and terminal restoration.

## Validation

T5.1 commit `c5a8568c938772c69018c18fa5f2bb935b003632` implements target
admission; T5.2 commit `18ce4a7f1138511ef060b79fbde16755a8a9c494` completes
initialization, bounded listings, focus intent and native regression coverage.
The latter passed all-target Clippy and five focused tree-pointer tests. Its
documentation check caught an outdated tree keymap count; commit
`121c167275df9e2f19f5d5470859bdb13bb86b49` corrects that reference without
changing the check.

Controller receipt `5BMRLUDPOYEPF7E4V3XQYLLXMZ` passed these exact ordered
commands at clean head `121c167275df9e2f19f5d5470859bdb13bb86b49`:

1. `cargo fmt --all -- --check`
2. `cargo test --locked --test files --test tree --test path_completion --test configuration --test panels`
3. `git diff --check`
4. `cargo run --locked --quiet --example documentation-check`
5. `python3 scripts/check-directory-targets.py`

The native suites passed 336 tests: 133 configuration, 74 file, 19 panel,
43 path-completion and 67 tree cases. These include split-view identity/history
preservation and directory-symlink tree-leaf rejection. Documentation passed
305 commands, 179 configuration entries, 58 pages and 330 local links/anchors.
Eight successful CLI/PTY journeys and denied-directory startup verified exact
source/cwd preservation, bounded immediate presentation and terminal restoration.

Additional successful commands at the unchanged runtime implementation head
`18ce4a7f1138511ef060b79fbde16755a8a9c494` were:

- `cargo clippy --locked --all-targets -- -D warnings`
- `cargo test --locked --test pointer tree`

Legacy expectations now exercise approved directory acceptance, automatic-root
updates, an initially expanded root and retained focus intent. Source/view/mark/
history assertions remain in place. The regular-file loader's original directory
rejection case remains intact; session directory admission is a separate target
operation. The combined delivery gate is reserved for the final acceptance phase.

Canonical PR #178 merged into `dev` at
`941faa796047ef7b2dbc809afe495903f690baaa`. Final controller receipt
`HAYGF7IDRKHEKYSHCJNEVE2NZW` passed the exact configured three-command prefix
and all 336 tests on that clean PR-reference head. The controller recorded the
merge in Acta. Slice 5 of 8 completed.

## Risks and Follow-ups

A bounded directory observation can still wait on the filesystem. Incomplete
lists retain an observed subset rather than claim full sorted coverage. Later
refresh is explicit; no automatic retry or background discovery is introduced.

Both directory tickets are solved after verified integration.
