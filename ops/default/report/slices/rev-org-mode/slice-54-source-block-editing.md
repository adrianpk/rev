# Slice 54: edit native Org source blocks and headers

Status: reviewing
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-54-source-block-editing`
PR: `#155`

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
Unknown languages and Org-looking code retain literal text behavior. Eight
presentation cases cover declared and overridden grammars, disabled syntax, the
byte cutoff, unknown/empty languages and an Org declaration using Rust syntax.
Language-specific editing widths apply without granting Org behavior.
Stale source retains the draft; read-only source permits inspection and rejects
editing. Shared-view publication and undo restore the actual originating cursor.
Queued observation, header entry, malformed metadata and draft abort leave source
unchanged and finish owned work.

## Implementation Notes

An owned, cancellable native worker observes source structure and properties.
Header precedence follows file, ancestor, affiliated and local declarations.
Repeated variable declarations remain ordered raw data, with no interpretation.
Drafts strip common indentation, normalize LF and unescape Org literal lines;
reinsertion restores source indentation/newlines and escapes delimiter-looking
content. A no-op return preserves exact bytes. Revisions, read-only state and
shared view points pass ordinary publication admission before source changes.
Publication also checks the complete updated source against the 8 MiB limit and
checks combined local headers against line, byte and effective-argument bounds.

See [source block reference](../../../../../docs/reference/org-source-blocks.md)
and the Org manual's [source editing](https://orgmode.org/manual/Editing-Source-Code.html)
and [header arguments](https://orgmode.org/manual/Using-Header-Arguments.html).

Task commits: T54.1 `5d2f540e7ab29ae11e46a20d6709898c91d9093b`, T54.2 `7367134410c3ec5efcf9547e6520ac76e33bbc53`, T54.3 `d7a898565c59edc242e186960072af23bdaf88f0`.

Canonical PR: [#155](https://forge.adrianpk.com/adrianpk/rev/pulls/155).

## Contracts Added or Changed

- ORG28: declared block metadata, special editing and checked reinsertion.
- ORG02: canonical source remains authoritative; draft failure retains text.

## Files of Interest

- `src/org/block.rs` and `src/org/block/`: bounded observations and body conversion.
- `src/session/source_edit.rs`: retained draft lifecycle and header interaction.
- `src/buffer/syntax.rs`: language presentation without a synthetic file identity.
- `tests/org_source/source_blocks.rs`: header and reinsertion contracts.
- `tests/org_editing/source_blocks.rs` and `source_edges.rs`: real draft, failure,
  shared-view and history workflows.

## Validation

The pre-PR required receipt `DZETXQ7JBKPBVVWK6HNPP7WE2B` passed on
`aef536cf17d79ead7b405755634a1b17febb79be`:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_source --test org_editing --test syntax`
- `python3 scripts/check-source.py`
- `git diff --check`

All 325 tests passed: 226 editing, 69 source and 30 syntax; zero failures or
ignored tests. The complete source checker retained safe Rust, at most five
parameters, three nesting levels and cyclomatic complexity ten. The controller
records fresh required and focused evidence after this PR-reference commit.

Focused development also verified all eight presentation cases, exact-limit
publication, aggregate overflow rejection, palette registration and configuration
references. An initial legacy catalog assertion was reconciled to the delivered
commands; its remaining future-command assertion stays active.

## Risks and Follow-ups

Observation caps native source at 8 MiB, each header at 4096 bytes, combined
headers/affiliations at 64 KiB, effective arguments at 256 and nesting at 64.
Names and languages cap at 128 bytes. Header interpretation by later consumers
retains its own admission responsibility.
