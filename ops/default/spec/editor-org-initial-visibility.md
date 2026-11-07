# Org Initial Visibility and Drawers

Status: Approved
Date: 2026-11-06
Updated: 2026-11-07
Inspection baseline: `70c4e95bbd31a3550f5928ccaa1fbc1b8945ab23`
Format: OpenSpec capability addition
Approved: product owner requested specification, planning and one Looper delivery on 2026-11-07
Source: [TKT-20261103111614](../ticket/in_progress/20261103111614-configure-org-drawer-visibility.md)
Baseline: [Native Org](editor-org-mode.md), ORG03 and ORG05
Model: [Org Visibility Policy](editor-org-initial-visibility-model.md)
Plan: [Org Initial Visibility Delivery](../plan/rev-org-initial-visibility.md)
Tracker: [Org Initial Visibility Tracker](../tracker/rev-org-initial-visibility.md)

## Purpose and compatibility

Configure initial outline and drawer presentation and control all drawers in
the selected view. Reuse native structural observations and checked folds.
Visibility changes do not edit or save source text.

The earlier isolated draft proposed showall as a new default. This delivery
preserves the current fully visible ordinary-file default, showeverything,
which also follows the reference editor. Users can select showall to keep
ordinary text visible with metadata drawers closed. Existing explicit
org-show-all continues to expose everything, including drawer bodies.
Existing local/global cycling and individual drawer commands retain their
behavior; this addition does not redefine their existing states.

References verified on 2026-11-07:

