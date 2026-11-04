use super::*;
use crate::status::presentation::{editor, entry, key, plain, render, selected, text, wait};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use rev::command::{Command, GitCommand};

// Both native profiles route b/y and branch keys through one read-only owner,
// require explicit decisions and retain source layout/text on return.
#[test]
fn native_branches() {
    for profile in ["non-modal", "modal"] {
        let repository = Repository::new("sha256");
        let mut owned = editor(&repository, profile, "", true);
        let editor = &mut owned.0;
        let source = text(editor);
        let point = editor.view().point;
        entry(editor);
        wait(editor, |s| s.git_observation().is_some());
        plain(editor, 'b');
        wait(editor, |s| s.echo().contains("Refs and Status checked"));
        assert!(selected(editor).contains("refs/heads/main"));
        plain(editor, '?');
        assert!(render(editor).join("\n").contains("git-branch-create"));
        plain(editor, 'q');
        plain(editor, 'n');
        editor.handle(Event::Paste("topic".into()));
        key(editor, KeyCode::Enter, KeyModifiers::NONE);
        let target = repository.git(["rev-parse", "HEAD"]);
        let frame = render(editor).join("\n");
        assert!(
            frame.contains(String::from_utf8_lossy(&target).trim()),
            "{frame}"
        );
        assert!(frame.contains("Create local branch:"), "{frame}");
        key(editor, KeyCode::PageDown, KeyModifiers::NONE);
        key(editor, KeyCode::PageUp, KeyModifiers::NONE);
        assert!(
            editor.echo().contains("Create local branch topic at"),
            "{}",
            editor.echo()
        );
        key(editor, KeyCode::Enter, KeyModifiers::NONE);
        assert!(repository.git(["branch", "--list", "topic"]).is_empty());
        plain(editor, 'y');
        wait(editor, |s| s.echo().contains("Refs and Status checked"));
        assert!(!repository.git(["branch", "--list", "topic"]).is_empty());
        key(editor, KeyCode::Down, KeyModifiers::NONE);
        assert!(selected(editor).contains("refs/heads/topic"));
        plain(editor, 'S');
        assert!(editor.echo().contains("Switch to refs/heads/topic"));
        plain(editor, 'y');
        wait(editor, |s| s.echo().contains("Refs and Status checked"));
        assert_eq!(
            repository.git(["symbolic-ref", "HEAD"]),
            b"refs/heads/topic\n"
        );
        plain(editor, 'R');
        editor.handle(Event::Paste("renamed".into()));
        key(editor, KeyCode::Enter, KeyModifiers::NONE);
        plain(editor, 'y');
        wait(editor, |s| s.echo().contains("Refs and Status checked"));
        assert_eq!(
            repository.git(["symbolic-ref", "HEAD"]),
            b"refs/heads/renamed\n"
        );
        plain(editor, 'D');
        plain(editor, 'n');
        assert!(editor.echo().contains("canceled"));
        plain(editor, 'q');
        assert!(render(editor).join("\n").contains("Git:"));
        plain(editor, 'y');
        wait(editor, |s| s.echo().contains("Refs and Status checked"));
        editor.execute(Command::SaveBuffer);
        editor.insert('!');
        assert_eq!(text(editor), source);
        plain(editor, 'q');
        plain(editor, 'q');
        assert!(!editor.git_visible());
        assert_eq!(text(editor), source);
        assert_eq!(editor.view().point, point);
    }
}

// Unsaved repository sources reject switch before review; an external ref
// change while confirmation is open cannot silently select another target.
#[test]
fn native_guards() {
    let repository = Repository::new("sha1");
    let mut owned = editor(&repository, "non-modal", "", true);
    let editor = &mut owned.0;
    repository.git(["branch", "topic"]);
    plain(editor, 'x');
    let source = text(editor);
    entry(editor);
    wait(editor, |s| s.git_observation().is_some());
    editor.execute(Command::Git(GitCommand::Refs));
    wait(editor, |s| s.echo().contains("Refs and Status checked"));
    key(editor, KeyCode::Down, KeyModifiers::NONE);
    plain(editor, 'S');
    assert!(
        editor.echo().contains("modified repository-backed buffer"),
        "{}",
        editor.echo()
    );
    assert_eq!(
        repository.git(["symbolic-ref", "HEAD"]),
        b"refs/heads/main\n"
    );
    assert_eq!(text(editor), source);
    plain(editor, 'D');
    repository.git(["branch", "external"]);
    plain(editor, 'y');
    wait(editor, |s| s.echo().contains("Refs operation failed"));
    assert!(!repository.git(["show-ref", "refs/heads/topic"]).is_empty());
    plain(editor, 'g');
    wait(editor, |s| s.echo().contains("Refs and Status checked"));
    assert!(render(editor).join("\n").contains("refs/heads/external"));
}

