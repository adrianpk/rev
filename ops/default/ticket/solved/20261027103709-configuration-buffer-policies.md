---
id: TKT-20261027103709
title: Apply buffer and language editing preferences
status: solved
kind: task
severity: unclassified
priority: normal
scope: domain
tags: configuration
source: chat
reported_at: 2026-10-27T10:37:09Z
ready_at: 2026-10-27T10:37:09Z
commits: [95923e869370a44de581ed34003fc228549049ef, 9db4f5682730cb95dbe33bad260adf972aad8b1d, e6d781ad529cdb57ca20ede62ac76984c6a77505, 7672b1eed5709c05d0221f705603ba12628c203f, a4d9565cf4fcff7487808fe2281f78ec1d1ea24f]
started_at: 2026-10-27T18:27:50Z
reviewed_at: 2026-10-27T18:27:50Z
reconciled_at: 2026-10-27T18:27:50Z
closed_at: 2026-10-27T18:27:50Z
resolution: fixed
---

# Apply buffer and language editing preferences

## Expected outcome

Implement the accepted [configuration contract](../../spec/editor-configuration.md)
and [model catalog](../../spec/editor-configuration-model.md) for CFG09-CFG11, CFG16, modified R17/R20.
The approved [master plan](../../plan/rev-configuration-ui.md) defines exact task boundaries.

## Acceptance criteria

- Slice 5: Identify buffer languages; pass its recorded focused checks and report its observable outcome.
- Slice 6: Project buffer tab policies; pass its recorded focused checks and report its observable outcome.
- Slice 7: Apply indentation and new-file defaults; pass its recorded focused checks and report its observable outcome.
- Slice 8: Capture Emacs-style rectangles; pass its recorded focused checks and report its observable outcome.
- Slice 9: Keep rectangle and path transitions coherent; pass its recorded focused checks and report its observable outcome.
- Preserve canonical text, undo, saved-state identity and safe file publication.
- Record implementation commits, slice PRs and exact-candidate validation;
  move to solved only after every assigned slice integrates successfully.

## Delivery

Approved as part of the single configuration and interaction delivery. Slice
branches and reports are authoritative; this ticket does not request a separate
parallel branch or a second tracker. No future-inventory feature is included.

## Verified delivery evidence

- Slice 5: [#54](https://forge.adrianpk.com/adrianpk/rev/pulls/54); integrated `95923e869370a44de581ed34003fc228549049ef`. [Report](../../report/slices/rev-configuration-ui/slice-5-file-associations.md) records the delivered behavior and exact-head checks.
- Slice 6: [#55](https://forge.adrianpk.com/adrianpk/rev/pulls/55); integrated `9db4f5682730cb95dbe33bad260adf972aad8b1d`. [Report](../../report/slices/rev-configuration-ui/slice-6-buffer-display-policy.md) records the delivered behavior and exact-head checks.
- Slice 7: [#56](https://forge.adrianpk.com/adrianpk/rev/pulls/56); integrated `e6d781ad529cdb57ca20ede62ac76984c6a77505`. [Report](../../report/slices/rev-configuration-ui/slice-7-indentation-and-format.md) records the delivered behavior and exact-head checks.
- Slice 8: [#57](https://forge.adrianpk.com/adrianpk/rev/pulls/57); integrated `7672b1eed5709c05d0221f705603ba12628c203f`. [Report](../../report/slices/rev-configuration-ui/slice-8-rectangle-capture.md) records the delivered behavior and exact-head checks.
- Slice 9: [#58](https://forge.adrianpk.com/adrianpk/rev/pulls/58); integrated `a4d9565cf4fcff7487808fe2281f78ec1d1ea24f`. [Report](../../report/slices/rev-configuration-ui/slice-9-policy-transitions.md) records the delivered behavior and exact-head checks.