- [Initial visibility](https://orgmode.org/manual/Initial-visibility.html)
- [Drawers](https://orgmode.org/manual/Drawers.html)
- [Global and local cycling](https://orgmode.org/manual/Global-and-local-cycling.html)

## Startup policy

```toml
[org]
startup_folded = "showeverything"
```

| Value | Initial presentation |
| --- | --- |
| `overview` | Top-level headings; heading bodies hidden |
| `content` | All headings; heading bodies hidden |
| `showall` | All headings and ordinary bodies visible; drawer bodies hidden |
| `showeverything` | All content visible, including drawer bodies |

The finite string enum uses ordinary startup validation, include/override
precedence and provenance. Omission inherits showeverything without rewriting
an existing configuration; an unsupported configuration value rejects the
candidate through the normal configuration boundary.

A native recognized `#+STARTUP:` keyword can override the configuration for
its document. Keywords may occur outside the preamble. Compare the keyword and
the four visibility tokens without case sensitivity. The last recognized
visibility token in source order wins, including multiple words on one line.
Other startup words and unsupported visibility words remain literal metadata
and do not suppress the last recognized policy or its configuration fallback.
Do not interpret directives inside source/example/literal blocks or drawers.
This finite subset does not add depth-specific states or per-heading VISIBILITY
properties.

## View lifetime and native ownership

Resolve initial policy through the existing bounded native observer. Apply it
once to a genuinely new Org view after a complete current observation is
available. A pending, cancelled, partial or malformed observation cannot
authorize guessed fold ranges. Leave the view usable while observation is
pending and preserve the ordinary failure/reporting path.

An explicit visibility choice made before initialization completes wins over
that pending initialization. A split clones its originating view's current
folds and initialization state; subsequent commands affect the selected view
only. Repainting, resizing, buffer switching, source edits and external reload
do not repeatedly apply startup policy. Retire stale ranges using existing
source/revision rules; retain deliberate folds only where those rules admit
them. Reopening a closed view initializes it again.

The reset command resolves the current document policy and applies it explicitly.
Normal search, jump and source-navigation reveal rules remain authoritative.
Keep point reachable through existing checked fold anchors and reveal rules;
never substitute a visual row for canonical source identity.

Generated AI Org results keep their existing deliberate REQUEST initialization.
The general startup policy must not reopen that drawer or repeatedly override
the user's later choice. No Pi/network request is part of this behavior.

## Named commands

All four commands use M-x and the modal colon selector. They add no factory
keybinding. Register their descriptions with the existing metadata owners and
complete command/binding references; preserve the previous descriptions.

| Command | Exact description |
| --- | --- |
| `org-hide-all-drawers` | Hide all valid Org drawer bodies in the selected view. |
| `org-show-all-drawers` | Show all valid Org drawer bodies in the selected view. |
| `org-toggle-all-drawers` | Toggle all valid Org drawer bodies in the selected view. |
| `org-cycle-set-startup-visibility` | Restore the selected Org view's configured or document startup visibility. |

Drawer commands affect recognized property and custom drawer bodies without
changing heading/body folds, ordinary prose or literal blocks. Existing native
delimiter presentation is retained. Toggle hides all admitted drawers if any
is open; otherwise it opens them all. An empty valid drawer set is a harmless
no-op with ordinary feedback. Incomplete or malformed structure must not fold
uncertain ranges. The same UI-only behavior remains available in read-only
Org buffers. Each invocation uses one normal command identity and usage record.

## ADDED Requirements

### Requirement: OIV01 Configure initial visibility and document precedence

The editor SHALL admit the four policy values and the compatibility default
described above. Only current structurally recognized document metadata SHALL
override admitted configuration. Preserve source spelling and unknown words.

#### Scenario: Existing configuration

- **GIVEN** a configuration that omits startup_folded
- **WHEN** an ordinary Org file opens without recognized startup metadata
- **THEN** all its content is initially visible as before this addition.

#### Scenario: Readable prose with hidden metadata

- **GIVEN** configured showall and valid property/custom drawers
- **WHEN** a new Org view initializes
- **THEN** heading and ordinary body text remains visible while drawer bodies fold.

#### Scenario: Source precedence and literal protection

- **GIVEN** several mixed-case startup keywords, unrelated words and literal examples
- **WHEN** the native observation resolves the policy
- **THEN** the last recognized nonliteral visibility token wins
- **AND** examples, drawer contents and unknown words cannot change that result.

### Requirement: OIV02 Control all drawers independently of the outline

Whole-view drawer commands SHALL affect valid drawer bodies only. Preserve
heading/body visibility, individual drawer commands and source coordinates.

#### Scenario: Mixed drawer state

- **GIVEN** open and closed custom/property drawers and a folded subtree
- **WHEN** the user toggles all drawers
- **THEN** all admitted drawer bodies close and the subtree stays folded
- **AND** toggling again opens drawer bodies without unfolding that subtree.

#### Scenario: Invalid and empty targets

- **GIVEN** no valid drawers or incomplete/malformed structural information
- **WHEN** a drawer command runs
- **THEN** it does not invent ranges or change ordinary source/body visibility.

### Requirement: OIV03 Initialize once and preserve independent views

Initial policy SHALL apply once per genuinely new view. Explicit runtime choices
SHALL outrank pending initialization. A split SHALL inherit current visibility,
and later changes SHALL remain local to the selected view.

#### Scenario: Runtime choice and delayed observation

- **GIVEN** initialization is pending and the user explicitly reveals content
- **WHEN** a current structural observation arrives
- **THEN** initialization does not collapse the explicitly revealed content.

#### Scenario: Split, edit and reload

- **GIVEN** two split views with different runtime folds
- **WHEN** a redraw, edit or external reload changes observation readiness
- **THEN** existing admission retires stale ranges without reapplying startup
- **AND** one pane's new visibility command does not change the other's state.

### Requirement: OIV04 Preserve source, full reveal and bounded work

Use existing native observation, revision checks, cancellation and fold limits.
Add no synchronous whole-file scan to drawing or idle polling. Visibility
SHALL NOT create source edits, saves, undo entries or persisted fold state.
Existing org-show-all and source navigation SHALL retain their behavior.

#### Scenario: Full reveal and deliberate AI presentation

- **GIVEN** folded headings/drawers or an AI result with REQUEST deliberately closed
- **WHEN** initial policy and later explicit visibility commands are applied
- **THEN** automatic initialization preserves deliberate AI presentation
- **AND** explicit org-show-all remains able to reveal everything.

### Requirement: OIV05 Expose complete native command metadata and guidance

The four exact names and descriptions SHALL participate in the shared command
catalog and both selectors. Documentation SHALL distinguish startup tokens,
drawer-only commands and the existing full-reveal command.

#### Scenario: Both editing profiles

- **GIVEN** either editing profile and an active Org view
- **WHEN** the user completes and invokes a new named command
- **THEN** the shared selector displays its description and invokes it once
- **AND** the action preserves source bytes, dirty state, mark and editing history.

## Delivery and documentation

One vertical slice implements the complete policy, controls and acceptance.
Update Org reference, the full TOML sample and configuration catalog, existing
command descriptions/binding references and the separate Org User Guide path.
Project Markdown into Spore and update Unreleased. Native source/configuration
fixtures cover both editing profiles, all policies, Unicode/EOF, read-only
buffers, split/reload and delayed observation, including source-state safety.
Use existing limits and owners; do not introduce a separate folding subsystem.
