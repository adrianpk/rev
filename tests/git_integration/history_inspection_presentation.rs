use super::*;
use crate::status::presentation::{editor, entry, key, plain, render, selected, text, wait};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use rev::command::{Command, GitCommand, Motion};

// Range-base selection and ordered revision roles stay independent through
// navigation and detail return. Both profiles use the approved native chords.
#[test]
fn range_and_ordered_marks() {
    for profile in ["non-modal", "modal"] {
        let r = Repository::new("sha1");
        for n in 0..4 {
            fs::write(r.root.join("source"), format!("value {n}\n")).unwrap();
            r.commit();
        }
        let mut owned = editor(&r, profile, "", false);
        let s = &mut owned.0;
        s.load(&r.root.join("source")).unwrap();
        entry(s);
        wait(s, |s| s.git_observation().is_some());
        plain(s, 'h');
        wait(s, |s| s.echo().contains("Git history inspected"));
        let head = object(&r, "HEAD");
        let one = object(&r, "HEAD~1");
        let two = object(&r, "HEAD~2");
        plain(s, '*');
        plain(s, 'n');
        plain(s, 'a');
        plain(s, 'n');
        plain(s, 'a');
        plain(s, '*');
        let frame = render(s).join("\n");
        assert!(frame.contains(&format!("Base: {two}")), "{frame}");
        assert!(frame.contains(&format!("[1]{one}")), "{frame}");
        assert!(frame.contains(&format!("[2]{two}")), "{frame}");
        s.execute(Command::Motion(Motion::BufferStart));
        key(s, KeyCode::Enter, KeyModifiers::NONE);
        wait(s, |s| s.echo().contains("Git history inspected"));
        let frame = render(s).join("\n");
        assert!(
            frame.contains(&format!("Git Range endpoint: {head}")),
            "{frame}"
        );
        assert!(frame.contains(&format!("Range base: {two}")), "{frame}");
        plain(s, 'q');
        plain(s, 'n');
        plain(s, 'U');
        let frame = render(s).join("\n");
        assert!(frame.contains(&format!("[1]{two}")));
        assert!(!frame.contains(&format!("[1]{one}")));
        assert!(frame.contains(&format!("Base: {two}")));
        s.execute(Command::Git(GitCommand::UnmarkAll));
        key(s, KeyCode::Enter, KeyModifiers::NONE);
        assert!(s.echo().contains("range base"), "{}", s.echo());
        assert_eq!(object(&r, "HEAD"), head);
    }
}

// Reflog and blame share inspection navigation, while historical content stays
// outside the editable source owner. Named current-source visits preserve dirty
// text and the original buffer identity instead of checking out old bytes.
#[test]
fn native_inspections() {
    for (profile, suffix) in [("non-modal", 'b'), ("modal", 'B')] {
        let r = Repository::new("sha256");
        let path = r.root.join("source.txt");
        fs::write(&path, "first\nsecond\n").unwrap();
        r.commit();
        fs::write(&path, "first\nchanged\n").unwrap();
        r.commit();
        let mut owned = editor(&r, profile, "", false);
        let s = &mut owned.0;
        s.load(&path).unwrap();
        if profile == "modal" {
            plain(s, 'i');
        }
        plain(s, 'x');
        let before = text(s);
        let id = s.buffer().unwrap().id();
        let head = object(&r, "HEAD");
        let index = r.git(["ls-files", "--stage", "-z"]);
        entry(s);
        wait(s, |s| s.git_observation().is_some());
        plain(s, 'B');
        plain(s, suffix);
        wait(s, |s| s.echo().contains("Git history inspected"));
        assert!(render(s).join("\n").contains("Git Blame:"));
        key(s, KeyCode::Enter, KeyModifiers::NONE);
        wait(s, |s| s.echo().contains("Git history inspected"));
        let frame = render(s).join("\n");
        assert!(frame.contains("Git Historical File:"), "{frame}");
        assert!(frame.contains("second"), "{frame}");
        s.handle(Event::Paste("forbidden".into()));
        s.execute(Command::SaveBuffer);
        assert_eq!(text(s), before);
        plain(s, 'q');
        plain(s, 'v');
        wait(s, |s| s.echo().contains("Git history inspected"));
        assert!(render(s).join("\n").contains("Git Revision:"));
        plain(s, 'q');
        assert!(render(s).join("\n").contains("Git Blame:"));
        s.execute(Command::Git(GitCommand::VisitFile));
        assert!(!s.git_visible());
        assert_eq!(s.buffer().unwrap().id(), id);
        assert_eq!(text(s), before);
        entry(s);
        wait(s, |s| s.git_observation().is_some());
        plain(s, 'l');
        plain(s, 'r');
        wait(s, |s| s.echo().contains("Git history inspected"));
        let selected = selected(s);
        assert!(selected.contains(&head), "{selected}");
        plain(s, 'v');
        wait(s, |s| s.echo().contains("Git history inspected"));
        plain(s, 'q');
        assert!(render(s).join("\n").contains("Git Reflog:"));
        plain(s, 'g');
        wait(s, |s| s.echo().contains("Git history inspected"));
        assert_eq!(crate::status::presentation::selected(s), selected);
        assert_eq!(r.git(["ls-files", "--stage", "-z"]), index);
        assert_eq!(object(&r, "HEAD"), head);
        assert_eq!(fs::read_to_string(path).unwrap(), "first\nchanged\n");
        assert_eq!(text(s), before);
    }
}
