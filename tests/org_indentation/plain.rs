use super::*;

// Concealing a prefix changes cells, not source positions: hits target the
// visible final star, copy keeps every star, and presentation changes reflow.
#[test]
fn prefix_interaction() {
    let text = "*** Heading\nbody\n";
    for wrapped in [true, false] {
        let (_f, mut session) = editor(
            &format!("org.indent=false\neditor.soft_wrap={wrapped}"),
            text,
            40,
            12,
        );
        let frame = session.frame().unwrap();
        assert_eq!(frame.rows()[0].text().trim_end(), "✸ Heading");
        assert_eq!(frame.rows()[1].text().trim_end(), "body");
        for point in 0..3 {
            assert_eq!(session.view().point.position, Scalar(point));
            assert_eq!(
                session.frame().unwrap().cursor(),
                Some(Position { row: 0, column: 0 })
            );
            session.execute(Command::Motion(Motion::ForwardChar));
        }
        session.execute(Command::Motion(Motion::BufferStart));
        session.execute(Command::Region(rev::command::RegionCommand::Set));
        for _ in 0..3 {
            session.execute(Command::Motion(Motion::ForwardChar));
        }
        assert_eq!(session.frame().unwrap().rows()[0].selection(), Some(0..1));
        session.execute(Command::Linear(LinearCommand::Copy));
        assert_eq!(session.killed_text().unwrap().to_string(), "***");
        session.execute(Command::KeyboardQuit);

        for (column, point) in [(0, 2), (2, 4)] {
            mouse(
                &mut session,
                MouseEventKind::Down(MouseButton::Left),
                0,
                column,
            );
            mouse(
                &mut session,
                MouseEventKind::Up(MouseButton::Left),
                0,
                column,
            );
            assert_eq!(session.view().point.position, Scalar(point));
        }
        for (command, heading, column) in [
            (OrgCommand::PrettyHeadings, "*** Heading", 4),
            (OrgCommand::PrettyHeadings, "✸ Heading", 2),
            (OrgCommand::IndentEnable, "    ✸ Heading", 6),
            (OrgCommand::IndentDisable, "✸ Heading", 2),
        ] {
            session.execute(Command::Org(command));
            settle(&mut session);
            let frame = session.frame().unwrap();
            assert_eq!(frame.rows()[0].text().trim_end(), heading);
            assert_eq!(frame.cursor(), Some(Position { row: 0, column }));
            assert_eq!(session.view().point.position, Scalar(4));
        }
        assert_eq!(source(&session), text);
        assert!(!session.buffer().unwrap().is_modified());
        assert!(session.buffer().unwrap().history().is_empty());
    }
}

// Removed prefix cells do not consume wrapping width. Literal source stars
// remain visible when pretty presentation or leading-star hiding is disabled.
#[test]
fn prefix_wrapping() {
    let text = "*** abcdefghij\nbody\n";
    let (_f, mut session) = editor("org.indent=false", text, 6, 12);
    let frame = session.frame().unwrap();
    let rows: Vec<_> = frame
        .rows()
        .iter()
        .take(4)
        .map(|r| r.text().trim_end())
        .collect();
    assert_eq!(rows, ["✸", "abcdef", "ghij", "body"]);
    let fragments = session
        .view()
        .wrapped_rows(session.buffer().unwrap(), 6, 4)
        .unwrap();
    assert_eq!(fragments[0].source.start, Scalar(0));
    assert_eq!(fragments[0].source.end, Scalar(4));
    assert_eq!(fragments[1].source.start, Scalar(4));
    for (config, expected) in [
        ("org.headings.pretty=false", "*** Heading"),
        ("org.headings.hide_leading_stars=false", "**✸ Heading"),
    ] {
        let (_f, mut other) = editor(
            &format!("org.indent=false\n{config}"),
            "*** Heading\n",
            30,
            10,
        );
        assert_eq!(other.frame().unwrap().rows()[0].text().trim_end(), expected);
    }
    assert_eq!(source(&session), text);
}
