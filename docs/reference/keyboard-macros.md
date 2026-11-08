# Keyboard Macros

Both startup profiles use the same native recording, replay, ring, naming and
persistence owners. Function keys and C-x gestures are ordinary effective bindings.
Modal recordings also check their source entry state before playback and further
iterations. Printable keys and native modal operators retain their current meanings.

- [Commands and descriptions](commands.md#keyboard-recording-and-replay): every macro command, count policy, counter format, query, region and named-call rule.
- [Bindings](bindings.md#keyboard-recording-and-replay): global gestures and named definition/menu actions.
- [Definition, recent-input and step editing](macro-editing.md): checked TOML drafts, native reads and read-only ring management.
- [Saved library](macro-library.md): exact schema, step fields, private state ownership, explicit writes, concurrency and backup.
- [Configuration catalog](defaults.md#configuration-value-catalog): `macros.ring_limit`, `macros.execute_before_append`, `keys.macro_definition` and `keys.macro_menu`.

## Limits and stopping behavior

| Resource | Bound |
| --- | --- |
| Session ring | Default 8; configured 1..32 entries. |
| Named definitions and saved records | 256 each; admitted saved names share the session name ceiling. |
| Session bindings | 256 entries. |
| One definition | 4,096 steps and 256 KiB encoded. |
| Encoded saved library | 8 MiB. |
| Description | 512 UTF-8 bytes. |
| Recent recordable input | 300 logical inputs and 256 KiB, memory-only. |
| Macro-owned retained data | 48 MiB, including retained shared/copy targets. |
| Counter format and result | 256 / 512 UTF-8 bytes; width/precision at most 128. |
| Named call depth / manual pause frames | 16 / 4. |
| Original region-line targets | 65,536 logical line beginnings. |
| One outer run | 1,000,000 dispatched logical inputs, 1,000,000 iterations, 64 MiB generated text and 60 active automatic seconds. |
| Cooperative host work | At most 256 logical inputs per turn, with a four-millisecond yield target between native primitives. |

Native source/edit admission remains independent, including its existing 8 MiB
single-edit bound. User query, consent and manual waits do not consume active
automatic time. Native I/O keeps its own deadline. Exhaustion stops explicitly
and preserves real completed/partial effects. A saturated movement that succeeds
normally is not a stopping signal; zero/negative replay still has finite budgets.

C-g cancels capture/playback and pending repeat/count input. Aborted or over-limit
capture retains the previous valid recording. Source loss, an incompatible entry
state, native failure or missing command stops before further effects. Protected
components and external jobs cannot be treated as synchronous macro steps.

The recording indicator is Def, automatic playback shows Macro, a query shows
Macro?, and manual editing shows its bounded depth. Step questions display the
next logical step and their own answer keys. Completion or cancellation releases
automatic macro wake ownership; no persistent macro service is started.

For actions, follow [Apply a Keyboard Macro](../how-to/apply-keyboard-macros.md) or
[Save a Keyboard Macro](../how-to/save-keyboard-macros.md). For the ownership model,
read [Keyboard Macro Ownership](../explanation/keyboard-macro-ownership.md).
