# rev Editor: Core and terminal

Plan: [rev Editor](../rev-editor.md)
Specification: [rev Editor](../../spec/rev-editor.md)
Model: [rev Editor Model](../../spec/rev-editor-model.md)

Read the master plan, its pinned decisions, and the current section before work.
This file supplies detailed tasks; the master plan/tracker own slice identities.
Every task includes appropriate behavior tests. Each slice also requires the
common quality gate, one canonical report, one PR, and controller-owned merge.

## Slice 1: Own terminal lifetime

Requirements: R01, R02, R03, R34, R35, R38
Dependencies: Approved bootstrap on dev
Owners: entrypoint, terminal guard, quality scripts
Focused validation: `python3 scripts/check-pty.py --suite terminal`

### T1.1

Create the Rust 2024 executable/library package, pin the approved dependency set, forbid
unsafe at every first-party crate root, and enter/restore the terminal through one RAII
owner. Implement the real exit sequence without placeholder editor commands.

### T1.2

Provide executable source-quality and per-slice/full-gate entry points. Add a minimal
PTY lifecycle probe for startup, ordinary exit, and a failure after terminal
acquisition; later suites must be required by the final gate rather than silently
skipped.

### Required evidence

Observe raw-mode and alternate-screen restoration through a real PTY; partial
acquisition cleanup runs only acquired obligations. Cargo and source checks pass on the
first code candidate.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-1-terminal.md`. Update the tracker after integration.

## Slice 2: Introduce canonical rope buffers

Requirements: R08, R09, R11, R33
Dependencies: Slice 1
Owners: buffer registry and rope owner
Focused validation: `cargo test --test core`

### T2.1

Add stable buffer identity, one canonical Ropey store, initial empty state, checked
scalar ranges, and bounded chunk/line reads. Keep file identity and view identity
separate.

### T2.2

Exercise insertion-range admission, empty/multiline stores, foreign identities, stale
revisions, UTF-8 boundaries, and two references observing one text owner without a
second mutable copy.

### Required evidence

Reject invalid requests before text or revision changes; reading a visible fragment does
not flatten the unaffected document.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-2-buffers.md`. Update the tracker after integration.

## Slice 3: Resolve Unicode and display coordinates

Requirements: R07, R09, R10, R16
Dependencies: Slice 2
Owners: coordinate conversion and display policy
Focused validation: `cargo test --test coordinates`

### T3.1

Implement explicit scalar/byte/line-byte/display conversion and grapheme navigation over
rope chunks. Use Unicode segmentation context across seams; retain CRLF as one logical
separator.

### T3.2

Add behavior cases for emoji sequences, combining marks, CJK width, tab stops, empty
text, and a grapheme straddling chunks. Define safe display of controls without emitting
terminal escape bytes.

### Required evidence

Round trips identify the same source boundary; byte, scalar, grapheme, and cell counts
differ where expected. No movement divides an encoded scalar or supported cluster.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-3-coordinates.md`. Update the tracker after integration.

## Slice 4: Publish coherent text edits

Requirements: R08, R09, R17, R21, R33
Dependencies: Slice 2, Slice 3
Owners: buffer edit and reversible transaction
Focused validation: `cargo test --test editing`

### T4.1

Introduce one checked edit path with read-only admission, a prepared replacement,
reversible history data, and exact position mapping. Multi-range edits state their
coordinate space before publication.

### T4.2

Reject stale/read-only/invalid-range edits without partial publication. Establish atomic
multi-range rollback and mapped point/mark behavior so later commands cannot bypass the
owner.

### Required evidence

A recoverable preparation failure preserves text, history, and positions; a successful
transaction updates each once. No independent mutable document copy becomes
authoritative.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-4-edits.md`. Update the tracker after integration.

## Slice 5: Decode prefixes and focused prompts

Requirements: R22, R23, R24, R25, R33
Dependencies: Slice 1, Slice 4
Owners: input state, static command enum, argument prompt
Focused validation: `cargo test --test commands`

### T5.1

Normalize keyboard events, route one focused context, decode nested chord prefixes, and
dispatch only implemented command variants. Provide a small editable argument/decision
prompt used by subsequent file and safety flows.

### T5.2

Cover invalid prefixes, C-g cancellation, prompt/document isolation, press versus
release, and exactly-once dispatch. Advertise no unavailable command or
functional-looking stub.

### Required evidence

C-x r followed by cancellation cannot become inserted text; prompt keystrokes do not
mutate the underlying buffer and ordinary input resumes afterward.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-5-input.md`. Update the tracker after integration.

## Slice 6: Render one rope-backed viewport

Requirements: R05, R06, R07, R08
Dependencies: Slice 3, Slice 5
Owners: view, cell projection, drawing
Focused validation: `cargo test --test display`

### T6.1

Display the initial rope-backed view, visible fragments, cursor, mode line, and echo
area with clipped cell geometry and deliberate flushing. Track point and
horizontal/vertical viewport state independently of text.

### T6.2

Validate mixed-width clipping, tabs, zero dimensions, status dirty/read-only/mode
indicators, and readable error messages. Keep projection caches derived and correctly
invalidated.

### Required evidence

Every draw remains within its assigned cells, controls cannot inject terminal output,
and resizing presentation does not rewrite the rope.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-6-display.md`. Update the tracker after integration.

## Slice 7: Edit printable text and graphemes

Requirements: R02, R04, R08, R17, R25
Dependencies: Slice 4, Slice 5, Slice 6
Owners: editing commands and active view
Focused validation: `cargo test --test editing`

### T7.1

Connect printable input, Enter, Tab, Backspace, and delete-char to the common edit owner
and visible cursor updates. Preserve the chosen newline policy and coherent CRLF
deletion.

### T7.2

Add boundary, Unicode, and read-only behavior checks through command input. Guard
modified text immediately: until saving exists, refuse an unsafe quit rather than offer
a fake save action.

### Required evidence

Direct text appears without a mode transition, backspace cannot cross buffer start, and
unknown modified keys cannot write control bytes.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-7-insertion.md`. Update the tracker after integration.

## Slice 8: Implement character and word motion

Requirements: R07, R09, R16, R17
Dependencies: Slice 3, Slice 6, Slice 7
Owners: navigation commands
Focused validation: `cargo test --test coordinates`

### T8.1

Implement forward/backward character, beginning/end of line and buffer, and
Unicode-consistent word navigation using checked source positions.

### T8.2

Exercise empty text, punctuation, mixed scripts, CRLF, emoji clusters, and scroll reveal
through the real command registry. Arrow/Home/End aliases may mirror supported commands
without replacing Emacs bindings.

### Required evidence

Motion never mutates text, clamps safely at document boundaries, and reveals the
selected point at a valid cell boundary.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-8-motion.md`. Update the tracker after integration.
