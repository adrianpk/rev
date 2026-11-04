use crate::{
    status::presentation::{editor, entry, key, plain, render, selected, text, wait},
    support::Repository,
};
use crossterm::event::{KeyCode, KeyModifiers};
use rev::command::{Command, Motion, Split};
use std::fs;

// The same s/u keys in both profiles stage disk bytes through checked native
// preflight, retain a modified source owner and return to the surviving layout.
#[test]
fn native_paths() {
    for profile in ["non-modal", "modal"] {
        let repository = Repository::new("sha1");
        let path = repository.root.join("source");
        fs::write(&path, "original\n").unwrap();
        repository.commit();
        let mut owned = editor(&repository, profile, "", false);
        let editor = &mut owned.0;
        editor.load(&path).unwrap();
        if profile == "modal" {
            plain(editor, 'i');
        }
        plain(editor, 'x');
        editor.execute(Command::Split(Split::Below));
        let before = text(editor);
        let point = editor.view().point;
        let id = editor.buffer().unwrap().id();
        let views: Vec<_> = editor.windows().iter().map(|(id, _)| id).collect();
        fs::write(&path, "disk change\n").unwrap();
        entry(editor);
        wait(editor, |s| s.git_observation().is_some());
        plain(editor, 'n');
        assert!(selected(editor).contains("source"));
        plain(editor, 's');
        wait(editor, |s| s.echo().contains("Index updated and verified"));
        assert_eq!(repository.git(["show", ":source"]), b"disk change\n");
        assert_eq!(text(editor), before);
        plain(editor, '$');
        assert!(render(editor).join("\n").contains("Stage index"));
        plain(editor, 'q');
        editor.execute(Command::Motion(Motion::BufferEnd));
        plain(editor, 'u');
        wait(editor, |s| s.echo().contains("Index updated and verified"));
        assert_eq!(repository.git(["show", ":source"]), b"original\n");
        assert_eq!(fs::read(&path).unwrap(), b"disk change\n");
        plain(editor, 'q');
        assert!(!editor.git_visible());
        assert_eq!(editor.buffer().unwrap().id(), id);
        assert!(editor.buffer().unwrap().is_modified());
        assert_eq!(text(editor), before);
        assert_eq!(editor.view().point, point);
        assert_eq!(
            editor
                .windows()
                .iter()
                .map(|(id, _)| id)
                .collect::<Vec<_>>(),
            views
        );
    }
}

// Selecting a hunk through native expansion changes only that exact index
// span. A second action follows a fresh staged comparison, never a stale row.
#[test]
fn native_hunks() {
    for profile in ["non-modal", "modal"] {
        let repository = Repository::new("sha1");
        let path = repository.root.join("source");
        let base: Vec<_> = (1..=48).map(|n| format!("line {n:02}\n")).collect();
        fs::write(&path, base.concat()).unwrap();
        repository.commit();
        let mut content = base.clone();
        content[3] = "first change\n".into();
        content[39] = "second change\n".into();
        fs::write(&path, content.concat()).unwrap();
        let mut owned = editor(&repository, profile, "", false);
        let editor = &mut owned.0;
        editor.load(&path).unwrap();
        entry(editor);
        wait(editor, |s| s.git_observation().is_some());
        plain(editor, 'n');
        key(editor, KeyCode::Tab, KeyModifiers::NONE);
        wait(editor, |s| {
            s.echo().contains("comparison retained as read-only")
        });
        plain(editor, 'n');
        assert!(selected(editor).contains("@@"));
        plain(editor, 's');
        wait(editor, |s| s.echo().contains("Index updated and verified"));
        let mut expected = base.clone();
        expected[3] = "first change\n".into();
        assert_eq!(
            repository.git(["show", ":source"]),
            expected.concat().as_bytes()
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), content.concat());
        // Old immutable hunk evidence cannot authorize a second changing job.
        plain(editor, 's');
        assert!(editor.echo().contains("hunk is stale"));
        editor.execute(Command::Motion(Motion::BufferEnd));
        key(editor, KeyCode::Tab, KeyModifiers::NONE);
        wait(editor, |s| {
            s.echo().contains("comparison retained as read-only")
        });
        plain(editor, 'n');
        assert!(selected(editor).contains("@@"));
        plain(editor, 'u');
        wait(editor, |s| s.echo().contains("Index updated and verified"));
        assert_eq!(
            repository.git(["show", ":source"]),
            base.concat().as_bytes()
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), content.concat());
    }
}
