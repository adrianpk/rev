use super::execution::configured;
use super::*;
use rev::file::Publication;
const XML: &str = "<rss version='2.0'><channel><item><guid>stable</guid><title>Item 界</title><description>Literal &amp; text</description><link>https://example.test/item</link></item></channel></rss>";
const POLICY: &str =
    "[[org.feeds.sources]]\nname='News'\nsource='news.xml'\nfile='inbox.org'\nheadline='Inbox'\n";
// An explicit local update reads/imports/saves through real file ownership.
// Repeating it against the opened canonical inbox cannot duplicate an item.
#[test]
fn feed_local() {
    let (f, mut s) = configured("* Selected\n", POLICY);
    f.write("news.xml", XML);
    action(&mut s, C::FeedUpdateAll);
    let result = &s.feed_results()[0];
    assert_eq!(
        result.publication,
        Some(Publication::Complete),
        "{}",
        result.detail
    );
    assert_eq!(result.added, 1);
    assert_eq!(result.skipped, 0);
    assert_eq!(text(&s), "* Selected\n");
    let path = f.0.join("inbox.org");
    let saved = std::fs::read_to_string(&path).unwrap();
    assert!(saved.contains("** Item 界\n"));
    assert!(saved.contains("((\"stable\" t nil))"));
    s.load(&path).unwrap();
    settle(&mut s);
    assert_eq!(text(&s), saved);
    assert_eq!(s.buffer().unwrap().history().len(), 1);
    action(&mut s, C::FeedUpdateAll);
    assert_eq!(s.feed_results()[0].added, 0);
    assert_eq!(s.feed_results()[0].skipped, 1);
    assert_eq!(text(&s), saved);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), saved);
}
// Selecting an inbox is independent of transport/input availability. It makes a
// missing heading through ordinary source history without fetching or saving it.
#[test]
fn feed_inbox() {
    let (f, mut s) = configured("* Selected\n", POLICY);
    action(&mut s, C::FeedGotoInbox);
    argument(&mut s, "News");
    assert_eq!(text(&s), "* Inbox\n");
    assert!(s.buffer().unwrap().is_modified());
    assert!(!f.0.join("inbox.org").exists());
    assert_eq!(s.feed_results()[0].publication, None);
    s.execute(Command::History(rev::command::History::Undo));
    settle(&mut s);
    assert_eq!(text(&s), "");
}
// The selected adapter receives one literal path token, owns private HOME and
// has no effect before consent. Native ingress and output share finite bounds.
#[test]
fn feed_transport() {
    let policy = format!("{POLICY}transport=['/bin/sh','{{target}}']");
    let (f, mut s) = configured("* Selected\n", &policy);
    f.write(
        "news.xml",
        &format!(
            "printf '%s' \"$HOME\" > '{}'\ncat <<'REV_FEED'\n{XML}\nREV_FEED\n",
            f.0.join("home").display()
        ),
    );
    action(&mut s, C::FeedUpdateAll);
    assert!(!f.0.join("home").exists());
    argument(&mut s, "no");
    assert!(!f.0.join("home").exists());
    assert!(!f.0.join("inbox.org").exists());
    action(&mut s, C::FeedUpdate);
    argument(&mut s, "News");
    argument(&mut s, "yes");
    let result = &s.feed_results()[0];
    assert_eq!(
        result.publication,
        Some(Publication::Complete),
        "{}",
        result.detail
    );
    let home = std::fs::read_to_string(f.0.join("home")).unwrap();
    assert!(!Path::new(&home).exists());
    assert_eq!(result.added, 1);
}
// A bad source or destination never becomes a successful partial feed. Other
// explicitly selected feeds retain independent, truthful outcomes in update-all.
#[test]
fn feed_failures() {
    for (name, source, file, tool, detail) in [
        ("invalid", "<!DOCTYPE rss><rss/>", "inbox.org", "", "DTD"),
        ("absent", "", "inbox.org", "", "missing"),
        ("destination", XML, "missing/inbox.org", "", ""),
        (
            "tool",
            XML,
            "inbox.org",
            "transport=['/missing/feed-transport']",
            "Start configured",
        ),
    ] {
        let policy = format!(
            "{}\n{tool}",
            POLICY.replace("file='inbox.org'", &format!("file='{file}'"))
        );
        let (f, mut s) = configured("* Selected\n", &policy);
        if name != "absent" {
            f.write("news.xml", source);
        }
        action(&mut s, C::FeedUpdateAll);
        if name == "tool" {
            argument(&mut s, "yes");
        }
        let result = &s.feed_results()[0];
        assert_eq!(
            result.publication,
            Some(Publication::NotPublished),
            "{name}: {}",
            result.detail
        );
        assert!(
            result
                .detail
                .to_ascii_lowercase()
                .contains(&detail.to_ascii_lowercase()),
            "{name}: {}",
            result.detail
        );
        assert!(!f.0.join(file).exists(), "{name}");
        assert_eq!(text(&s), "* Selected\n");
    }
}