// Full object facts and configured remotes remain reachable after long ref
// lists. Detail is read-only, and stale list coordinates cannot select it.
#[test]
fn details_and_remotes() {
    use rev::command::Motion;
    let repository = Repository::new("sha256");
    let mut owned = editor(&repository, "non-modal", "", true);
    let editor = &mut owned.0;
    for i in 0..24 {
        repository.git(["branch", &format!("branch-{i:02}")]);
    }
    repository.git([
        "config",
        "remote.first.url",
        "https://user:password@example.invalid/one",
    ]);
    repository.git(["config", "remote.last.url", "https://example.invalid/two"]);
    let head = repository.git(["rev-parse", "HEAD"]);
    entry(editor);
    wait(editor, |s| s.git_observation().is_some());
    plain(editor, 'b');
    wait(editor, |s| s.echo().contains("Refs and Status checked"));
    key(editor, KeyCode::Tab, KeyModifiers::NONE);
    let frame = render(editor).join("\n");
    assert!(frame.contains("Git Ref Detail"), "{frame}");
    assert!(
        frame.contains(String::from_utf8_lossy(&head).trim()),
        "{frame}"
    );
    key(editor, KeyCode::PageDown, KeyModifiers::NONE);
    plain(editor, 'q');
    editor.execute(Command::Motion(Motion::BufferEnd));
    assert!(selected(editor).contains("remote last"));
    key(editor, KeyCode::Tab, KeyModifiers::NONE);
    let frame = render(editor).join("\n");
    assert!(frame.contains("Configured remote name"));
    assert!(frame.contains("last"));
    assert!(!frame.contains("password"));
    plain(editor, 'q');
    editor.execute(Command::Motion(Motion::BufferStart));
    render(editor);
    plain(editor, 't');
    editor.handle(Event::Paste("cancelled".into()));
    key(editor, KeyCode::Enter, KeyModifiers::NONE);
    key(editor, KeyCode::Enter, KeyModifiers::NONE);
    key(editor, KeyCode::Enter, KeyModifiers::NONE);
    wait(editor, |s| {
        s.echo().contains("Create local lightweight tag")
    });
    render(editor);
    plain(editor, 'n');
    crate::status::presentation::click(editor, 3);
    assert!(editor.echo().contains("redraw"), "{}", editor.echo());
    assert!(repository.git(["tag", "--list"]).is_empty());
}

// Owned branch changes refresh retained history through the normal worker and
// reconcile changed source through the existing automatic reload policy.
#[test]
fn refreshed_history_and_source() {
    let repository = Repository::new("sha1");
    let mut owned = editor(&repository, "non-modal", "", true);
    let editor = &mut owned.0;
    repository.git(["switch", "--create", "other"]);
    fs::write(repository.root.join("source.txt"), "Other branch source\n").unwrap();
    repository.commit();
    let other = repository.git(["rev-parse", "HEAD"]);
    repository.git(["switch", "main"]);
    entry(editor);
    wait(editor, |s| s.git_observation().is_some());
    plain(editor, 'h');
    wait(editor, |s| s.echo().contains("Git history inspected"));
    plain(editor, 'q');
    plain(editor, 'b');
    wait(editor, |s| s.echo().contains("Git history inspected"));
    key(editor, KeyCode::Down, KeyModifiers::NONE);
    plain(editor, 'S');
    plain(editor, 'y');
    wait(editor, |s| s.echo().contains("Git history inspected"));
    assert_eq!(repository.git(["rev-parse", "HEAD"]), other);
    wait(editor, |s| text(s) == "Other branch source\n");
    plain(editor, 'q');
    plain(editor, 'h');
    wait(editor, |s| s.echo().contains("Git history inspected"));
    let frame = render(editor).join("\n");
    assert!(
        frame.contains(String::from_utf8_lossy(&other).trim()),
        "{frame}"
    );
    plain(editor, 'q');
    plain(editor, 'q');
    assert_eq!(text(editor), "Other branch source\n");
}
