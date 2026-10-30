# Native Org capability and evidence map

Status: reconciliation in progress
Specification: [Org Mode](editor-org-mode.md)
Model: [Org Mode Model](editor-org-mode-model.md)
Plan: [Native Org delivery](../plan/rev-org-mode.md)
Tracker: [Native Org tracker](../tracker/rev-org-mode.md)

## Preserved tested candidates

| Delivery | Original tested candidate | Owning evidence |
| --- | --- | --- |
| Editor | `b56d705c7a84b823bf069fc874bc762938d6d897` | [Editor baseline](rev-editor.md), A01-A20 and R01-R37. |
| Configuration/interaction | `95713efb7f79fbc72417615a921841f8d49730ed` | [Configuration](editor-configuration.md), gate `FWDUCVRJCYBKFXEWEFLKPKA35E`. |
| Input/navigation | `dedd85064cb5565a5382a9d8e2e2902daae24d88` | [Editor baseline](rev-editor.md), gate `PZ4OIPXA6GUBSWDH5ZS7MJUOGX`. |
| Org combined terminal | `14cbf761a6492994da8d1f6674c8013046cb4878` | [Slice 61](../report/slices/rev-org-mode/slice-61-combined-terminal.md), required `B6DJ6COVBO6RDOADHXBI24YS5G`: 948 native tests and 69 owned PTYs. |
| Org resources | `18f55af4af0ecef5a88feeab388edbb3a53d66b7` | [Slice 62](../report/slices/rev-org-mode/slice-62-bounded-work.md), required `SLV234DTE5VQ7XX7PJVCU22J5S`: 522 native tests, historical and Org release measurements. |

These identities keep their original meaning. Terminal and resource records from
different candidates are not combined into a new tested head. Slice 63 runs its
own required native/reference/prerequisite checks. The remaining integration and
unchanged full delivery gate still belong to the controller.

## Native capability boundaries

[Controls](../../../docs/reference/org-controls.md) maps every shipped capability
to actual compiled commands, typed preferences and fixed native policies.
[Org mode](../../../docs/reference/org-mode.md) owns canonical editing and view
behavior; the feature references linked there retain their detailed context,
source preservation, refusal and publication contracts.

Core Org editing, calendar/agenda, capture, formulas and native export work without
an interpreter, converter, network connection or external service. Optional
programs run only for an explicitly selected documented action and remain outside
the editor's configuration/runtime language. Missing tools produce usable failure
results; they are not reported as skipped native acceptance.
