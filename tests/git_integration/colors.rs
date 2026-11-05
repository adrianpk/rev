use super::*;
use rev::config::{self, Rgb};

#[path = "../support/surface.rs"]
mod surface;

fn tones(frame: &rev::display::Frame, expected: &[(&str, Rgb)]) {
    let cells = surface::draw(frame);
    for &(label, foreground) in expected {
        let (row, text) = frame
            .rows()
            .iter()
            .enumerate()
            .find(|(_, row)| row.text().contains(label))
            .unwrap();
        let column = text.text().find(label).unwrap();
        assert_eq!(
            cells[&(row, column)].foreground,
            Some(foreground),
            "{label}"
        );
    }
}

// Real native rows retain their meanings in both profiles, including selected
// hunks, per-role overrides and source resembling patch delimiters in new files.
#[test]
fn semantic_colors() {
    for profile in ["non-modal", "modal"] {
        let repository = Repository::new("sha1");
        let extra = if profile == "modal" {
            "[palette.git]\nadded='#12ab34'\nremoved='#cd3456'"
        } else {
            ""
        };
        let added = if profile == "modal" {
            Rgb(0x12, 0xab, 0x34)
        } else {
            Rgb(0x9e, 0xce, 0x6a)
        };
        let removed = if profile == "modal" {
            Rgb(0xcd, 0x34, 0x56)
        } else {
            Rgb(0xf7, 0x76, 0x8e)
        };
        let mut owned = editor(&repository, profile, extra, true);
        let editor = &mut owned.0;
        fs::write(
            repository.root.join("source.txt"),
            "common before\nSOURCE_BEFORE\ncommon after\n",
        )
        .unwrap();
        fs::write(repository.root.join("staged.txt"), "before\n").unwrap();
        repository.commit();
        fs::write(repository.root.join("staged.txt"), "after\n").unwrap();
        repository.git(["add", "--", "staged.txt"]);
        fs::write(
            repository.root.join("source.txt"),
            "common before\nSOURCE_AFTER\ncommon after\n",
        )
        .unwrap();
        fs::write(
            repository.root.join("untracked.txt"),
            "+literal\n-literal\n@@ literal\n",
        )
        .unwrap();
        entry(editor);
        wait(editor, |s| s.git_observation().is_some());
        let frame = editor.frame().unwrap();
        tones(
            &frame,
            &[
                ("Untracked (1)", Rgb(0x2a, 0xc3, 0xde)),
                ("Unstaged (1)", Rgb(0xe0, 0xaf, 0x68)),
                ("Staged (1)", Rgb(0x9e, 0xce, 0x6a)),
            ],
        );
        let row = frame
            .rows()
            .iter()
            .position(|r| r.text().contains("source.txt"))
            .unwrap();
        click(editor, row as u16);
        key(editor, KeyCode::Tab, KeyModifiers::NONE);
        wait(editor, |s| {
            s.echo().contains("comparison retained as read-only")
        });
        let row = render(editor)
            .iter()
            .position(|r| r.contains("@@"))
            .unwrap();
        click(editor, row as u16);
        key(editor, KeyCode::Tab, KeyModifiers::NONE);
        check_diff(editor, added, removed);
        editor.execute(Command::Git(GitCommand::Diff));
        check_diff(editor, added, removed);
        let frame = editor.frame().unwrap();
        let row = frame
            .rows()
            .iter()
            .position(|r| r.text().contains("+SOURCE_AFTER"))
            .unwrap();
        let cells = surface::draw(&frame);
        assert_eq!(
            cells[&(row, 0)].background,
            Some(config::Palette::default().tree_selected)
        );
        plain(editor, 'q');
        let row = render(editor)
            .iter()
            .position(|r| r.contains("untracked.txt"))
            .unwrap();
        click(editor, row as u16);
        editor.execute(Command::Git(GitCommand::Diff));
        wait(editor, |s| {
            s.echo().contains("comparison retained as read-only")
        });
        tones(
            &editor.frame().unwrap(),
            &[
                ("++literal", added),
                ("+-literal", added),
                ("+@@ literal", added),
            ],
        );
    }
}

fn check_diff(editor: &mut Session, added: Rgb, removed: Rgb) {
    tones(
        &editor.frame().unwrap(),
        &[
            ("+SOURCE_AFTER", added),
            ("-SOURCE_BEFORE", removed),
            ("@@ ", Rgb(0x9d, 0x7c, 0xd8)),
            ("common before", config::Palette::default().foreground),
        ],
    );
}

// Every Git palette leaf resolves through includes and provenance; malformed
// colors and unknown names reject the whole candidate rather than leak overrides.
#[test]
fn palette_contract() {
    let fixture = crate::fixture::Fixture::new();
    let base = fixture.write("base.toml", "[palette.git]\nheading='#010101'\nmetadata='#020202'\nuntracked='#030303'\nunstaged='#040404'\nstaged='#050505'\nconflict='#060606'\nadded='#070707'\nremoved='#080808'\nhunk='#090909'");
    let root = fixture.write(
        "root.toml",
        "include=['base.toml']\n[palette.git]\nadded='#101112'",
    );
    let prepared = config::prepare(&root);
    assert!(prepared.diagnostic.is_none(), "{:?}", prepared.diagnostic);
    let p = &prepared.snapshot.settings().palette.git;
    for (name, actual, expected) in [
        ("heading", p.heading, Rgb(1, 1, 1)),
        ("metadata", p.metadata, Rgb(2, 2, 2)),
        ("untracked", p.untracked, Rgb(3, 3, 3)),
        ("unstaged", p.unstaged, Rgb(4, 4, 4)),
        ("staged", p.staged, Rgb(5, 5, 5)),
        ("conflict", p.conflict, Rgb(6, 6, 6)),
        ("added", p.added, Rgb(16, 17, 18)),
        ("removed", p.removed, Rgb(8, 8, 8)),
        ("hunk", p.hunk, Rgb(9, 9, 9)),
    ] {
        assert_eq!(actual, expected, "{name}");
        let origin = if name == "added" { &root } else { &base };
        assert_eq!(
            prepared.snapshot.origin(&["palette", "git", name]),
            Some(origin.as_path()),
            "{name}"
        );
    }
    for bad in ["added='#xyzxyz'", "added='#123'", "unknown='#123456'"] {
        let path = fixture.write(
            "bad.toml",
            &format!("editor.tab_width=3\n[palette.git]\n{bad}"),
        );
        let prepared = config::prepare(&path);
        assert!(prepared.diagnostic.is_some(), "{bad}");
        assert_eq!(prepared.snapshot.settings(), &config::Settings::default());
    }
}
