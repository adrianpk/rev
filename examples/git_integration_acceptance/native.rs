use crate::{
    repository::{Repository, finish},
    support::*,
};
use rev::git::{Completeness, Group};
use std::fs;

// The same installed-Git fixture is used by the integration target and this
// executable. Assertions inspect actual stored blobs, index and working bytes.
pub(crate) fn run() -> Result<()> {
    println!(
        "MEASURE native entry threads={}",
        crate::metrics::threads()?
    );
    for format in ["sha1", "sha256"] {
        let repository = Repository::new(format);
        fs::write(repository.root.join("source.txt"), "base\n")?;
        repository.commit();
        fs::write(repository.root.join("source.txt"), "index\n")?;
        repository.git(["add", "--", "source.txt"]);
        fs::write(repository.root.join("source.txt"), "working\n")?;
        let mut owner = repository.owner("");
        assert!(repository.open(&mut owner, &repository.root).published);
        let context = &owner.contexts()[0];
        let status = context.observation.status.as_ref().unwrap();
        assert_eq!(status.completeness, Completeness::Complete);
        assert_eq!(status.paths.len(), 1);
        assert!(status.paths.iter().any(|p| p.in_group(Group::Staged)));
        assert!(status.paths.iter().any(|p| p.in_group(Group::Unstaged)));
        let id = context.id;
        owner.refresh(id)?;
        assert!(finish(&mut owner).published);
        assert_eq!(repository.git(["show", "HEAD:source.txt"]), b"base\n");
        assert_eq!(repository.git(["show", ":source.txt"]), b"index\n");
        assert_eq!(fs::read(repository.root.join("source.txt"))?, b"working\n");
        for profile in ["non-modal", "modal"] {
            let mut owned = editor(&repository, profile)?;
            let editor = &mut owned.0;
            key(
                editor,
                crossterm::event::KeyCode::Char('x'),
                crossterm::event::KeyModifiers::CONTROL,
            );
            key(
                editor,
                crossterm::event::KeyCode::Char('g'),
                crossterm::event::KeyModifiers::NONE,
            );
            wait(editor, |s| s.git_observation().is_some());
            assert!(editor.git_visible());
            named(editor, "git-quit");
            assert!(!editor.git_visible());
            assert_eq!(text(editor), "working\n");
        }
        println!(
            "PASS native shared fixture format={format}: exact HEAD/index/working bytes, both-profile entry and restoration"
        );
    }
    Ok(())
}
