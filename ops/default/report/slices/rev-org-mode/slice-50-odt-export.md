# Slice 50: export native Org to OpenDocument text

Status: reviewing
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-50-odt-export`
PR: `#151`

## Purpose

Deliver native ODT packages under the
[approved plan](../../../plan/rev-org-mode.md#slice-50-export-native-org-to-opendocument-text)
and [tracker](../../../tracker/rev-org-mode.md#slice-50-export-native-org-to-opendocument-text).

## Delivered Behavior

T50.1 packages immutable snapshots as bounded OpenDocument text archives.
The first stored ZIP member contains the exact media type without extra fields;
content, style, metadata and manifest XML members have explicit relationships.
Native CRC32, ZIP32 sizes and fixed headers produce deterministic archives without
an external archiver or new Cargo dependency. XML-invalid scalars reject rather
than becoming substituted or malformed data.

T50.2 adds `org-export-to-odt` and preserves native headings, Unicode, list
markers, table cells, emphasis, captions, targets, links and named/inline notes.
Styles, bookmark links, note references and manifest members have checked local
relationships. Unsupported assets, rich forms and entity/math spelling remain
readable literal data with diagnostics. Binary publication shares the existing
checked file owner; successful export retains source format, revision, view and
undo history without acknowledging a source save.

T50.3 checks independent ZIP/CRC/XML semantics and exact source preservation,
including the final archive byte boundary, invalid scalars, unsupported rich
content, missing/cyclic notes, duplicate targets, ragged cells, stale inputs and
replacement observations, aliases, cancellation and read-only subtree selection.
The owned RGB/NO_COLOR journey now parses actual ODT output as well as the other
native formats. Unsupported math initially lost native emphasis delimiters;
shared bounded delimiter recognition now retains the complete ODT math spelling
with diagnostics while TeX keeps its existing admission limit.

## Implementation Notes

`Snapshot::package` returns binary archive bytes separately from text rendering.
XML emission, CRC and ZIP assembly share cancellation, operation deadline and the
snapshot output limit, including archive headers and directory records.
Preparation has no publication, asset-reading, process or network authority.
The package profile follows the OASIS OpenDocument 1.3 package specification.

Task commits: T50.1 `9e11ce19ed890e1ff96839478a96916d78fa1545`, T50.2 `5cabdf4461e1f4edf618cbc73c273d1ad2a2d59b`, T50.3 `36e5da2623d75404c54ff7bca9183068aa4c8170`.

Canonical PR: [#151](https://forge.adrianpk.com/adrianpk/rev/pulls/151).

## Contracts Added or Changed

- ORG25: native OpenDocument text export from checked current source.
- ORG18: bounded package assembly retains ordinary editor guarantees.

## Files of Interest

- `src/org/export/odt.rs`: native package and metadata assembly.
- `src/org/export/odt/zip.rs`: fixed stored ZIP profile and CRC32.
- `src/org/export/odt/xml.rs`: bounded XML text and whitespace admission.
- `tests/org_io/odt_exports.rs`: independent ZIP/XML expectations.

## Validation

- `cargo test --test org_io odt_`: nine package, semantic, selection and publication workflows passed, zero failures or ignored tests.
- `cargo test --test org_io tex_export`: five shared-delimiter regression workflows passed, zero failures or ignored tests.
- `cargo clippy --lib --test org_io --test commands -- -D warnings`: passed.
- `xmllint --noout --relaxng OpenDocument-v1.3-schema.rng content.xml styles.xml meta.xml`: the representative generated XML parts passed the published ODF 1.3 Relax NG schema in an owned temporary validation directory. No office-application GUI was run.

- `cargo test --test org_io odt_`: four package, native-document and publication workflows passed, zero failures or ignored tests.
- `cargo test --test commands visibility_keys`: compiled command catalog round-trip passed.

- `cargo test --test org_io odt_package`: two workflows passed, zero failures or ignored tests.
- Changed-source quality passed with unchanged native limits.

## Risks and Follow-ups

None recorded.
