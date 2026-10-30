use super::execution::configured;
use super::*;
use rev::file::Publication;
const XML: &str =
    "<rss version='2.0'><channel><item><guid>one</guid><title>New</title></item></channel></rss>";
const POLICY: &str =
    "[[org.feeds.sources]]\nname='News'\nsource='news.xml'\nfile='inbox.org'\nheadline='Inbox'\n";
fn script(f: &Fixture, body: &str) {
    f.write(
        "news.xml",
        &format!("{body}\ncat <<'REV_FEED'\n{XML}\nREV_FEED\n"),
    );
}
// Independent declared feeds keep separate outcomes: a rejected first feed
// cannot hide or prevent a successful second canonical save.
#[test]
fn feed_batch() {
    let policy = format!(
        "{POLICY}\n{}",
        POLICY
            .replace("News", "Other")
            .replace("news.xml", "good.xml")
    );
    let (f, mut s) = configured("* Selected\n", &policy);
    f.write("news.xml", "<!DOCTYPE rss><rss/>");
    f.write("good.xml", XML);
    action(&mut s, C::FeedUpdateAll);
    assert_eq!(s.feed_results().len(), 2);
    assert_eq!(
        s.feed_results()[0].publication,
        Some(Publication::NotPublished)
    );
    assert_eq!(s.feed_results()[1].publication, Some(Publication::Complete));
    assert_eq!(s.feed_results()[1].added, 1);
    assert_eq!(text(&s), "* Selected\n");
    assert!(
        std::fs::read_to_string(f.0.join("inbox.org"))
            .unwrap()
            .contains("** New")
    );
}
// A read-only retained target rejects before launching transport. A disk change
// by a real transport is detected at publication and is never overwritten.
#[test]
fn feed_target() {
    for mode in ["readonly", "stale"] {
        let policy = format!("{POLICY}transport=['/bin/sh','{{target}}']");
        let policy = policy.replace("inbox.org", "source.org");
        let (f, mut s) =
            super::execution::configured_readonly("* Inbox\nold\n", &policy, mode == "readonly");
        let path = f.0.join("source.org");
        let body = format!(
            "printf ran > '{}'\nprintf '* Inbox\\nexternal\\n' > '{}'",
            f.0.join("ran").display(),
            path.display()
        );
        script(&f, &body);
        let before = text(&s);
        let stamp = s.buffer().unwrap().stamp();
        action(&mut s, C::FeedUpdateAll);
        argument(&mut s, "yes");
        let out = &s.feed_results()[0];
        assert_eq!(
            out.publication,
            Some(Publication::NotPublished),
            "{mode}: {}",
            out.detail
        );
        assert_eq!(text(&s), before);
        assert_eq!(s.buffer().unwrap().stamp(), stamp);
        assert_eq!(s.buffer().unwrap().history().len(), 0);
        assert_eq!(f.0.join("ran").exists(), mode == "stale");
        assert_eq!(
            std::fs::read_to_string(path).unwrap(),
            if mode == "stale" {
                "* Inbox\nexternal\n"
            } else {
                "* Inbox\nold\n"
            }
        );
    }
}
// Failed exit, malformed bytes and both output ceilings reject complete updates.
// No truncated XML or stderr-only failure can publish a prefix of the feed.
#[test]
fn feed_limits() {
    for (name, body, global, expected) in [
        ("exit", "printf failure >&2; exit 7", "", "failed"),
        ("utf8", "printf '\\377'; exit", "", "UTF-8"),
        (
            "output",
            "printf abc; printf def >&2; exit",
            "org.limits.job_output_bytes=5",
            "output limit",
        ),
        (
            "feed",
            "printf abcdef; exit",
            "org.limits.feed_bytes=5",
            "output limit",
        ),
    ] {
        let policy = format!("{global}\n{POLICY}transport=['/bin/sh','{{target}}']");
        let (f, mut s) = configured("* Selected\n", &policy);
        let path = f.write("inbox.org", "* Inbox\nold\n");
        script(&f, body);
        action(&mut s, C::FeedUpdateAll);
        argument(&mut s, "yes");
        let out = &s.feed_results()[0];
        assert_eq!(out.publication, Some(Publication::NotPublished), "{name}");
        assert!(out.detail.contains(expected), "{name}: {}", out.detail);
        assert_eq!(std::fs::read_to_string(path).unwrap(), "* Inbox\nold\n");
        assert_eq!(text(&s), "* Selected\n");
    }
}
fn started(s: &mut Session, path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        s.frame().unwrap();
        if std::fs::read_to_string(path).is_ok_and(|v| !v.is_empty()) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "owned feed transport did not start: {}",
            s.echo()
        );
        s.expire_input(Instant::now() + Duration::from_millis(10));
        std::thread::yield_now();
    }
}
// Cancel, timeout and drop reap the owned leader, stop descendants and remove
// private HOME. Active input cannot edit source and idle never polls a feed.
#[test]
fn feed_lifetime() {
    for mode in ["cancel", "timeout", "drop"] {
        let policy =
            format!("org.limits.job_seconds=1\n{POLICY}transport=['/bin/sh','{{target}}']");
        let (f, mut s) = configured("* Selected\n", &policy);
        let home = f.0.join("home");
        let leader = f.0.join("leader");
        let child = f.0.join("child");
        script(
            &f,
            &format!(
                "echo $$ > '{}'; sleep 30 & echo $! > '{}'; printf '%s' \"$HOME\" > '{}'; wait",
                leader.display(),
                child.display(),
                home.display()
            ),
        );
        action(&mut s, C::FeedUpdateAll);
        s.handle(Event::Paste("yes".into()));
        s.handle(Event::Key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE,
        )));
        started(&mut s, &home);
        let home = std::fs::read_to_string(home).unwrap();
        let leader = std::fs::read_to_string(leader).unwrap();
        let child = std::fs::read_to_string(child).unwrap();
        if mode == "drop" {
            drop(s);
        } else {
            if mode == "cancel" {
                s.handle(Event::Paste("blocked".into()));
                s.execute(Command::DeleteChar);
                cancel(&mut s);
            }
            settle(&mut s);
            assert_eq!(
                s.feed_results()[0].publication,
                Some(Publication::NotPublished)
            );
            assert_eq!(text(&s), "* Selected\n");
            assert_eq!(s.remaining_input(Instant::now()), None);
        }
        assert!(!Path::new(&home).exists(), "{mode}: private HOME survived");
        assert!(
            !Path::new(&format!("/proc/{}", leader.trim())).exists(),
            "{mode}: leader not reaped"
        );
        assert!(
            std::fs::read_to_string(format!("/proc/{}/stat", child.trim())).map_or(true, |v| v
                .split_once(") ")
                .unwrap()
                .1
                .starts_with('Z')),
            "{mode}: descendant running"
        );
        assert!(!f.0.join("inbox.org").exists());
    }
}
// Inbox creation at an unterminated EOF navigates to the new heading rather
// than its separator. Creating that heading remains one unsaved undo unit.
#[test]
fn feed_inbox_eof() {
    let (f, mut s) = configured("* Selected\n", POLICY);
    let path = f.write("inbox.org", "* Previous\r\nkeep");
    action(&mut s, C::FeedGotoInbox);
    argument(&mut s, "News");
    assert_eq!(text(&s), "* Previous\r\nkeep\r\n* Inbox\r\n");
    assert_eq!(s.view().point.position.0, "* Previous\r\nkeep\r\n".len());
    assert_eq!(std::fs::read_to_string(path).unwrap(), "* Previous\r\nkeep");
    s.execute(Command::History(rev::command::History::Undo));
    settle(&mut s);
    assert_eq!(text(&s), "* Previous\r\nkeep");
}
