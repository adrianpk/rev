# Slice 50: export native Org to OpenDocument text

Status: drafting
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-50-odt-export`
PR: pending

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

## Implementation Notes

`Snapshot::package` returns binary archive bytes separately from text rendering.
XML emission, CRC and ZIP assembly share cancellation, operation deadline and the
snapshot output limit, including archive headers and directory records.
Preparation has no publication, asset-reading, process or network authority.
The package profile follows the OASIS OpenDocument 1.3 package specification.

## Contracts Added or Changed

- ORG25: native OpenDocument text export from checked current source.
- ORG18: bounded package assembly retains ordinary editor guarantees.

## Files of Interest

- `src/org/export/odt.rs`: native package and metadata assembly.
- `src/org/export/odt/zip.rs`: fixed stored ZIP profile and CRC32.
- `src/org/export/odt/xml.rs`: bounded XML text and whitespace admission.
- `tests/org_io/odt_exports.rs`: independent ZIP/XML expectations.

## Validation

- `cargo test --test org_io odt_package`: two workflows passed, zero failures or ignored tests.
- Changed-source quality passed with unchanged native limits.

## Risks and Follow-ups

Rich native forms, command integration and final candidate validation remain in progress.
