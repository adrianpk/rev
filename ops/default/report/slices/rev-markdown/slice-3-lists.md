# Slice 3: List presentation and editing

Status: drafting
Delivery set: rev-markdown
Plan: [Markdown Delivery](../../../plan/rev-markdown.md)
Tracker: [Markdown Tracker](../../../tracker/rev-markdown.md)
Branch: `feat/rev-markdown-03-lists`
PR: pending

## Purpose

Deliver T3.1-T3.3: native list symbols, checked source actions and the Markdown
editing overlay in both supported profiles. Table actions retain Slices 4-5.

## Delivered Behavior

- Unordered markers repeat the approved •, ◦ and ▪ cycle by parser-admitted
  logical depth, including mixed ordered/unordered lists and quote containers.
  Ordered source numbers, indentation, literal blocks and source width remain
  intact. The existing view-local pretty flag controls headings and lists.
- Continue-item inserts a sibling after the complete current subtree, preserving
  bullet or ordered delimiter/padding and incrementing ordered source numbers.
  An empty item exits one level. Invalid moves and exhausted nine-digit numbers
  refuse without partial edits.
- Indent moves an item below its previous sibling. Outdent places its complete
  child/continuation subtree after the parent's remaining children. LF/CRLF and
  final-newline identity survive; one undo restores source and canonical points.
- Return and Meta-Return dispatch list continuation or ordinary newline.
  Meta-Right/Meta-Left indent/outdent. keys.markdown inherits the effective
  editor/profile routes; explicit parent remaps beat factory Markdown defaults,
  and explicit local routes take priority. Modal Normal/Visual and popup owners
  retain input authority.
- All five list action names appear in both native selectors. Read-only, stale,
  canceled, over-limit and literal-block contexts cannot authorize list edits.

## Implementation Notes

Implementation and report introduction: pending.
Canonical PR: pending.

The private parser adapter retains editor-owned item source spans, logical
parent/list identity, source marker form and continuation indentation alongside
the existing revision-qualified observation. Inline literals remain inside a
list context; literal blocks refuse list actions. No parser type escapes the
adapter, dependency changes or per-frame parsing are introduced.

Source actions prepare owned replacements and retained-source runs, then use
the existing checked buffer/window publication owner. Moving a subtree retains
body spans so shared points follow their source. Ambiguous or missing structural
evidence never falls back to an inferred list edit. Ordinary newline remains
available through contextual Return when structural evidence is unavailable.

The Markdown map is an editing overlay with its own configuration provenance.
It activates in non-modal editing and modal Insert. Effective resolution also
feeds selector hints, macro keys and existing plugin priority checks without
adding an SDK/wire context or a session persistence record.

## Contracts Added or Changed

- keys.markdown joins the complete configuration sample/catalog and normal
  leaf merge, terminal alias normalization and whole-candidate rejection.
- markdown-continue-item, markdown-indent-item, markdown-outdent-item,
  markdown-return and markdown-meta-return join the exact native command catalog.
- markdown-pretty-mode now describes both heading and list symbols. Existing
  palette leaves, source geometry and presentation reset semantics remain valid.
- Source-work byte/event/depth/cancellation bounds and dependency graph remain
  unchanged. No delivery-set gate, installation or release is part of this slice.

## Files of Interest

- src/markdown/item.rs, src/markdown/parse.rs and src/markdown/lists/: admitted
  list ownership, checked continuation, subtree layout and retained source.
- src/buffer/markdown.rs, src/display/markdown.rs and src/session/markdown.rs:
  current observations, width-preserving symbols and canonical publication.
- src/input/markdown.rs, src/input/keymap/ and src/input/plugin_keymaps.rs:
  profile-local dispatch, override priority, discoverability and native ownership.
- tests/markdown/lists.rs, tests/markdown/list_controls.rs and configuration
  Markdown cases: source, transaction, modal and candidate regressions.
- src/input/markdown/tests.rs: effective hints and completion-popup precedence.
- docs/reference/ bindings, commands, defaults, syntax, complete config sample,
  exact command description specification and CHANGELOG.md.

## Validation

Required development checks are pending. Focused source/profile cases, exact
command descriptions and documentation pass. Final evidence must be recorded against the
clean published PR HEAD after all PR-reference commits through the controller's
validate command. The controller owns integration and the full delivery-set gate.

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test markdown --test configuration --test commands --test display --test pointer -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test modal contexts:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
- `git diff --check`

Focused development established list continuation/exits, subtree moves, actual
profile routes, read-only/cancellation refusal and inherited/local override
priority. The complete Markdown target passed all 31 cases in 64.16 s; the final
six list cases, extended with interleaved quote/list containers, passed in
13.78 s. All five configuration Markdown cases and the exact command description
guard pass. The focused private input check passed effective Return hints and
completion acceptance in both profiles:
`env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib input::markdown::tests:: -- --test-threads=1`.
Documentation passes 698 commands, 322 settings, 106 reachable pages and 753
local links/anchors.

The first complete Markdown run passed 28 of 29 cases; its sole failure
was the previous heading-in-list fixture expecting a literal bullet, now updated
to the approved list symbol. Initial continuation fixtures were corrected to
target a parser-admitted empty child at EOF; an empty marker cannot interrupt
the parent's paragraph. A test API spelling and checked coordinate API usage
were corrected without changing any bound, timeout or acceptance requirement.
The initial configuration rejection fixture named an available editing command;
it now uses an unavailable prompt action under the unchanged admission contract.
The documentation check caught the new map's missing exact sample count; the
catalog now records all four local entries in its existing table format.
Quote-prefix regressions retain quote containers across continuation/indent/
outdent, including an ordered ancestor whose indentation exceeds three spaces.

## Risks and Follow-ups

Physical glyph coverage remains terminal-owned. Full PTY acceptance and User
Guide delivery remain Slice 6; table editing remains Slices 4-5. The existing
foreign crossterm unused_parens warning is outside first-party source policy.
