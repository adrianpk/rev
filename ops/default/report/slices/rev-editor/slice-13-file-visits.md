# Slice 13: Reuse canonical visited files

Status: reviewing
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-13-file-visits`
PR: pending

## Purpose

Give file visits one destination identity and one live buffer, shared by startup
and focused `find-file` entry without replacing unsaved text through an alias.

## Delivered Behavior

- `C-x C-f` and the named `find-file` handler open a labeled argument prompt.
  Prompt edits stay in the minibuffer; Enter consumes the argument once and
  `C-g` returns to the editing origin without visiting its partially typed path.
- Existing paths resolve to their canonical referents. Symlink prefixes resolve
  before `..`; relative paths and normalized absent suffixes converge on the
  intended absolute destination without creating files or missing parents.
- Reopening a live destination selects its existing canonical buffer before
  reading disk content again. Its unsaved text, history and format metadata
  survive external file changes and alias visits. Reopening the active buffer
  also retains its point and scroll state.
- An absent destination opens an empty buffer with an initial clean in-memory
  root and an explicit not-yet-present association. Startup uses the same visit
  owner. Opening alone makes no filesystem publication or save acknowledgment.
- Each buffer retains its destination separately from its unique display label.
  Equal basenames receive distinguishable suffixes, including collisions with
  existing internal labels and literal suffix names. Non-UTF-8 OS paths retain
  exact identity even when their display labels need lossy rendering.
- Rejected unvisited input preserves the active text, association, label,
  history and view. Directories, non-directory components, invalid UTF-8,
  unreadable files, empty arguments and dangling symlinks report an error.
- The prompt cursor belongs to the echo row. Horizontal clipping retains the
  cursor through wide/combining input, tiny frames and zero-size suspension.

## Implementation Notes

- T13.1: `1b6ab4d133d1cb54538fc9a63595a47b971c95ca` implements resolution,
  associations, reuse, unique labels and focused command/projection behavior.
  Ownership and operation-context follow-ups end at
  `38471f583db48ceaa8602febe5183e783dd88d1f`.
- T13.2: `d5ff90a3cde1cb54309309ebf7165c558bad3df4` exercises native identity,
  prompt recovery and real terminal journeys. The viewport fixture in
  `bce68fa55cb13316148deb9992d0712cc902e2c1` observes a handled key before
  its first resize, so event registration precedes the tested signal.
- The registry owns the path-to-buffer index. A private visit candidate either
  names an admitted existing owner or holds one boxed new buffer. All
  recoverable work completes before index, registry and view publication.
- A reused owner is selected without decoding another document copy. A new
  owner consumes the strict ingress rope directly. Labels are presentation
  metadata; at most N+1 candidates distinguish N live names.
- Pending command ownership distinguishes prompt cancellation from ordinary
  keyboard quit, preserving both the editing origin and existing recenter
  reset behavior. Printable argument changes request redraw; releases remain
  inert. Prompt strings remain bounded by the existing 4096-scalar limit.
- Source checks pass safe Rust, five parameters, nesting at most three and
  cyclomatic paths at most ten.

## Contracts Added or Changed

- `file::resolve` returns a `Location` with a canonical or intended `path` and
  resolution-time `is_present` observation. It creates no filesystem entries.
- `Buffer::file` exposes the retained association independently of `name`.
  `Buffers::visit` owns destination reuse and validated new-buffer installation.
- `Session::load` shares that owner with startup and `find-file`. Failed direct
  admission leaves editing context intact; completed prompt arguments return
  input focus to editing, including after a rejected visit.
- `Command::FindFile` joins the implemented registry and `C-x C-f` binding.
  `Outcome::Changed` identifies prompt edits that need presentation updates.
- `Frame` projects the focused prompt and its bounded echo-row cursor.
- `CHANGELOG.md` describes interactive visits, reuse and new destinations.

## Files of Interest

- `src/file/location.rs`: path resolution and truthful presence observations.
- `src/buffer/visits.rs`, `src/buffer.rs`: private candidates, index and labels.
- `src/session/file.rs`, `src/session.rs`: common visit and pending argument owners.
- `src/command.rs`, `src/input.rs`, `src/input/prompt.rs`: command/focus routing.
- `src/display/prompt.rs`: prompt clipping and cursor projection.
- `tests/files.rs`, `tests/commands.rs`: identity, association and input contracts.
- `scripts/check-pty.py`: actual interactive visits, startup and terminal lifetime.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; 121 tests.
- `python3 scripts/check-source.py`: passed.
- `cargo test --test files`: passed; twenty-two tests, zero skips. Added cases
  cover dirty canonical reuse, symlink-parent semantics, relative and absent
  paths, duplicate/raw names, rejected association/index changes, newly
  appearing disk targets, cancellation and exact prompt cursor geometry.
- `python3 scripts/check-pty.py --suite terminal`: passed; twenty-two cases,
  zero skips. Added journeys check actual `C-x C-f`, dirty symlink reuse,
  duplicate labels, cancellation, absent startup paths, consumed empty input
  and unchanged filesystem entries. Resolution failures carry their exact
  visit operation context. Owned fixtures stop after their checks finish.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
