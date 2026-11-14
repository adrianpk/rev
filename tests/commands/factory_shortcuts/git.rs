use super::*;
use crate::config_fixture as fixture;
use rev::buffer::{Stamp, edit::Point};
use rev::command::GitCommand;
use std::{
    fs,
    time::{Duration, Instant},
};

#[path = "../../git_integration/repository_fixture.rs"]
mod repository_fixture;
use repository_fixture::Repository;
#[path = "git/refs.rs"]
mod refs;

fn repository() -> Repository {
    let repository = Repository::new("sha256");
    for n in 0..3 {
        fs::write(repository.root.join("source"), format!("version {n}\n")).unwrap();
        repository.commit();
    }
    repository
}

fn editor(repository: &Repository, modal: bool, extra: &str, modified: bool) -> Session {
    let snapshot = repository.snapshot(&format!(
        "history_page_size=2\n[editor]\ninteraction_profile='{}'\n{extra}",
        if modal { "modal" } else { "non-modal" }
    ));
    let mut buffers = Buffers::with_settings(snapshot.clone()).unwrap();
    let id = buffers.visit(&repository.root.join("source")).unwrap();
    let view = View::with_settings(buffers.get(id).unwrap(), &snapshot);
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 120,
            height: 32,
        },
    )
    .unwrap();
    if modified {
        session.insert('!');
    }
    session
}

