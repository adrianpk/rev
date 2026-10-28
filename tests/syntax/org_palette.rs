use rev::org::{Cancellation, ColorScan, Limits, Role};
use rev::window::Windows;

use super::*;

fn color(buffer: &mut Buffer) {
    let colors = ColorScan::new(buffer, Limits::default())
        .unwrap()
        .run(&Cancellation::default());
    buffer.install_org_coloring(colors).unwrap();
}

// Every configured Org role has a real native capture and a renderer-visible
// effect. A single override recolors only that role's cells, including roles
// whose default channels happen to match; all sibling/UI cells stay identical.
#[test]
fn role_rendering() {
    let fixture = Fixture::new();
    let cases = [
        ("heading", Role::Heading, "* Heading\n"),
        ("keyword", Role::Keyword, "#+TITLE: Sample\n"),
        ("drawer", Role::Drawer, ":LOGBOOK:\nentry\n:END:\n"),
        (
            "property",
            Role::Property,
            "* Heading\n:PROPERTIES:\n:ID: kept\n:END:\n",
        ),
        (
            "planning",
            Role::Planning,
            "* Heading\nSCHEDULED: <2026-10-28 Wed>\n",
        ),
        (
            "block",
            Role::Block,
            "#+begin_example\nbody\n#+end_example\n",
        ),
        ("code", Role::Code, "~code~\n"),
        ("comment", Role::Comment, "# comment\n"),
        ("list", Role::List, "- item\n"),
        ("table", Role::Table, "| cell |\n"),
        ("formula", Role::Formula, "| cell |\n#+TBLFM: $1=2\n"),
        ("emphasis", Role::Emphasis, "*bold*\n"),
        ("link", Role::Link, "[[file:a][description]]\n"),
        ("target", Role::Target, "<<target>>\n"),
        ("footnote", Role::Footnote, "text [fn:x]\n"),
        ("entity", Role::Entity, "\\alpha\n"),
        ("malformed", Role::Malformed, "#+begin_example\n* Raw\n"),
    ];
    assert_eq!(cases.map(|(_, role, _)| role), Role::ALL);
    for (key, role, source) in cases {
        let path = fixture.file("notes.org", source);
        let mut defaults = Buffers::new().unwrap();
        let id = defaults.visit(&path).unwrap();
        color(defaults.get_mut(id).unwrap());
        let baseline = frame(defaults.get(id).unwrap());
        assert!(
            baseline
                .rows()
                .iter()
                .flat_map(|row| row.syntax())
                .any(|p| p.style == Style::Org(role)),
            "{key}"
        );
        let startup = rev::config::prepare(
            &fixture.file("config.toml", &format!("palette.org.{key} = '#012345'")),
        );
        assert!(startup.diagnostic.is_none());
        let mut buffers = Buffers::with_settings(startup.snapshot.clone()).unwrap();
        let id = buffers.visit(&path).unwrap();
        color(buffers.get_mut(id).unwrap());
        let buffer = buffers.get(id).unwrap();
        let composition = Frame::compose(
            &buffers,
            &Windows::new(View::new(buffer)).unwrap(),
            Size {
                width: 80,
                height: 8,
            },
            "ready",
        )
        .unwrap();
        assert_eq!(
            surface::draw(&composition),
            expected(&baseline, role),
            "{key}"
        );
        assert_eq!(
            surface::draw(&composition),
            surface::draw(&frame(buffer).with_palette(&startup.snapshot.settings().palette)),
            "{key}"
        );
        assert_eq!(text(buffer), source);
        assert!(buffer.history().is_empty());
        assert!(!buffer.is_modified());
    }
}

fn expected(
    frame: &Frame,
    role: Role,
) -> std::collections::BTreeMap<(usize, usize), surface::Cell> {
    let mut cells = surface::draw(frame);
    for row in frame.rows() {
        for paint in row
            .syntax()
            .iter()
            .filter(|paint| paint.style == Style::Org(role))
        {
            for column in paint.columns.clone() {
                cells
                    .get_mut(&(usize::from(row.row()), usize::from(column)))
                    .unwrap()
                    .foreground = Some(rev::config::Rgb(1, 35, 69));
            }
        }
    }
    cells
}
