# Slice 59: preview native Org links and images

Status: drafting
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
4096 PNG chunks, 8192 pixels per axis and 16,777,216 pixels per image. Cancellation
is checked during reads and CRC work. Unsupported resources retain viewer-path
feedback. Commands and terminal ownership are the next task.

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

## Validation

Development checks passed:

- `cargo check --lib`
- `cargo test --lib org::preview -- --test-threads=4`
- `git diff --check`

Three native cases passed, covering exact PNG bytes, source preservation,
resource revalidation, URI/format fallback, byte/resource-count limits,
cancellation, regular-file admission, corrupt CRCs, truncation, trailing bytes,
animation and excessive decoded dimensions. The unchanged source checker accepts
the new native code. Command/rendering integration and exact-head validation
remain pending.

## Risks and Follow-ups

PNG container admission does not decode compressed pixels. Terminal integration
must retain readable source and own transfer, geometry and restoration.
