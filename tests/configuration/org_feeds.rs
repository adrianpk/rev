use super::fixture::Fixture;
use rev::config::prepare;
const FEED: &str = "[[org.feeds.sources]]\nname='News'\nsource='https://example.test/news.xml'\nfile='inbox.org'\nheadline='Inbox'\n";
// Feed arrays replace by leaf and retain assigning-file origin. Configuration
// admits absent files/tools without eager reads, fetches or workspace creation.
#[test]
fn feed_configuration() {
    let f = Fixture::new();
    let base = f.write("profiles/base.toml", &format!("org.tools.fetch=['missing-tool','{{target}}']\norg.limits.feed_bytes=200\norg.limits.feed_items=2\n{FEED}"));
    let path = f.write("config.toml", "include=['profiles/base.toml']");
    let startup = prepare(&path);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let snapshot = startup.snapshot;
    assert_eq!(
        snapshot.origin(&["org", "feeds", "sources"]),
        Some(base.as_path())
    );
    assert_eq!(
        snapshot.origin(&["org", "tools", "fetch"]),
        Some(base.as_path())
    );
    assert_eq!(snapshot.settings().org.limits.feed_bytes, 200);
    assert_eq!(snapshot.settings().org.limits.feed_items, 2);
    assert_eq!(
        snapshot.settings().org.feeds.sources[0].drawer,
        "FEEDSTATUS"
    );
    let path = f.write(
        "config.toml",
        "include=['profiles/base.toml']\norg.feeds.sources=[]\norg.tools.fetch=[]",
    );
    let replaced = prepare(&path);
    assert!(replaced.diagnostic.is_none());
    assert!(replaced.snapshot.settings().org.feeds.sources.is_empty());
    assert!(replaced.snapshot.settings().org.tools.fetch.is_empty());
    assert!(!f.0.join("profiles/news.xml").exists());
    assert!(!f.0.join("profiles/inbox.org").exists());
}
// Closed declarations and typed argv reject ambiguous names, unsafe token
// substitution and out-of-contract resource values before any job is possible.
#[test]
fn feed_rejections() {
    let f = Fixture::new();
    for (name, source) in [
        ("duplicate", format!("{FEED}{FEED}")),
        (
            "scheme",
            FEED.replace(
                "https://example.test/news.xml",
                "ftp://example.test/news.xml",
            ),
        ),
        (
            "empty host",
            FEED.replace("https://example.test/news.xml", "https://"),
        ),
        ("drawer", format!("{FEED}drawer='LOG-BOOK'")),
        ("blank", FEED.replace("name='News'", "name=' '")),
        ("extra", format!("{FEED}poll=true")),
        (
            "embedded",
            format!("{FEED}transport=['curl','--url={{target}}']"),
        ),
        (
            "twice",
            format!("{FEED}transport=['curl','{{target}}','{{target}}']"),
        ),
        ("executable", format!("{FEED}transport=['{{target}}']")),
        ("zero bytes", "org.limits.feed_bytes=0".into()),
        ("over bytes", "org.limits.feed_bytes=16777217".into()),
        ("zero items", "org.limits.feed_items=0".into()),
        ("over items", "org.limits.feed_items=10001".into()),
        (
            "argv",
            format!("org.tools.fetch=[{}]", vec!["'x'"; 65].join(",")),
        ),
        (
            "sources",
            (0..65)
                .map(|n| FEED.replace("News", &format!("News{n}")))
                .collect(),
        ),
    ] {
        let path = f.write("invalid.toml", &source);
        assert!(prepare(&path).diagnostic.is_some(), "{name}");
    }
}
