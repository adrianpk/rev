# Slice 25: Resolve Meta Escape and undo aliases

Status: delivered
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-25-terminal-keys`
PR: `#25`
Integrated: Rebase + Fast-forward into dev at `68c55eb542060a0b99ef2ba90118e0a975da7105`.

## Purpose

Resolve supported terminal key reports and finite Escape-prefix input without
blocking resize, cancellation, guarded exit or terminal restoration.

## Delivered Behavior

- Reported Alt and Meta modifiers reach the same bindings, preserving reported
  `<` and `>` characters. A separately reported Escape stages Meta for 100 ms;
  its next plain chord before the deadline follows the focused keymap.
- An existing `C-x` or `C-x r` prefix retains ownership of an Escape-modified
  suffix. Invalid nested sequences clear their prefix without inserting text
  or invoking an unrelated linear command.
- At or after the deadline, standalone Escape clears key prefixes and displays
  `Escape`. Prompt/decision focus remains owned until explicit cancellation.
  A later fresh chord is processed normally, without being lost or modified.
- `C-g` cancels pending Meta immediately. Control chords retain their meanings,
  so editing focus can request guarded exit while Meta is pending. Resize stays
  actionable, and ordinary I/O errors retain normal guard cleanup.
- Reported `C-/`, `C-_` and `C-x u` reach real undo. The Unix legacy `0x1f`
  report, decoded by Crossterm as Control-7, also reaches undo. Control-space,
  Control-`@` and normalized NUL retain mark-command parity.
- Releases and held Escape/prefix suffixes do not dispatch again or extend the
  deadline. Consumed and rejected suffix repeats cannot become document text;
  a distinct new press restores normal input.
- Staging Meta preserves current kill/yank continuity for its eventual command.
  Expiry, cancellation or an unbound chord ends that continuity. Focused input
  never leaks into the underlying document.

## Implementation Notes

- T25.1: `3c1f54f394d1c445ca78491a8636d8925347561c` adds finite Meta timing,
  key normalization, timed event polling and terminal input reference.
- T25.2: `c3be5ef27235601b0752d524e6a9a3ef64c447fc` adds native timing,
  phases, focus, guarded exit and real terminal report evidence.
- The input owner retains one optional monotonic deadline alongside the current
  chord prefix. It exposes remaining duration and one-shot expiration; no timer
  thread, sleep or unbounded event queue is introduced.
- The session polls Crossterm only for the remaining Escape duration. Timers are
  processed before each event, and an ignored event cannot hide a required expiry
  redraw. Native callers can provide deterministic times and pump idle expiry.
- Escape-modified suffixes are routed through the existing context and keymap.
  Meta display staging preserves editing continuity until resolution; ordinary
  prefix messages and rejected input retain the established invalidation rules.
- Key normalization folds reported Meta into Alt and maps the legacy undo alias.
  Prefix completion/rejection suppresses held suffixes until a fresh press.

## Contracts Added or Changed

- `META_TIMEOUT` defines the 100 ms Escape deadline. `Input::handle_at`,
  `remaining` and `expire` expose deterministic event/deadline processing.
- `Prefix::Meta` exposes the staged modifier without discarding an outer chord
  prefix. A second separately reported Escape clears it immediately.
- `Session::handle_at` and `expire_input` support timed native hosts; terminal
  execution uses the same path through finite event polling.
- `docs/reference/terminal-input.md` records supported reports, focus/phase
  policy, standalone Escape behavior and unavoidable library/terminal limits.
- Documentation indexes, README and `CHANGELOG.md` link or describe the delivered
  input policy and legacy undo support.

## Files of Interest

- `src/input/meta.rs`, `src/input.rs`: deadline and focused prefix ownership.
- `src/input/key.rs`: modifier normalization and legacy undo alias.
- `src/input/inventory.rs`: focus acquisition clears pending Meta.
- `src/session/events.rs`: deadline polling and native event delivery.
- `src/session.rs`: modifier staging versus command continuity boundaries.
- `tests/commands/terminal.rs`: native report, timing, phase and focus evidence.
- `scripts/check-pty.py`: actual reports, idle expiry and guarded exit journeys.
- `docs/reference/terminal-input.md`: durable target-terminal policy.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; 255 tests.
- `python3 scripts/check-source.py`: passed; safe Rust, five parameters,
  nesting at most three and cyclomatic paths at most ten.
- `cargo test --test commands`: passed; twenty-six tests, zero skips. Eleven
  terminal cases cover modifier parity, exact deadlines, nested Meta, undo aliases,
  yank continuity, suffix phases, focus, cancellation, resize, guarded exit,
  simultaneous expiry/release and real per-buffer undo.
- `python3 scripts/check-pty.py --suite terminal`: passed; fifty cases,
  zero skips. New journeys exercise legacy `0x1f`, enhanced C-/ and C-_, C-x u,
  C-SPC, M-</M->, nested rectangle prefixes, unknown escape isolation, separately
  delivered Escape, idle expiry, resize/cancel, prompt ownership and guarded exit.
  Saved source bytes and terminal restoration are checked. Owned fixtures stop
  after their checks finish.
- `git diff --check`: passed.

## Risks and Follow-ups

Physical report availability and Escape grouping depend on the terminal and
Crossterm. The terminal input reference documents these limits and the reportable
C-x u alternative; the application deadline applies to delivered Escape events.
