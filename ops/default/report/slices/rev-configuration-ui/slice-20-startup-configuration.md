# Slice 20: Activate startup configuration

Status: delivered
Delivery set: rev-configuration-ui
Plan: [Configuration and Interaction Delivery](../../../plan/rev-configuration-ui.md)
Tracker: [Configuration and Interaction Tracker](../../../tracker/rev-configuration-ui.md)
Branch: `feat/rev-config-ui-20-startup-configuration`
PR: [#69](https://forge.adrianpk.com/adrianpk/rev/pulls/69)
Integrated: `2736cc112f04968c2780a496a18020397d373cb9`

## Purpose

Activate XDG configuration discovery and one validated immutable snapshot in the
executable before creating buffers and input owners, with usable default fallback.

## Delivered Behavior

- Executable startup resolves the user XDG roots and reads the sole implicit
  `rev/config.toml` once. Existing bounded loading, explicit includes, partial
  precedence and whole-candidate validation run before any buffer is created.
- The admitted snapshot reaches the existing buffer/language/format, palette,
  window/scroll, tree/syntax, retention and context-keymap consumers. Library
  `Session::new` and `empty` retain their explicit/default construction behavior.
- A missing configuration is silent and uses complete built-ins. Invalid TOML,
  invalid leaves/maps, broken roots and missing explicit includes produce a
  source/key diagnostic where available while starting a usable editor with
  complete defaults. Unresolvable XDG roots produce an environment diagnostic.
- The diagnostic is installed after initial file ingress, so loading the command
  line file cannot erase it. Initial file failures keep their existing error
  behavior. Configuration failure cannot partially apply palette, key, editing
  or file-format preferences.
- Startup reads no implicit project-local source, executes no path expansion,
  writes no settings and creates no empty configuration/data/state/cache roots.
  Configuration bytes remain unchanged after rejection.
- Existing files retain their observed BOM/newlines. New buffers receive admitted
  format defaults. Source edits affect a new startup; the running session and
  buffers it opens later retain the original snapshot even after sources vanish.
- Real controlling-terminal checks verify configured RGB/cursor, indentation,
  BOM/CRLF, key dispatch, restart-only changes, nonfatal rollback, silent absence
  and ordinary terminal restoration.

## Implementation Notes

- T20.1: `5360fd648c27305d4feef87a2b1e4955df9f71db` adds environment preparation
  and a startup session constructor, then delegates the terminal entrypoint to it.
  Existing configuration and consumer owners retain all admission and mutation.
- T20.2: `190b2e22967c8826692b8e03f85a2ea73ce614c5` tests silent absence,
  complete fallback, source/key diagnostics after ingress, broken roots and
  environment resolution without modifying the parent process environment.
- T20.3: `d9004750b8165e7db2deb580171f49da022fd5c1` adds restart/consumer and executable PTY
  regressions, updates startup documentation and records the delivered capability.
- Isolated child processes supply HOME, XDG roots and CWD without unsafe global
  environment mutation. Their fixtures cover malformed project-local files,
  explicit user includes and byte-preserving read-only discovery behavior.
- The PTY suite reuses the existing terminal fixture owner and builds the actual
  executable. It checks saved bytes and wire RGB/cursor sequences rather than
  inferring behavior from configuration fields. All four owned sessions exit.

## Contracts Added or Changed

- `config::Startup::from_env()` selects the user root and returns a complete
  admitted snapshot or complete defaults plus one diagnostic. It never creates
  directories or selects a CWD configuration as fallback.
- `Session::startup(path, size)` composes all consumers from that snapshot before
  initial file ingress and presents configuration diagnostics afterward.
- The executable's terminal initialization uses this constructor. Cursor color
  is selected from the same admitted palette before the first frame is drawn.
- Running-session loading and rendering continue to use the retained snapshot;
  no reload command, watcher, polling or second source-selection pass is added.

## Files of Interest

- `src/config/startup.rs`, `config.rs`: environment discovery and startup contract.
- `src/session/startup.rs`, `session.rs`: snapshot composition and diagnostic order.
- `tests/configuration/startup.rs`, `startup_errors.rs`, `startup_lifetime.rs`:
  seven native cases with twelve isolated child invocations.
- `scripts/check-startup.py`: four actual executable controlling-PTY sessions.
- `docs/reference/defaults.md`, bindings/prompts/tree/syntax/recovery references,
  `README.md`, `CHANGELOG.md`: discovery, supported overrides and restart behavior.

## Validation

Development checks:

- `cargo test --test configuration startup::`: passed; seven native startup cases.
- `cargo test --test commands palette::context_reference`: passed.
- `cargo clippy --lib --test configuration -- -D warnings`: passed.
- `cargo run --quiet --example source-check -- --source src/session/startup.rs --source src/config/startup.rs --source src/session.rs --source tests/configuration/startup.rs`: passed.
- `cargo run --quiet --example source-check -- --source tests/configuration/startup.rs --source tests/configuration/startup_errors.rs`: passed.
- `cargo run --quiet --example source-check -- --source src/session/startup.rs --source src/config/startup.rs --source tests/configuration/startup.rs --source tests/configuration/startup_errors.rs --source tests/configuration/startup_lifetime.rs`: passed; nesting at most two.
- `python3 scripts/check-startup.py`: passed; four terminal sessions, zero skips.

Final evidence `HDKGXOS4KJGTRHA2HCIOXHQGUW` passed all nine ordered checks
on clean head `2736cc112f04968c2780a496a18020397d373cb9`: 269 native tests
and four terminal sessions, zero skips. The controller integrated PR #69 into dev
at that same commit. Acta merge event: `01M5Z0BTG0TPAP5B7TFEBPQMYG`.
Required checks precede the focused executable startup suite:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test configuration`
- `cargo test --test commands`
- `cargo test --test files`
- `cargo test --test recovery`
- `python3 scripts/check-source.py`
- `python3 scripts/check-startup.py`
- `git diff --check`

## Risks and Follow-ups

Configuration sources are read only during startup; source edits require a new
session. Diagnostics use the existing clipped echo surface at small geometry.
Vertical command completion and path completion retain their recorded successor
slice boundaries on the existing immutable configuration/input owners.
