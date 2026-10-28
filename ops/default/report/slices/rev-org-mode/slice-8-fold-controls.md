# Slice 8: Deliver native Org visibility controls

Status: reviewing
Delivery set: rev-org-mode
Plan: [Native Org Delivery](../../../plan/rev-org-mode.md)
Tracker: [Native Org Tracker](../../../tracker/rev-org-mode.md)
Branch: `feat/rev-org-08-fold-controls`
PR: [#109](https://forge.adrianpk.com/adrianpk/rev/pulls/109)

## Purpose

Expose native outline and drawer visibility through shared key and named handlers.

## Delivered Behavior

TAB cycles the heading at point through heading-only, direct child headings and
complete subtree. On an admitted drawer it toggles only that drawer's body.
S-TAB cycles this view through overview, contents and show-all. Eleven compiled
commands expose those cycles and direct heading/drawer choices. Initial source
visibility is show-all. Plain, capture and prompt TAB meanings remain distinct.

Native structural observations exclude literal block text from outline targets.
A missing exact TAB target reports unavailability without folding an ancestor or
editing source. Drawer boundaries remain visible. Explicit folding of a point's
body moves that view to its checked visible owner; no display row aliases source.

The owned outline terminal journey exercises native keys, M-x, independent panes,
targeted search and cancellation in RGB and NO_COLOR. It saves the unchanged
BOM/CRLF/unterminated Unicode source, verifies fixture identity, and reaps its
child after terminal restoration with both PTY descriptors closed.

## Implementation Notes

The existing single temporary observation worker builds a stamped fold model
from structural observations. At most 4096 heading/drawer records participate;
incomplete observations reject visibility operations without changing source.
The command path does not parse source. A single pending request captures its
view and source and is admitted only while they remain current. C-g cancels that
intent and owned work, including requests made with syntax coloring disabled.

View choices retain immutable shared model data and independent requested states.
State and interval projection remain view metadata, outside canonical text/history.

Task commits: T8.1 `f989a86eddfde326e52544d7b4b2c3224b363078`,
T8.2 `8e799f9335e6bcda550e91a898f6973049273ced`,
T8.3 `23f5d8fb5e71164e3707e8afec5517bc8136e760`.

Targeted motion and search reveal only the ancestor path and target entry;
sibling folds and peer views keep their requested choices. Search cancellation
restores the saved complete view state, and resize preserves live choices.
Visible-row movement skips folded source. Native edits/undo/redo retire obsolete
choices instead of requalifying stale byte offsets. Read-only controls remain
usable. Requests whose source, view or prompt owner changes cannot publish late.

## Contracts Added or Changed

- ORG03: local/global cycles and exact drawer visibility through native views.
- ORG17: compiled M-x identities, context keys, complete sample and catalog.

## Files of Interest

- `src/command/org.rs`, `src/input/keymap/defaults.rs`: compiled identities and keys.
- `src/org/visibility/`: bounded structural fold model and state transitions.
- `src/view/visibility.rs`: checked view-only publication.
- `src/session/visibility.rs`, `src/session/org.rs`: request and observation lifetime.
- `tests/org_editing/visibility.rs`, `tests/commands/org_visibility.rs`: native controls.
- `tests/org_editing/visibility_reveal.rs`: hierarchy, saved state, read-only,
  named equivalence, incomplete observations and retired-request behavior.
- `scripts/configuration_ui/org_outline.py`: owned native outline journey.
- `docs/reference/org-mode.md`, `config.toml`, `commands.md`, `bindings.md`:
  active controls and reference catalog.

## Validation

Implementation checks passed:

- `cargo check --lib`
- `cargo test --test org_editing visibility::`: four passed.
- `cargo test --test commands org_visibility::`: one passed.
- `cargo test --test org_editing visibility_reveal::`: six passed.
- `cargo test --test commands catalog_reference`
- `python3 scripts/check-config-reference.py`
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-outline`:
  two owned sessions passed, zero skipped.

Focused source-quality checks passed for changed visibility owners.
The first focused controller batch (`7XPCH6ZFWUABSIEYVNBH7WZWLX`) failed
`cargo test --test configuration`: the complete-default catalog still omitted
the new Org overlay. The catalog now composes normalized inherited/local entries;
the existing complete-map assertion passes unchanged in focused reproduction.
The terminal driver's show-all predicate was corrected to the handler's actual
`Org show-all` echo. Expected source rows, byte fidelity and cleanup assertions
were preserved.

Required exact-head controller validation follows the pushed PR-reference
follow-up, in this order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_editing --test commands --test display`
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-outline`
- `python3 scripts/check-source.py`
- `git diff --check`

Focused configuration tests and reference checks are recorded separately at the
same candidate. Exact-head receipts are retained by the controller; integration
evidence will close this report after the canonical merge is verified.

## Risks and Follow-ups

Existing edits release obsolete view choices. Incomplete structural observations
reject folding; show-all and editing remain available.
