# Org feeds

Declared RSS/Atom sources are data. Feed observations never interpret XML as
configuration, load entities or start a background poller.

`org.feeds.sources` holds at most 64 records with unique `name`, local path or
HTTP(S) `source`, target `file` and `headline`, optional `drawer` (FEEDSTATUS by
default) and optional transport argv. Paths retain their assigning-file origin.

`C-c C-x g` (`org-feed-update-all`) updates declared sources serially.
`org-feed-update` selects one by name. `C-c C-x G` (`org-feed-goto-inbox`)
opens the selected canonical file/heading without fetching; a missing heading is
an unsaved ordinary edit. `org-feed-results` inspects retained per-feed outcomes.
Visiting a file never reads a feed or starts a transport.

A local update uses bounded native file ingress; canonical unsaved XML wins over
disk. HTTP(S) uses the feed's `transport` override or `org.tools.fetch`. Empty
transport is a diagnosed missing prerequisite. Each actual tool launch requires
confirmation; decline and pre-launch cancellation create no process. Typed argv
admits at most 64 tokens of 4096 bytes each. One whole `{target}` token receives
the source URL or local path; otherwise it is appended once. Relative executables
with a slash resolve against the assigning configuration file. No token is
interpolated by a shell.

```toml
[org.tools]
fetch = ["curl", "--fail", "--silent", "--show-error", "{target}"]

[[org.feeds.sources]]
name = "News"
source = "news.xml"
file = "inbox.org"
headline = "News"
```

The existing process owner provides private HOME/XDG/TMP paths, no stdin,
combined output/time ceilings, child groups and cleanup. Confirmed programs have
normal host authority; isolation does not sandbox them. Effective fetch output
is the smaller `feed_bytes`/`job_output_bytes` ceiling, and `job_seconds` bounds
the operation. `C-g` cancels and reaps owned work. No idle poller remains.

Successful updates save the canonical destination, including its unsaved text,
using the ordinary checked file owner. Read-only, stale or failed publication
retains previous source/history. An unresolved completion remains a distinct
outcome with the inserted canonical status retained; repeating an update can
retry its save without duplicating entries. Update-all reports each source
separately and continues after a source failure.

The native parser supports RSS 0.91/0.92/2.0, RDF RSS 1.0 and Atom 1.0 namespaces.
UTF-8, XML comments/CDATA, predefined/numeric references, namespace bindings and
HTTP-relative links are observed as literal data. DTD/entity declarations,
unbound prefixes, malformed XML and duplicate/missing identities reject the
whole observation. No partial item set authorizes an import.

`org.limits.feed_bytes` defaults to 4 MiB (maximum 16 MiB), and `feed_items` to
1000 (maximum 10000). XML has at most 100000 nodes, 64 levels, 64 attributes and
64 namespace bindings per element, 128-byte names and 64 KiB text tokens/fields.
The ordinary Org source ceiling is 8 MiB. Cancellation remains observable during
native parsing and result preparation.

Items use guid, Atom id or RDF about; RSS may fall back to its link. FEEDSTATUS
uses a literal list of `(id handled checksum)` records compatible with Org's
status drawer. Native imports preserve handled records and prior checksums;
new records use `nil` for the optional checksum. They do not claim cross-parser
change-detection hashes. Existing handled items retain user edits. Retained
identity history is capped at 10000, and exhaustion rejects new imports instead
of forgetting IDs. Use different headings or drawers for feeds whose IDs may
collide.

Entries and status prepare together as one checked edit. Titles become child
headings, HTTP links remain links, and description/date text is fixed-width
literal content. Feed text cannot introduce configuration directives or source
blocks. Unknown or malformed status data is retained with a diagnostic.

See the [Org feed manual](https://orgmode.org/manual/RSS-Feeds.html),
[RSS specification](https://www.rssboard.org/rss-specification) and
[Atom RFC](https://www.rfc-editor.org/rfc/rfc4287).
