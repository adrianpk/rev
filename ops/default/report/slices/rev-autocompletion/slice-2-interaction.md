# Slice 2: Cursor popup and automatic invocation

Status: reviewing
Delivery set: rev-autocompletion
Plan: [Native Autocompletion Delivery](../../../plan/rev-autocompletion.md)
Tracker: [Native Autocompletion Tracker](../../../tracker/rev-autocompletion.md)
Branch: `feat/rev-autocompletion-02-interaction`
PR: [#196](https://forge.adrianpk.com/adrianpk/rev/pulls/196)

## Purpose

Deliver CMP01 and CMP04-CMP06 automatic/manual interaction, cancellation,
finite preferences and current cursor-popup presentation.

## Delivered Behavior

Eligible Rust identifier edits trigger completion after a 150 ms debounce and
two Unicode identifier characters by default. Dot and double-colon separators
bypass the minimum; manual invocation bypasses both delay and minimum.
Comments, strings and character literals suppress automatic lists by default,
with independent startup controls. Manual requests retain declaration coverage
there. Typed/deleted prefixes schedule the newest context; paste cancels and
does not enqueue per-character requests. Cancellation stays quiet until another
eligible edit or explicit request.

The shared popup preselects the first ranked candidate, uses a single border
and full-row selection, fits above/below the cursor and preserves source focus.
Resize uses current pane geometry; narrow panes can hide the overlay while
cancellation remains available. Unrelated navigation, prompts and pane/buffer
changes dismiss requests before ordinary dispatch. Tab/Return and Org behavior
retain their ordinary owners outside an active list. A configurable local map
admits popup navigation, acceptance and cancellation, including chord sequences.

Smart-case prefix/subsequence ranking and literal one-transaction acceptance
remain shared with manual completion. `completion-mode` and
`completion-auto-mode` change only session preferences. Source text and startup
files are never changed by preselection, cancellation or these controls.

## Implementation Notes

One temporary worker borrows the existing retained Rust parser/tree and a shared
rope snapshot. It checks cancellation and a 50 ms parse/query deadline. Cancelled
work retains the slot until reaped; one newest pending context replaces earlier
requests. Snapshot admission remains 2 MiB and extraction remains bounded to
4096 symbols and 512-byte labels. Candidate and popup-row preferences have
finite admitted ranges. No second parser/index owner or permanent worker exists.

The main syntax slot cannot reparse while borrowed. Returned syntax metadata
retains its actual source revision; canonical edits continue independently and
obsolete proposals cannot publish. Request and acceptance checks pin view,
point, source revision, syntax-admission generation and invocation identity.
Worker failure retires its ownership without editing source. The event loop
waits on debounce deadlines and polls only owned work; idle completion has no
wakeup. Rendering reads retained candidates and does no completion extraction.

## Contracts Added or Changed

Eight startup `completion` leaves expose enabled/automatic behavior, delay,
minimum prefix, comment/string policy, candidate cap and popup rows. Their
defaults, types and ranges are complete in the reference/sample. Two implemented
runtime commands and the `keys.completion` context enter the compiled catalogs.
C-g keeps its globally reserved `keyboard-quit` configuration value and cancels
the active list; Escape is literal cancellation there and retains normal Meta
timing elsewhere. Complete configuration, command/binding references and
Unreleased describe the delivered behavior.

## Files of Interest

- `src/completion/job.rs`: finite ownership, cancellation and newest pending work.
- `src/session/completion.rs`: debounce, controls, admission and publication.
- `src/syntax.rs`, `src/buffer/syntax.rs`: retained syntax transfer/generations.
- `src/syntax/completion.rs`: bounded queries and Rust literal-context policy.
- `src/config/completion.rs`: complete/partial startup values and bounds.
- `src/input/completion.rs`, `src/input/keymap.rs`: popup-local dispatch.
- `tests/completion/automatic.rs`: real automatic input, context and policy cases.
- `docs/reference/completion.md`, `config.toml`, `defaults.md`: actual public behavior.

## Validation

Development checks passed:

- `cargo test --locked --test completion`: all 16 interaction cases passed.
- `cargo test --locked --test completion --test configuration`: 15 completion
  cases and all 136 configuration cases passed before the additional matching/
  layering case was added.
- `cargo test --locked --lib session::completion`: three request-lifetime,
  read-only and real worker-replacement cases passed before adding the syntax
  generation regression.
- `cargo test --locked --lib session::completion::tests::worker_replacement`:
  the corrected pending-slot case passed.

Coverage includes debounce timing, idle shutdown, newest pending edits, active
worker cancellation, Unicode minimums, separators, smart case, no matches,
comment/string policy, paste, cancellation suppression, configurable keys and
prefixes, startup layering/limits, runtime controls, literal edit/undo, pane
placement, wrapping/resize, stale views/revisions and read-only admission.
An initial custom-prefix fixture used C-j, which aliases Return; the existing
keymap correctly rejected its command/prefix conflict. The fixture now uses
the distinct admitted C-c prefix and retains its acceptance assertion.

The first controller-focused batch stopped in the worker-replacement test:
manual invocation can eagerly reap an already finished worker before that test
observes the pending slot. The fixture now queues before polling, which pins
the actual single-slot boundary independently of thread timing. Its pending,
newer-generation, idle-cleanup and source/history assertions remain intact;
the work deadline and resource limits are unchanged.

T2.1 is commit `14984e2853774740cef539de872075346c5a8156`.
T2.2 is commit `85fe583f59ed28075bc4a9d95fd151221f7870cb`.
Focused controller receipt `ODRI6E6D3OQNDJKQOK33KAWRAZ` passed at clean candidate
`4f6601a45dc0179a6927b61521a5104ef847bf54`:

1. `cargo test --locked --lib completion`: 7 tests passed.
2. `cargo test --locked --test syntax`: 30 tests passed.
3. `cargo run --locked --quiet --example source-check -- --file src/completion.rs --file src/completion/job.rs --file src/session/completion.rs --file src/syntax.rs --file src/syntax/completion.rs --file src/buffer/syntax.rs --file src/input/completion.rs --file src/config/completion.rs --file src/display/native_completion.rs`: passed.
4. `cargo run --locked --quiet --example documentation-check`: 326 commands,
   198 configuration entries, 60 reachable pages and 346 local links/anchors passed.
5. `git diff --check`: passed.

After the PR-reference follow-up, the controller records a fresh receipt for the
final clean PR head and its exact configured ordered list:

1. `cargo fmt --all -- --check`
2. `cargo test --locked --test completion --test commands --test configuration --test display --test panels --test wrapping`
3. `git diff --check`

The configured PR prefix includes the final diff command, so additional focused
evidence stays in this report instead of changing the canonical PR list.

## Risks and Follow-ups

Current native extraction remains Rust/current-buffer declaration observation.
Broader languages, scope coverage, other open buffers and the integrated
resource/PTY acceptance remain with their approved slices. The full delivery-set
gate remains pending after all slices integrate.
