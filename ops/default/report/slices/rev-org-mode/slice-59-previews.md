# Slice 59: preview native Org links and images

Status: reviewing
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-59-previews`
PR: pending

## Purpose

Deliver explicit native resource previews under approved Slice 59, contracts
ORG30 and ORG15.

## Delivered Behavior

Native link observations select description-free external resources at point,
in a region or section, or across the buffer. Local PNG observations are bounded
and disposable. Source and resource identity remain separate; no source/history
publication, viewer launch or network operation belongs to observation.

## Implementation Notes

Existing native link parsing, attachment containment and regular-file identity
checks own resource admission. Binary reads verify identity before/after bounded
ingress and recheck before a result is attached to a view. PNG container checks
retain exact encoded bytes, CRCs and finite dimensions; pixel decoding belongs
to the terminal. Fixed ceilings are 32 resources, 16 MiB aggregate image bytes,
4096 PNG chunks, 8192 pixels per axis and 16,777,216 aggregate pixels. Cancellation
is checked during reads and CRC work. Unsupported resources retain viewer-path
feedback. Four native preview commands, typed per-view preferences and explicit viewer
fallback now use these observations. Terminal output owns chunked transfers,
geometry updates and deletion of only its own image identifiers. Partial-write
cleanup closes incomplete graphics before retrying owned deletion. Each frame
keeps the ordinary completed-frame/cursor boundary after graphics.

Reference behavior: [Org previews](https://orgmode.org/manual/Images-and-link-previews.html)
and [Kitty graphics](https://sw.kovidgoyal.net/kitty/graphics-protocol/).

## Contracts Added or Changed

- ORG30: bounded disposable preview observations without source authority.
- ORG15: checked native links and attachment resource containment.

## Files of Interest

- `src/org/preview.rs` and `src/org/preview/png.rs`: resources and PNG admission.
- `src/org/links/preview.rs`: native source/selection observations.
- `src/org/io/bytes.rs`: bounded binary ingress through file identity checks.
- `src/org/preview/tests.rs`: real resource, hostile container and limit cases.
- `src/session/previews.rs`: explicit commands and revision-qualified view publication.
- `src/display/preview.rs` and `src/terminal/graphics.rs`: bounded geometry and output ownership.
- `scripts/configuration_ui/org_external.py`: real owned source/viewer/terminal journey.
- `examples/terminal_probe/preview.rs`: existing terminal-owner failure/unwind probe.

## Validation

Development checks passed:

- `cargo check --lib`
- `cargo test --lib org::preview -- --test-threads=4`
- `cargo test --test org_io previews -- --test-threads=4`
- `cargo test --test commands --test configuration -- --test-threads=4`
- `python3 scripts/check-config-reference.py`
- `cargo test --lib terminal::graphics -- --test-threads=4`
- `cargo test --test configuration org_previews -- --test-threads=4`
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-external`
- `git diff --check`

Three native cases passed, covering exact PNG bytes, source preservation,
resource revalidation, URI/format fallback, byte/resource-count limits,
cancellation, regular-file admission, corrupt CRCs, truncation, trailing bytes,
animation and excessive decoded dimensions. The unchanged source checker accepts
the new native code. Three additional session cases passed for exact source/point/history, split-local
clear, read-only fallback, cancellation and stale-result rejection. All 287
command/configuration tests passed. References agree on 178 schema/catalog
entries, 54 RGB roles and 263 default bindings. Four owned terminal sessions passed: Kitty RGB output, NO_COLOR fallback,
failed cleanup/retry and unwind. Source/graphics assertions separately check exact
base64 fixture bytes, bounded placements, unchanged source and point, mouse
coordinates, independent splits, data reuse across resize, viewer consent and
real child/group cancellation. Normal/failure/unwind paths restore termios,
paste, mouse, cursor/text colors and the alternate screen, with no skipped cases.
The graphics-owner cases separately cover chunk limits and partial write/flush/
delete retry. Configuration retains assigning origins, explicit false, bounds
and no resource IO on an enabled view. Required exact-head validation remains
pending.

## Risks and Follow-ups

PNG container admission does not decode compressed pixels. Native terminal
presentation is admitted only in direct Kitty environments; other terminals
retain source and an explicit configured viewer path. PTYs verify the graphics
wire and lifecycle, without capturing a terminal pixel renderer.
