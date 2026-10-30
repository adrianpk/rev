# Org feeds

Declared RSS/Atom sources are data. Feed observations never interpret XML as
configuration, load entities or start a background poller.

`org.feeds.sources` holds at most 64 records with unique `name`, local path or
HTTP(S) `source`, target `file` and `headline`, optional `drawer` (FEEDSTATUS by
default) and optional transport argv. Paths retain their assigning-file origin.

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
