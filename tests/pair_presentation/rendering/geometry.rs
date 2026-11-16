use super::{Editor, marks, surface};

// Whole graphemes keep source identities and physical columns across CRLF,
// wide/combining text and native wrapping; paint never shifts canonical point.
#[test]
fn graphemes_and_wrap() {
    let source = "(界e\u{301}[abcdefghijk])\r\n";
    for wrap in [true, false] {
        let mut e = Editor::new(&format!("editor.soft_wrap={wrap}"), source, 0);
        e.session.handle(crossterm::event::Event::Resize(9, 14));
        let frame = e.frame();
        let cells = surface::draw(&frame);
        let positions = marks(&frame);
        assert_eq!(positions.len(), if wrap { 4 } else { 2 });
        for (source, row, column) in positions {
            let character = e
                .session
                .buffer()
                .unwrap()
                .slice(
                    &e.session
                        .buffer()
                        .unwrap()
                        .span(source..source + 1)
                        .unwrap(),
                )
                .unwrap()
                .to_string();
            assert_eq!(cells[&(row, column)].character.to_string(), character);
        }
        e.unchanged(source, 0);
    }
}

// Org/Markdown retain their native heading/list prefixes and table source cells;
// switching pair styles changes delimiter ink without changing the projection.
#[test]
fn prose_projections() {
    for (language, heading, marker) in [("org", "**", "○"), ("markdown", "##", "○")] {
        let source = format!("{heading} Title\n- item ([body])\n| a | (cell) |\n");
        let config = format!("files.associations.'*.case'='{language}'\norg.indent=false");
        let mut e = Editor::new(&config, &source, 0);
        let end = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let frame = loop {
            assert!(
                std::time::Instant::now() < end,
                "native heading publication"
            );
            let frame = e.frame();
            if frame.rows()[0].text().contains(marker) {
                break frame;
            }
            std::thread::yield_now();
        };
        let baseline = surface::draw(&frame);
        assert_eq!(marks(&frame).len(), 6);
        let rows: Vec<_> = frame.rows().iter().map(|r| r.text().to_string()).collect();
        e.choose("off");
        let off = e.frame();
        assert!(marks(&off).is_empty());
        for (row, expected) in rows.iter().take(3).enumerate() {
            assert_eq!(off.rows()[row].text(), expected);
        }
        let unpainted = surface::draw(&off);
        for column in 0..10 {
            assert_eq!(baseline[&(0, column)], unpainted[&(0, column)]);
        }
        assert_eq!(off.cursor(), frame.cursor());
        e.unchanged(&source, 0);
    }
}

// Folded logical lines never acquire terminal marks. A visible mate can retain
// its structural depth while its source partner is hidden by the existing fold.
#[test]
fn folded_rows() {
    use rev::coordinates::Byte;
    let source = "* H\n(hidden\nbody\n)\n* Tail\n";
    let mut e = Editor::prepared(
        "files.associations.'*.case'='org'\norg.indent=false",
        source,
        0,
        |buffer, view| {
            let hidden = rev::org::Source::new(buffer)
                .range(Byte(4)..Byte(17))
                .unwrap();
            view.set_folds(buffer, &[hidden]).unwrap();
        },
    );
    let frame = e.frame();
    let positions = marks(&frame);
    assert_eq!(positions, [(17, 1, 0)]);
    assert_eq!(surface::draw(&frame)[&(1, 0)].character, ')');
    assert_eq!(
        e.session.view().visible_lines(e.session.buffer().unwrap()),
        4
    );
    e.unchanged(source, 0);
}

// Independent sibling policy and points share source qualification without
// sharing a retained paint cache. Closing one view retires only that interest.
#[test]
fn split_styles() {
    use rev::command::{Command, Motion, Split, WindowCommand};
    let source = "([x])";
    let mut e = Editor::new("", source, 0);
    e.session.execute(Command::Split(Split::Right));
    let ids = e.session.windows().order();
    e.session.select_view(ids[1]).unwrap();
    e.choose("match");
    e.session.execute(Command::Motion(Motion::ForwardChar));
    let frame = e.frame();
    let cells = surface::draw(&frame);
    let positions = marks(&frame);
    assert_eq!(positions.len(), 6);
    let active: Vec<_> = positions
        .iter()
        .filter(|(_, row, column)| cells[&(*row, *column)].underline)
        .map(|m| m.0)
        .collect();
    assert_eq!(active, [0, 4, 1, 3]);
    assert_eq!(e.session.windows().get(ids[0]).unwrap().point.position.0, 0);
    e.session.execute(Command::Window(WindowCommand::Delete));
    let frame = e.frame();
    assert_eq!(marks(&frame).len(), 4);
    assert!(
        e.session
            .remaining_input(std::time::Instant::now())
            .is_none()
    );
    assert!(e.session.buffer().unwrap().retained_pair_bytes() <= rev::pairs::MAX_METADATA);
    e.unchanged(source, 0);
}
