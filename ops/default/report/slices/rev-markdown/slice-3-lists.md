# Slice 3: List presentation and editing

Status: delivered
Delivery set: rev-markdown
Plan: [Markdown Delivery](../../../plan/rev-markdown.md)
Tracker: [Markdown Tracker](../../../tracker/rev-markdown.md)
Branch: `feat/rev-markdown-03-lists`
PR: [#36](https://forge.adrianpk.com/rev/rev/pulls/36)

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

Implementation and report introduction: `4251b929e6b4493a59a0e7146ffb82a5e1c7b38b`.
Canonical PR: https://forge.adrianpk.com/rev/rev/pulls/36
Development evidence commit: `83aa5658824d38e8e0650ed11a4b74e1e4fa8c25`.

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

All five required development commands passed in canonical order on clean
implementation HEAD `4251b929e6b4493a59a0e7146ffb82a5e1c7b38b`, from
2026-11-13T18:42:28Z to 2026-11-13T19:01:07Z. All 542 tests across six native
targets passed without failures or ignored cases. Full lint, exact command and
configuration documentation and whitespace checks passed. Controller receipt `O7DFCKIXNY2ZQQTIKXKH7L3PGQ` passed all five ordered
checks on clean published HEAD `d6d65ac09aa124b7155754bda6db1bf158e68349`,
from 2026-11-13T19:03:08Z to 2026-11-13T19:21:13Z. All 542 native tests
passed without failures or ignored cases; the native targets took 1,069.42 s.
Canonical PR #36 merged into dev at that exact commit, verified against both
dev refs and its Acta integration record. Slice 3 of 6 completed. The controller
owns the remaining full delivery-set gate.

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test markdown --test configuration --test commands --test display --test pointer -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test modal contexts:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
- `git diff --check`

| Target | Passed | Seconds |
| --- | --- | --- |
| commands | 205 | 624.36 |
| configuration | 169 | 268.93 |
| display | 47 | 35.12 |
| markdown | 31 | 67.01 |
| pointer | 69 | 70.93 |
| modal contexts | 21 | 26.56 |

The native targets took 1,092.91 s in total. Modal filtering retained the exact
approved contexts:: selection. This measured cost changes no command, timeout or
acceptance condition. Full workspace lint and the installed immutable-index hook
pass all 1,867 first-party Rust files, both packages and 74 Cargo roots, with
zero blocking or deferred diagnostics.

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

Physical glyph coverage remains terminal-owned. Slice 6 delivers joined
native/PTY acceptance and the User Guide; Slices 4-5 deliver table editing.
The existing foreign crossterm unused_parens warning is outside first-party
source policy.

## Delivery-set closure

The unchanged controller-owned full gate passed on 2026-11-14 at clean published
candidate `4a01a05ee0ee02b0ec02aed1beac0cdc19c59664`, receipt
`X5PEQCHM7HGP4DHHSW4ZJ7I53J`. All six canonical PRs #34-#39 are merged;
this report remains delivered. The
[Slice 6 closure](slice-6-acceptance.md#delivery-set-closure) records the exact
command, 721 selected native tests, joined native/PTY acceptance, documentation,
measured costs and evidence limits. Specification, model, plan and tracker are
Completed. Only the configured closure Markdown paths differ from the tested
candidate; all tested implementation and guide inputs retain their bytes.
