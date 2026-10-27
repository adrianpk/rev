# Slice 25: Open selected and new file paths safely

Status: reviewing
Delivery set: rev-configuration-ui
Plan: [Configuration and Interaction Delivery](../../../plan/rev-configuration-ui.md)
Tracker: [Configuration and Interaction Tracker](../../../tracker/rev-configuration-ui.md)
Branch: `feat/rev-config-ui-25-find-file-integration`
PR: [#74](https://forge.adrianpk.com/adrianpk/rev/pulls/74)

## Purpose

Accept selected or typed find-file paths through canonical file ownership, retain
failed drafts, and descend into directories without visiting them as text buffers.

## Delivered Behavior

- C-x C-f and M-x find-file share one retained path prompt. Return after explicit
  navigation accepts that candidate's original identity, independently of its
  clipped or escaped label. The initial highlight does not choose a candidate.
- Return on a freely typed draft accepts that filename, including a prefix of an
  existing suggestion or a new name. Completion and opening never create disk
  entries or save output; missing files and parents remain absent until saving.
- Accepted directories descend in the same prompt, preserving relative/home
  spelling and a trailing separator. The prompt bound applies atomically. Child
  symlinks remain observed leaves until an explicitly accepted path follows them.
- Files use the existing prepare/publish ingress and canonical buffer registry.
  Relative and symlink aliases reuse dirty text, history and saved-state identity.
  Returning to the current destination also retains its qualified view and point.
- New buffers use the startup snapshot's language and new-file format settings.
  Changes to TOML after startup do not affect them. Existing text retains its
  separators, BOM and final-newline state without a configuration-driven rewrite.
- Invalid contents, non-directory components, denied access, broken/cyclic links
  and removed selections retain the editable draft and original buffer. An
  observed selection that disappears reports an error rather than becoming an
  implicit new file. Retrying or C-g remains available.
- UTF-8 control-containing names open only through their original selected path;
  escaped labels never supply filename bytes. Non-UTF-8 names remain explicitly
  unsupported without visiting a lossy replacement spelling.
- Configured accept/navigation aliases use the same stages. Repeat and release
  events cannot descend again, accept a following stage or insert leaked text.

## Implementation Notes

- T25.1: `27b9fd9133d8a018a29a8ac2b47b7a64a98a00c2` adds a retained path-accept
  outcome, selected/typed acceptance, directory continuation and four native
  acceptance regressions. Empty-path failure now retains find-file focus.
- T25.2: `cb8e0aa1b0bb3bb9d87a1c499dd6cca46789f4ae` validates directory access
  before changing its draft and adds four canonical-reuse/startup-policy journeys.
- T25.3: `0f15778c80e7e05e165531a5408dccb397f2c8fb` adds six failure/identity regressions,
  two configured accept/bound regressions, two actual terminal journeys and
  delivered behavior references and this report.
- Input retains the bounded prompt on PathAccept. Session delegates selection
  interpretation to file navigation, directory text to the existing prompt bound,
  and file opening to Session::load. Only successful ingress consumes the prompt
  and pending navigation after canonical preparation succeeds.
- Directory acceptance validates access without enumerating entries; actual
  navigation uses the existing bounded provider. Rendering and filtering remain
  independent of filesystem access. Selection acceptance does not refresh or
  reinterpret a changed sort position.
- The existing absent-path PTY journey now cancels the retained empty-path prompt
  before typing into the document. Its text assertions tolerate palette SGR while
  retaining exact buffer, point, normalized-reuse and no-creation checks.

## Contracts Added or Changed

- UI03: selected versus freely typed Return, directory descent, canonical dirty
  reuse and new files remaining in memory across both find-file entry points.
- UI04: original candidate path identity, unsupported-name refusal, retained exact
  errors/drafts, and symlink traversal only through explicit acceptance.
- CFG09: canonical ingress applies the frozen language policy to new destinations.
- CFG16: new-file defaults do not rewrite existing formats or publish saves.
- `Outcome::PathAccept` retains the active path prompt until the file owner
  succeeds. Ordinary write/save and other argument owners retain their contracts.

## Files of Interest

- `src/input/path.rs`, `src/input.rs`: retained native acceptance intent.
- `src/file/completion/navigation/accept.rs`: selected/typed path interpretation.
- `src/session/path.rs`, `session/file.rs`: directory continuation and canonical ingress.
- `tests/path_completion/acceptance.rs`, `visits.rs`, `failures.rs`: 14 new journeys.
- `tests/commands/context_keymaps/flows.rs`: accept aliases and directory bounds.
- `tests/files.rs`: empty-path error focus and cancellation.
- `scripts/check-path-acceptance.py`, `scripts/check-pty.py`: actual terminal checks.
- Prompt, binding and provider references; `CHANGELOG.md`: delivered behavior.

## Validation

Successful development checks:

- `cargo test --test path_completion acceptance::`: four entry-point, selected/typed, directory and retained-error cases passed.
- `cargo test --test path_completion visits::`: four canonical ownership and frozen-policy cases passed.
- `cargo test --test path_completion failures::`: six real filesystem failure/identity cases passed, with no skipped permission checks.
- `cargo test --test files prompt_geometry`: retained empty-path prompt, tiny geometry and cancellation passed.
- `cargo test --test files format::`: five format regressions passed.
- `cargo test --test recovery paths::failed_paths`: real permission/symlink recovery passed.
- `cargo test --test commands find_file_aliases`: existing configured file prompt aliases passed.
- `cargo test --test commands path_accept_aliases`: configured accept phases and directory/file continuation passed.
- `cargo test --test commands path_directory_bound`: atomic directory acceptance bound and new-name retry passed.
- `cargo clippy --lib --test path_completion --test commands --test files -- -D warnings`: passed.
- Focused source-check over all changed Rust modules and test roots passed: at most five parameters, cyclomatic complexity ten and nesting two.
- `python3 scripts/check-path-navigation.py`: two RGB/NO_COLOR navigation sessions passed, zero skips.
- `python3 scripts/check-path-acceptance.py`: two RGB/NO_COLOR acceptance sessions passed, zero skips; fixture bytes and directory entries unchanged.
- `python3 -c 'import runpy; from pathlib import Path; runpy.run_path("scripts/check-pty.py")["new_visit"](Path.cwd())'`: absent startup paths, retained empty-path focus, normalized live reuse and no filesystem creation passed.

Final readiness uses controller evidence on the clean PR head after the
PR-reference follow-up commit, with this exact ordered command list:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test path_completion`
- `cargo test --test files`
- `cargo test --test commands`
- `cargo test --test recovery`
- `python3 scripts/check-source.py`
- `python3 scripts/check-path-navigation.py`
- `python3 scripts/check-path-acceptance.py`
- `python3 -c 'import runpy; from pathlib import Path; runpy.run_path("scripts/check-pty.py")["new_visit"](Path.cwd())'`
- `git diff --check`

## Risks and Follow-ups

Filesystem state can change after observation. Acceptance checks the requested
identity and canonical ingress reports any resulting failure without discarding
the draft or active buffer. Integration and delivery-set gate evidence remain
owned by the controller.
