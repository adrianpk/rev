# Slice 59: preview native Org links and images

Status: reviewing
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-59-previews`
PR: `#160`

## Purpose

Deliver explicit native resource previews under approved Slice 59, contracts
ORG30 and ORG15.

## Delivered Behavior

Explicit native commands enable, refresh and clear bounded view-local previews
of description-free links at point, in a region/section or across the buffer.
Direct Kitty terminals receive local PNGs behind readable source text, clipped
to the owning pane. Other resources retain links and the explicit configured
viewer path. Previewing preserves source bytes, point, coordinates and history;
visits and redraws never fetch or launch a viewer.

## Implementation Notes

Existing native link parsing, attachment containment and regular-file identity
checks own resource admission. Binary reads verify identity before/after bounded
ingress and recheck before a result is attached to a view. PNG container checks
retain exact encoded bytes, CRCs and finite dimensions; pixel decoding belongs
to the terminal. Fixed ceilings are 32 resources, 16 MiB aggregate image bytes,
4096 PNG chunks, 8192 pixels per axis and 16,777,216 aggregate pixels. Cancellation
is checked during reads and CRC work. Unsupported resources retain viewer-path
feedback. Four native preview commands, typed per-view preferences and explicit
viewer fallback now use these observations. Terminal output owns chunked transfers,
geometry updates and deletion of only its own image identifiers. Partial-write
cleanup closes incomplete graphics before retrying owned deletion. Each frame
keeps the ordinary completed-frame/cursor boundary after graphics.

Reference behavior: [Org previews](https://orgmode.org/manual/Images-and-link-previews.html)
and [Kitty graphics](https://sw.kovidgoyal.net/kitty/graphics-protocol/).

Task commits: T59.1 `8e82195c8eb1a1f64d75ce9983a3aa2d092c53da`,
T59.2 `b34afd6cf1ad6da2f267dcbe652390129daf8b2b`,
T59.3 `e03dd17ab7bb5828d6766cf9e3d40eefd57afc8a`.

Canonical PR: [#160](https://forge.adrianpk.com/adrianpk/rev/pulls/160), targeting `dev`.

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

Required ordered commands passed on the clean implementation candidate:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_io --test org_editing --test display --test pointer`
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-external`
- `python3 scripts/check-source.py`
- `git diff --check`

All 514 native regressions passed: 39 display, 226 editing, 192 I/O and 57 pointer;
zero failures, ignored or filtered tests. Implementation receipt
`X7FWFHNTZ3ATDN3XUD7A4MVTGH` identifies
`6ba48d0c90451d2507642dc2bc042ee501549fdd`.

Focused commands passed:

- `cargo test --test commands --test configuration -- --test-threads=4`
- `cargo test --lib -- --test-threads=4`
- `python3 scripts/check-config-reference.py`
- `git diff --check`

All 343 focused tests passed: 158 commands, 130 configuration and 55 library
cases, with zero failures, ignored or filtered tests. Focused implementation
receipt `FT5YIXZ2R3S5QO65I3X7AVTUFE` identifies
`e03dd17ab7bb5828d6766cf9e3d40eefd57afc8a`. Final readiness requires both ordered
lists on the clean PR-reference head; controller receipts and Acta retain that
candidate and the final validation IDs.

Native cases cover exact PNG bytes, source/point/history preservation, resource
revalidation, URI/format fallback, byte/resource-count limits, cancellation,
regular-file admission, corrupt CRCs, truncation, trailing bytes, animation and
excessive dimensions. Other cases cover split-local clear, read-only source,
stale-result rejection, typed origins/false/bounds and no resource IO on an
enabled view. The unchanged source checker retains safe Rust, five parameters,
three nesting levels and cyclomatic complexity ten. References agree on 178
schema/catalog entries, 54 RGB roles and 263 default bindings.

All four owned terminal sessions passed, zero skips: Kitty RGB, NO_COLOR
fallback, failed cleanup/retry and unwind. Source/graphics assertions separately
check exact base64 fixture bytes, bounded placement, unchanged source and point,
mouse coordinates, independent splits, resize without retransmission, viewer
consent and real child/group cancellation. Normal/failure/unwind restore termios,
paste, mouse, cursor/text colors and the alternate screen. Graphics-owner cases
also exercise chunk limits and partial write/flush/delete retries. The driver
separates APC graphics controls from source cells and independently checks their
payload, geometry and ownership; source assertions retain all ordinary cells.
The direct owned PTY clears inherited multiplexer variables only in its child.


## Risks and Follow-ups

PNG container admission does not decode compressed pixels. Native terminal
presentation is admitted only in direct Kitty environments; other terminals
retain source and an explicit configured viewer path. PTYs verify the graphics
wire and lifecycle, without capturing a terminal pixel renderer.
