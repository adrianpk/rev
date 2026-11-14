use super::*;

fn select(session: &mut Session, needle: &str) {
    keys(session, "M-<");
    for _ in 0..16 {
        if selected(session).contains(needle) {
            return;
        }
        keys(session, "Down");
    }
    panic!("Missing fixture ref {needle}: {}", frame(session));
}

// New f fetches only the selected local fixture remote; retained aliases share
// its source admission. P/p/M p retain exact upstream review and cancellation.
#[test]
fn refs_routes() {
    for modal in [false, true] {
        let repository = repository();
        let remote = repository.fixture.0.join("origin.git");
        repository.git([
            "clone",
            "--bare",
            repository.root.to_str().unwrap(),
            remote.to_str().unwrap(),
        ]);
        repository.git(["remote", "add", "origin", remote.to_str().unwrap()]);
        let head = object(&repository, "HEAD");
        repository.git(["update-ref", "refs/remotes/origin/main", &head]);
        repository.git(["branch", "--set-upstream-to=origin/main", "main"]);
        let mut session = editor(&repository, modal, "", true);
        let baseline = Baseline::new(&repository, &session);
        keys(&mut session, "C-x g");
        ready(&mut session, |s| s.git_observation().is_some());
        keys(&mut session, "b");
        ready(&mut session, |s| {
            s.echo().contains("Refs and Status checked")
        });
        for route in ["f", "&", "M f"] {
            select(&mut session, "refs/heads/main");
            keys(&mut session, route);
            assert!(
                session
                    .echo()
                    .contains("Select one exact configured remote")
            );
        }
        select(&mut session, "remote origin");
        keys(&mut session, "f");
        ready(&mut session, |s| s.echo().contains("Fetch completed"));
        assert_eq!(object(&repository, "refs/remotes/origin/main"), head);
        for route in ["P", "p", "M p"] {
            select(&mut session, "refs/heads/main");
            keys(&mut session, route);
            ready(&mut session, |s| {
                s.echo().starts_with("Push refs/heads/main at")
            });
            let review = frame(&mut session);
            assert!(review.contains("Single upstream branch push review"));
            assert!(review.contains(&head) && review.contains("origin"));
            keys(&mut session, "RET f P c a c f +");
            keys(&mut session, "A");
            baseline.retained(&repository, &session);
            assert!(frame(&mut session).contains("Single upstream branch push review"));
            keys(&mut session, "C-g");
        }
        assert_eq!(
            repository.git(["--git-dir", remote.to_str().unwrap(), "rev-parse", "main"]),
            baseline.head
        );
        baseline.retained(&repository, &session);
    }
}

// Incomplete History c cannot survive cancellation or a role transition.
// Status c still opens literal composition; a following a is message text.
#[test]
fn prefix_ownership() {
    for modal in [false, true] {
        let repository = repository();
        let mut session = editor(&repository, modal, "", true);
        let baseline = Baseline::new(&repository, &session);
        history(&mut session);
        keys(&mut session, "c");
        assert!(session.git_visible());
        baseline.retained(&repository, &session);
        keys(&mut session, "C-g");
        baseline.retained(&repository, &session);
        history(&mut session);
        keys(&mut session, "c");
        session.execute(Command::Git(GitCommand::Quit));
        keys(&mut session, "c a");
        assert!(!session.git_visible());
        assert_eq!(text(&session), "a");
        keys(&mut session, "C-g");
        baseline.retained(&repository, &session);
    }
}
