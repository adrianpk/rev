# Slice 4: Live Terminal Interaction

Status: reviewing
Delivery set: rev-integrated-terminal
Plan: [Integrated Terminal Delivery](../../../plan/rev-integrated-terminal.md)
Tracker: [Integrated Terminal Tracker](../../../tracker/rev-integrated-terminal.md)
Branch: `feat/rev-integrated-terminal-04-interaction`
PR: `#251`

## Purpose

Route native input and present interactive child tools through the existing
terminal emulator, host context and cell renderer.

## Delivered Behavior

Live terminal keys bypass source and modal editing in both profiles. Return,
Tab, deletion, arrows, Home/End, paging and F1-F20 use classic VT encodings,
including modified movement, child application-cursor and newline modes.
Reported keypad events respect application-keypad mode. C-x retains its host
prefix; C-c and modal C-w reach the child. M-x opens native commands and C-g
cancels a stronger host prompt or prefix before returning to the editor.
C-q and terminal-send-next-key forward exactly one normalized key literally,
including protected host keys; releases do not consume that request.

Child cursor shape and blink state travel as native frame metadata. Host
prompts and source editing retain their own presentation; terminal restoration
continues through the existing hosting-terminal owner.

The native renderer resolves child palette changes into cell RGB values. A
controlled full-screen child exercises alternate-screen entry/return, six
cursor shapes, hidden cursor, styles, resize and output while source focus
remains selected. Unsupported effects and clipboard controls never reach the
host output.

Child mouse reporting uses its requested classic, UTF-8 or SGR coordinate
protocol. Shift bypasses reporting for local selection and bounded scrolling;
mirrored attachments retain independent state. Pointer capture clips releases
outside the original terminal and preserves source-owned drags. Control-click
validates an OSC 8 URI and starts one bounded configured-viewer operation with
literal argv. Unsolicited output never launches that viewer.

## Implementation Notes

The keyboard adapter emits bounded complete sequences to the existing atomic
input queue. Extended keyboard negotiation remains disabled. A single armed
literal key is scoped to the current session and cleared on source return.
Dependencies and terminal resource limits remain unchanged. Each attachment
retains two selection endpoints without a text copy; one editor-wide capture
retains only session/view identities. Viewer assembly reuses the configured
viewer owner and its existing limits. A completion wake and one bounded result
slot keep process IO and waits off the UI without an idle polling timer. The
noninteractive shutdown barrier also waits for owned viewer cancellation.
Native all-motion tracking is reversible and acquired only for a focused child
requesting it; source contexts retain their ordinary button tracking.

## Contracts Added or Changed

The native command catalog admits terminal-send-next-key with a focused
terminal requirement. Child controls cannot accidentally start unrelated host
prefixes. A frame exposes optional native terminal cursor style metadata.

## Files of Interest

- `src/session/terminals/keyboard.rs`: classic VT keyboard encoding.
- `src/session/terminals/input.rs`: host priority and literal-next routing.
- `src/display/terminal.rs`: native cursor, palette and local selection presentation.
- `src/session/terminals/pointer.rs`: checked pointer focus and capture.
- `src/session/terminals/link.rs`: URI admission and owned viewer completion.
- `tests/integrated_terminal/interaction.rs`: exact real-PTY input checks.

Task commits: T4.1 `5f3ae4f4d345`, with cancellation correction
`9671a9d28f0d` and bounded dispatch `580bd8538d2a`; T4.2 `7da3f48dff23`;
T4.3 `a819a54562a6`, with fixture correction `ef72e033fb2f`.

## Validation

Focused attempt `NA76K3A36DM2CK62VG3QC4QL2Q` exposed delayed Escape
cancellation returning from the child as well as closing the host prompt. The
expiration path now preserves the prior transient owner through dispatch.
The existing focus assertion remains unchanged.

Attempt `FLGGPRJOJ4AK2ERS2PCMBNIPFB` passed the raw-PTY cases after that
correction, then found two command-dispatch complexity violations. Input and
placement dispatch now delegate directory entry to its existing small owner.

Focused receipt `NZ6I2BQ7YVM7IJBMWFHBBJYCIE` passed exact input, six source
files and whitespace on `580bd8538d2a7b566d37d07e54803f0d7f266d59`.

Focused receipt `SCGY7G3XHNXBZ74CDIPCGGQK5Q` passed on
`7da3f48dff23ff0470fc0fd1d97b359d4215c9d0`:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test integrated_terminal panel::interaction:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example source-check -- --file src/display/terminal.rs`
- `git diff --check`

T4.3 attempt `P6OXNBLDEWSYLQ5SKRYDCU2UJL` passed child mouse and local
mirror cases, then found two fixture admission/producer errors. The legacy
coordinate case now requests the admitted zero-scrollback preference and
asserts its 300-column geometry within the unchanged cell limit. The link
producer uses `printf '%b'` with a separate payload so percent-encoded URI data
stays literal; its emulator URI must equal the original input before activation.
No protocol expectation, resource assertion or frozen command changes.

Both earlier interaction scenarios passed. The full-screen case uses a controlled shell
fixture; installed-tool journeys remain under the recorded acceptance slice.
T4.3 receipt `2MF3LO7YJWYNHK5YRRN7T5RZK7` passed on
`ef72e033fb2ffea99d62280631cb546c83154c4a`: four real-PTY pointer/link
scenarios, six host mouse restoration cases, five existing configured-viewer
regressions, seven source files and whitespace. These focused checks are distinct
from the final six ordered Slice 4 commands and the controller-owned set gate.
Final exact-head evidence follows the PR-reference commit; its receipt remains
in the controller and Acta so reporting does not mutate the validated candidate.

## Risks and Follow-ups

Stable retained copy projections and paste transfer remain
in their recorded subsequent slice.
