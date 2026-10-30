use super::*;
use rev::{
    config::{FeedSource, Org},
    org::{Cancellation, feed::Feed, io::CaptureRead},
};
const XML: &str = "<rss version='2.0'><channel><item><guid>stable</guid><title>News 界</title><description><![CDATA[#+BEGIN_SRC sh\n* Untrusted heading\n:END:]]></description><link>https://example.test/news</link></item></channel></rss>";
fn definition() -> FeedSource {
    FeedSource {
        name: "News".into(),
        source: "news.xml".into(),
        file: "inbox.org".into(),
        headline: "Inbox".into(),
        drawer: "FEEDSTATUS".into(),
        transport: None,
    }
}
fn observe(buffers: &Buffers, path: &Path) -> rev::org::feed::Plan {
    let policy = Org::default();
    let cancel = Cancellation::default();
    let source = CaptureRead::new(buffers, path, &policy.limits)
        .unwrap()
        .run(&cancel)
        .unwrap();
    Feed::parse(XML, None, &policy.limits, &cancel)
        .unwrap()
        .plan(source, &definition(), &policy, &cancel)
        .unwrap()
}
// The same checked transaction adds entries and literal compatible FEEDSTATUS.
// Repeating against canonical unsaved text deduplicates and leaves disk exact.
#[test]
fn feed_transaction() {
    let f = Fixture::new();
    let path = f.write(
        "inbox.org",
        "\u{feff}* Inbox\r\nkeep\r\n* Other\r\nuntouched\r\n",
    );
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let before = std::fs::read(&path).unwrap();
    let plan = observe(&buffers, &path);
    assert_eq!(plan.added, 1);
    let buffer = buffers.get_mut(id).unwrap();
    let edits = plan.insertion.replacements(buffer).unwrap();
    let change = buffer.prepare(&edits, &[], None).unwrap();
    buffer.publish(change, &mut []).unwrap();
    let text = buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string();
    assert!(
        text.contains(":FEEDSTATUS:\r\n((\"stable\" t nil))\r\n  :END:\r\n"),
        "{text}"
    );
    assert!(text.contains("** News 界\r\n[[https://example.test/news]]\r\n: #+BEGIN_SRC sh\r\n: * Untrusted heading\r\n: :END:\r\n"),"{text}");
    assert!(text.ends_with("* Other\r\nuntouched\r\n"));
    assert_eq!(buffer.history().len(), 1);
    assert!(buffer.has_bom());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    let again = observe(&buffers, &path);
    assert_eq!(again.added, 0);
    assert_eq!(again.skipped, 1);
    assert!(again.insertion.edits.is_empty());
}
// Existing reference handled records suppress duplicates; malformed or reader
// expressions reject without changing the canonical buffer or destination file.
#[test]
fn feed_status() {
    for (name, status, success, added) in [
        (
            "reference",
            format!("((\"stable\" t \"{}\"))", "a".repeat(40)),
            true,
            0,
        ),
        ("unhandled", "((\"stable\" nil nil))".into(), true, 1),
        ("reader", "(#.(run-tool))".into(), false, 0),
        (
            "duplicate",
            "((\"stable\" t nil)(\"stable\" t nil))".into(),
            false,
            0,
        ),
    ] {
        let f = Fixture::new();
        let source = format!("* Inbox\n:FEEDSTATUS:\n{status}\n:END:\n");
        let path = f.write("inbox.org", &source);
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.visit(&path).unwrap();
        let stamp = buffers.get(id).unwrap().stamp();
        let policy = Org::default();
        let cancel = Cancellation::default();
        let target = CaptureRead::new(&buffers, &path, &policy.limits)
            .unwrap()
            .run(&cancel)
            .unwrap();
        let result = Feed::parse(XML, None, &policy.limits, &cancel)
            .unwrap()
            .plan(target, &definition(), &policy, &cancel);
        assert_eq!(result.is_ok(), success, "{name}");
        if let Ok(plan) = result {
            assert_eq!(plan.added, added, "{name}");
        }
        assert_eq!(buffers.get(id).unwrap().stamp(), stamp);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), source);
    }
}
