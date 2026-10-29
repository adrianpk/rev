# Slice 54: edit native Org source blocks and headers

Status: drafting
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-54-source-block-editing`
PR: pending

## Purpose

Deliver native source block editing under the
[approved plan](../../../plan/rev-org-mode.md#slice-54-edit-native-org-source-blocks-and-headers)
and [tracker](../../../tracker/rev-org-mode.md#slice-54-edit-native-org-source-blocks-and-headers).

## Delivered Behavior

Source block observations retain declared language, names, switches and inherited
raw header data without execution. C-c ' opens an isolated source draft; return
publishes one checked canonical body change and abort restores the source view.
Header inspection is read-only; inserting one local argument uses source undo.
Draft presentation reuses the configured language and bundled Rust grammar.
Unknown languages and Org-looking code retain literal text behavior.

## Implementation Notes

An owned, cancellable native worker observes source structure and properties.
Header precedence follows file, ancestor, affiliated and local declarations.
Repeated variable declarations remain ordered raw data, with no interpretation.
Drafts strip common indentation, normalize LF and unescape Org literal lines;
reinsertion restores source indentation/newlines and escapes delimiter-looking
content. A no-op return preserves exact bytes. Revisions, read-only state and
shared view points pass ordinary publication admission before source changes.

See [source block reference](../../../../../docs/reference/org-source-blocks.md)
and the Org manual's [source editing](https://orgmode.org/manual/Editing-Source-Code.html)
and [header arguments](https://orgmode.org/manual/Using-Header-Arguments.html).

## Contracts Added or Changed

- ORG28: declared block metadata, special editing and checked reinsertion.
- ORG02: canonical source remains authoritative; draft failure retains text.

## Files of Interest

- `src/org/block.rs` and `src/org/block/`: bounded observations and body conversion.
- `src/session/source_edit.rs`: retained draft lifecycle and header interaction.
- `src/buffer/syntax.rs`: language presentation without a synthetic file identity.
- `tests/org_source/source_blocks.rs`: header and reinsertion contracts.
- `tests/org_editing/source_blocks.rs`: real editor draft and history workflows.

## Validation

Development check `cargo test --test org_source source_blocks` passed four tests.
Development check `cargo test --test org_editing source_blocks` passed three
session tests. Changed-source quality passed. Required exact-HEAD checks remain pending.

## Risks and Follow-ups

Observation caps native source at 8 MiB, each header at 4096 bytes, combined
headers/affiliations at 64 KiB, effective arguments at 256 and nesting at 64.
Names and languages cap at 128 bytes. Header interpretation by later consumers
retains its own admission responsibility.
