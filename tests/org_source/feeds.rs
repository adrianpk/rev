use rev::{
    config::OrgLimits,
    org::{Cancellation, feed::Feed},
};
const RSS: &str = "<rss version='2.0'><channel><title>News</title><item><guid>one</guid><title>First &amp; &#x754c;</title><description><![CDATA[Literal <b>body</b>]]></description><link>../one</link></item></channel></rss>";
// RSS, RDF and namespaced Atom observations retain literal identity/content;
// relative links follow xml:base, while foreign extension names cannot spoof IDs.
#[test]
fn feed_formats() {
    let cases = [
        ("rss", RSS, "one", "First & 界", "https://example.test/one"),
        (
            "atom",
            "<?xml version='1.0' encoding='UTF-8'?><a:feed xmlns:a='http://www.w3.org/2005/Atom' xml:base='https://example.test/root/'><a:entry><a:id>urn:one</a:id><a:title>First</a:title><a:link href='entry'/><a:content type='xhtml'><div xmlns='http://www.w3.org/1999/xhtml'><p>body</p></div></a:content></a:entry></a:feed>",
            "urn:one",
            "First",
            "https://example.test/root/entry",
        ),
        (
            "rdf",
            "<rdf:RDF xmlns:rdf='http://www.w3.org/1999/02/22-rdf-syntax-ns#' xmlns='http://purl.org/rss/1.0/'><item rdf:about='identity'><title>First</title><link>https://example.test/one</link></item></rdf:RDF>",
            "identity",
            "First",
            "https://example.test/one",
        ),
    ];
    for (name, source, id, title, link) in cases {
        let feed = Feed::parse(
            source,
            Some("https://example.test/feeds/news"),
            &OrgLimits::default(),
            &Cancellation::default(),
        )
        .unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(feed.entries.len(), 1, "{name}");
        assert_eq!(feed.entries[0].id, id, "{name}");
        assert_eq!(feed.entries[0].title, title, "{name}");
        assert_eq!(feed.entries[0].link.as_deref(), Some(link), "{name}");
    }
}
// Hostile XML and ambiguous identities reject the entire observation. No DTD or
// reader extension can cause external IO, expansion or partial admitted entries.
#[test]
fn feed_rejections() {
    for (name,source) in [
        ("doctype", "<!DOCTYPE rss [<!ENTITY x SYSTEM 'file:///unowned'>]><rss version='2.0'><channel/></rss>".into()),
        ("entity",RSS.replace("First &amp; &#x754c;","&unknown;")),
        ("mismatch",RSS.replace("</item>","</other>")),
        ("duplicate",RSS.replace("</channel>","<item><guid>one</guid><title>Again</title></item></channel>")),
        ("missing identity",RSS.replace("<guid>one</guid>","").replace("<link>../one</link>","")),
        ("namespace", "<a:feed><a:entry/></a:feed>".into()),
        ("two roots",format!("{RSS}{RSS}")),
        ("control",RSS.replace("First", "Bad\0")),
        ("surrogate",RSS.replace("First", "&#xD800;")),
        ("depth",format!("{}{}","<x>".repeat(65),"</x>".repeat(65))),
        ("encoding","<?xml version='1.0' encoding='UTF-16'?><rss version='2.0'><channel/></rss>".into()),
    ] {
        assert!(Feed::parse(&source,None,&OrgLimits::default(),&Cancellation::default()).is_err(),"{name}");
    }
    let mut limits = OrgLimits::default();
    limits.feed_bytes = RSS.len() - 1;
    assert!(Feed::parse(RSS, None, &limits, &Cancellation::default()).is_err());
    limits.feed_bytes = RSS.len();
    assert!(Feed::parse(RSS, None, &limits, &Cancellation::default()).is_ok());
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(Feed::parse(RSS, None, &limits, &cancel).is_err());
}

// Exact item ceilings admit a complete feed; overflow is a diagnosed failure,
// while foreign namespace fields never replace the native stable identity.
#[test]
fn feed_items() {
    let source = RSS.replace("</channel>", "<item xmlns:x='urn:extension'><x:guid>spoof</x:guid><guid>two</guid><title>Second</title></item></channel>");
    let limits = OrgLimits {
        feed_items: 1,
        ..OrgLimits::default()
    };
    assert!(
        Feed::parse(&source, None, &limits, &Cancellation::default())
            .unwrap_err()
            .contains("item limit")
    );
    let limits = OrgLimits {
        feed_items: 2,
        ..limits
    };
    let feed = Feed::parse(&source, None, &limits, &Cancellation::default()).unwrap();
    assert_eq!(
        feed.entries
            .iter()
            .map(|e| e.id.as_str())
            .collect::<Vec<_>>(),
        ["one", "two"]
    );
}
