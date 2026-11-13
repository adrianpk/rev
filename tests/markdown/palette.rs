use rev::config::Rgb;

use super::*;

// Every semantic role must change real terminal cells independently. Equal
// factory colors cannot hide a wrong role mapping or a leaking override.
#[test]
fn native_palette() {
    let source = "*word*\n\n[word](https://example.com)\n\n`code`\n\n> quote\n\n- list\n\n| table | value |\n| --- | --- |\n";
    let (_fixture, mut baseline) = editor("", "notes.md", source);
    settle(&mut baseline);
    let frame = baseline.frame().unwrap();
    for (key, role) in [
        ("emphasis", MarkdownRole::Emphasis),
        ("link", MarkdownRole::Link),
        ("code", MarkdownRole::Code),
        ("quote", MarkdownRole::Quote),
        ("list", MarkdownRole::List),
        ("table", MarkdownRole::Table),
    ] {
        let config = format!("palette.markdown.{key}='#012345'");
        let (_fixture, mut changed) = editor(&config, "notes.md", source);
        settle(&mut changed);
        let actual = changed.frame().unwrap();
        assert_eq!(surface::draw(&actual), recolored(&frame, role), "{key}");
        assert_eq!(text(changed.buffer().unwrap()), source);
    }
}

// Rendering and mode changes preserve BOM/CRLF and Unicode source in a readonly
// buffer. Disabled colors keep native mode but emit no semantic cell treatments.
#[test]
fn source_fidelity() {
    for colored in [true, false] {
        let fixture = Fixture::new();
        let source = "*界e\u{301}*\r\n";
        let path = fixture.write("notes.markdown", &format!("\u{feff}{source}"));
        let startup = prepare(&fixture.write("config.toml", &format!("syntax.enabled={colored}")));
        let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
        let id = buffers.visit(&path).unwrap();
        buffers.get_mut(id).unwrap().set_readonly(true);
        let view = View::new(buffers.get(id).unwrap());
        let mut session = Session::new(buffers, view, SIZE).unwrap();
        let stamp = session.buffer().unwrap().stamp();
        settle(&mut session);
        assert_eq!(
            session
                .frame()
                .unwrap()
                .rows()
                .iter()
                .any(|row| !row.syntax().is_empty()),
            colored
        );
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        assert!(session.buffer().unwrap().has_bom());
        assert_eq!(text(session.buffer().unwrap()), source);
        assert_eq!(
            std::fs::read_to_string(path).unwrap(),
            format!("\u{feff}{source}")
        );
        assert!(session.buffer().unwrap().is_readonly());
        assert!(!session.buffer().unwrap().is_modified());
        assert!(session.buffer().unwrap().history().is_empty());
    }
}

fn recolored(
    frame: &Frame,
    role: MarkdownRole,
) -> std::collections::BTreeMap<(usize, usize), surface::Cell> {
    let mut expected = surface::draw(frame);
    let mut count = 0;
    for row in frame.rows() {
        for paint in row
            .syntax()
            .iter()
            .filter(|paint| paint.style == Style::Markdown(role))
        {
            for column in paint.columns.clone() {
                expected
                    .get_mut(&(usize::from(row.row()), usize::from(column)))
                    .unwrap()
                    .foreground = Some(Rgb(1, 35, 69));
                count += 1;
            }
        }
    }
    assert!(count > 0, "{role:?}");
    expected
}

// The shared monochrome writer must suppress RGB syntax output while retaining
// canonical Markdown. This exercises the same renderer switch used by NO_COLOR.
#[test]
fn monochrome_output() {
    let (_fixture, mut session) = editor("", "notes.md", "*word* and `code`\n");
    settle(&mut session);
    let frame = session.frame().unwrap();
    crossterm::style::force_color_output(false);
    let mut output = Vec::new();
    frame.write(&mut output).unwrap();
    crossterm::style::force_color_output(true);
    let ansi = std::str::from_utf8(&output).unwrap();
    assert!(!ansi.contains("[38;2;"));
    assert!(!ansi.contains("[48;2;"));
    let cells = surface::decode(output);
    let row: String = cells
        .iter()
        .filter(|((row, _), _)| *row == 0)
        .map(|(_, cell)| cell.character)
        .collect();
    assert!(row.contains("*word* and `code`"));
    assert_eq!(text(session.buffer().unwrap()), "*word* and `code`\n");
}