fn ready(session: &mut Session, mut condition: impl FnMut(&mut Session) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        session.expire_input(Instant::now());
        if condition(session) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "Git work did not settle: {}",
            session.echo()
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn frame(session: &mut Session) -> String {
    session
        .frame()
        .unwrap()
        .rows()
        .iter()
        .map(|r| r.text())
        .collect::<Vec<_>>()
        .join("\n")
}

fn selected(session: &mut Session) -> String {
    session
        .frame()
        .unwrap()
        .rows()
        .iter()
        .find(|r| r.text().starts_with('>'))
        .unwrap()
        .text()
        .to_owned()
}

fn history(session: &mut Session) {
    if session.git_observation().is_none() {
        keys(session, "C-x g");
        ready(session, |s| s.git_observation().is_some());
    }
    session.execute(Command::Git(GitCommand::Log));
    ready(session, |s| s.echo().contains("Git history inspected"));
}

fn invoke(session: &mut Session, route: &str) {
    if route.starts_with("git-") {
        keys(session, "M-x");
        session.handle(Event::Paste(route.into()));
        keys(session, "RET");
    } else {
        keys(session, route);
    }
}

fn object(repository: &Repository, revision: &str) -> String {
    String::from_utf8(repository.git(["rev-parse", revision]))
        .unwrap()
        .trim()
        .to_owned()
}

struct Baseline {
    head: Vec<u8>,
    index: Vec<u8>,
    disk: Vec<u8>,
    text: String,
    point: Point,
    stamp: Stamp,
    history: usize,
}

impl Baseline {
    fn new(repository: &Repository, session: &Session) -> Self {
        Self {
            head: repository.git(["rev-parse", "HEAD"]),
            index: repository.git(["ls-files", "--stage", "-z"]),
            disk: fs::read(repository.root.join("source")).unwrap(),
            text: text(session),
            point: session.view().point,
            stamp: session.buffer().unwrap().stamp(),
            history: session.buffer().unwrap().history().len(),
        }
    }
    fn repository(&self, repository: &Repository) {
        assert_eq!(repository.git(["rev-parse", "HEAD"]), self.head);
        assert_eq!(repository.git(["ls-files", "--stage", "-z"]), self.index);
        assert_eq!(fs::read(repository.root.join("source")).unwrap(), self.disk);
    }
    fn retained(&self, repository: &Repository, session: &Session) {
        self.repository(repository);
        assert_eq!(text(session), self.text);
        assert_eq!(session.view().point, self.point);
        assert_eq!(session.buffer().unwrap().stamp(), self.stamp);
        assert_eq!(session.buffer().unwrap().history().len(), self.history);
    }
}

// Retired keys cannot page or mutate. A reviews the selected ordered objects,
// keeping revision marks distinct from the range base; cancellation is inert.
#[test]
fn history_review() {
    for modal in [false, true] {
        let repository = repository();
        let mut session = editor(&repository, modal, "", false);
        let baseline = Baseline::new(&repository, &session);
        let head = object(&repository, "HEAD");
        let parent = object(&repository, "HEAD^");
        history(&mut session);
        for retired in ["P", "F", "f"] {
            keys(&mut session, retired);
            assert!(
                session.echo().contains("read-only"),
                "{retired}: {}",
                session.echo()
            );
            baseline.retained(&repository, &session);
        }
        keys(&mut session, "a Down a * A");
        ready(&mut session, |s| {
            s.echo().contains("exact ordered revisions") && s.echo().starts_with("Confirm")
        });
        let review = frame(&mut session);
        assert!(review.contains("Ordered revision review"));
        assert!(
            review.contains(&format!("1: {head}")) && review.contains(&format!("2: {parent}")),
            "{review}"
        );
        keys(&mut session, "RET c a c f + P F");
        baseline.retained(&repository, &session);
        assert!(frame(&mut session).contains("Ordered revision review"));
        keys(&mut session, "n");
        baseline.retained(&repository, &session);
    }
}

// c a always prepares checked HEAD; c f captures the older selected object.
// The first explicit submit opens review; plain keys cannot submit the child.
#[test]
fn composition_review() {
    for modal in [false, true] {
        let repository = repository();
        fs::write(repository.root.join("source"), "staged\n").unwrap();
        repository.git(["add", "source"]);
        for (route, label, target) in [
            ("c a", "Amend exact HEAD", "HEAD"),
            ("c f", "Fixup exact target", "HEAD^"),
        ] {
            let mut session = editor(&repository, modal, "", true);
            let baseline = Baseline::new(&repository, &session);
            history(&mut session);
            keys(&mut session, "Down");
            let selected = object(&repository, target);
            keys(&mut session, route);
            ready(&mut session, |s| s.echo().contains(label));
            assert!(!session.git_visible());
            assert!(session.echo().contains(&selected));
            keys(&mut session, "C-Enter");
            assert!(session.git_visible());
            let review = frame(&mut session);
            assert!(
                review.contains(label) && review.contains(&selected),
                "{review}"
            );
            keys(&mut session, "RET c a c f + A");
            baseline.repository(&repository);
            assert!(frame(&mut session).contains("Git Commit Review"));
            keys(&mut session, "C-g");
            baseline.retained(&repository, &session);
        }
    }
}

// Paging + retains its immutable root, selected full object and range/ordered
// marks across finite pages, without touching source, index or history.
#[test]
fn history_pages() {
    for modal in [false, true] {
        let repository = repository();
        let mut session = editor(&repository, modal, "", true);
        let baseline = Baseline::new(&repository, &session);
        history(&mut session);
        assert!(frame(&mut session).contains("more ancestry: +"));
        keys(&mut session, "* a");
        let before = selected(&mut session);
        keys(&mut session, "+");
        ready(&mut session, |s| frame(s).contains("3 revisions / 2 pages"));
        assert_eq!(selected(&mut session), before);
        let display = frame(&mut session);
        assert!(display.contains("1 ordered"));
        assert!(display.contains(&object(&repository, "HEAD")));
        assert!(display.contains("complete ancestry"));
        assert!(display.contains("c a amend") && display.contains("c f fixup"));
        baseline.retained(&repository, &session);
    }
}

// Canonical names and explicit old assignments keep their native effects.
// Defaults omitted from that legacy map still include the new c prefix family.
#[test]
fn old_access() {
    let overrides = "[keys.git_history]\n'A'='git-commit-amend'\n'F'='git-commit-fixup'\n'P'='git-cherry-pick'\n'f'='git-log-next-page'\n";
    for modal in [false, true] {
        let repository = repository();
        for routes in [
            ["A", "F", "P", "f"],
            [
                "git-commit-amend",
                "git-commit-fixup",
                "git-cherry-pick",
                "git-log-next-page",
            ],
        ] {
            let mut session = editor(&repository, modal, overrides, false);
            let baseline = Baseline::new(&repository, &session);
            history(&mut session);
            for (route, label) in [
                (routes[0], "Amend exact HEAD"),
                (routes[1], "Fixup exact target"),
            ] {
                invoke(&mut session, route);
                ready(&mut session, |s| s.echo().contains(label));
                assert!(!session.git_visible());
                keys(&mut session, "C-g");
                baseline.retained(&repository, &session);
                session.execute(Command::Git(GitCommand::Log));
                ready(&mut session, |s| s.echo().contains("Git history inspected"));
            }
            invoke(&mut session, routes[2]);
            ready(&mut session, |s| {
                s.echo().starts_with("Confirm") && s.echo().contains("exact ordered revisions")
            });
            assert!(frame(&mut session).contains(&object(&repository, "HEAD")));
            keys(&mut session, "n");
            invoke(&mut session, routes[3]);
            ready(&mut session, |s| frame(s).contains("3 revisions / 2 pages"));
            keys(&mut session, "c a");
            ready(&mut session, |s| s.echo().contains("Amend exact HEAD"));
            keys(&mut session, "C-g");
            baseline.retained(&repository, &session);
        }
    }
}
