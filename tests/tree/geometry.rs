use rev::buffer::Buffers;
use rev::config::prepare;
use rev::display::Frame;
use rev::panel::Composition;
use rev::tree::Tree;
use rev::view::View;
use rev::window::Windows;

use super::*;

// Both catalogs use the same name boundaries under whole-grapheme clipping.
// The standalone snapshot API, control sanitization and hidden/zero panels are
// exercised without a Session or filesystem work during frame composition.
#[test]
fn clipped_names() {
    for nerd_fonts in [false, true] {
        let fixture = config_fixture::Fixture::new();
        fixture.write("root/界é.txt", "retained");
        fixture.write("root/ctl\x1b[31m.txt", "safe");
        for (width, expected) in [
            (1, " "),
            (2, "  "),
            (3, "   "),
            (4, "    "),
            (5, "    ≡"),
            (6, "    ≡ "),
            (7, "    ≡  "),
            (8, "    ≡ 界"),
            (9, "    ≡ 界é"),
            (10, "    ≡ 界é."),
            (11, "    ≡ 界é.t"),
            (12, "    ≡ 界é.tx"),
            (13, "    ≡ 界é.txt"),
            (24, "    ≡ 界é.txt           "),
        ] {
            check_name_width(&fixture, nerd_fonts, width, expected);
        }
    }
}

fn check_name_width(
    fixture: &config_fixture::Fixture,
    nerd_fonts: bool,
    width: u16,
    expected: &str,
) {
    let startup = prepare(&fixture.write(
        "config.toml",
        &format!("tree.visible=true\ntree.auto_width=false\ntree.width={width}\ngutter.nerd_fonts={nerd_fonts}"),
    ));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let buffers = Buffers::with_settings(startup.snapshot.clone()).unwrap();
    let view = View::with_settings(buffers.get(buffers.initial()).unwrap(), &startup.snapshot);
    let windows = Windows::with_settings(view, &startup.snapshot).unwrap();
    let mut tree = Tree::with_settings(fixture.0.join("root"), &startup.snapshot);
    tree.expand().unwrap();
    for (columns, rows) in [(100, 8), (31, 8), (0, 0), (100, 8)] {
        let size = Size {
            width: columns,
            height: rows,
        };
        let composition = Composition::workspace(&windows, &tree, size, 0);
        if !size.usable() {
            assert!(composition.is_none());
            assert_eq!(tree.entries().len(), 4);
            continue;
        }
        let composition = composition.unwrap();
        let frame = Frame::compose_workspace(&buffers, &windows, &tree, &composition, "").unwrap();
        assert!(
            frame
                .rows()
                .iter()
                .all(|row| row.text().width() == usize::from(columns))
        );
        if let Some(panel) = composition.tree() {
            assert_eq!(panel.content().width(), width);
            let actual = frame.rows()[3].text().split('│').next().unwrap();
            let expected = if nerd_fonts {
                expected.replace('≡', "\u{f0219}")
            } else {
                expected.into()
            };
            assert_eq!(actual, expected, "{nerd_fonts}/{width}/{columns}");
            if width == 24 {
                assert!(frame.rows()[2].text().contains("ctl^[[31m.txt"));
            }
            let mut bytes = Vec::new();
            frame.write(&mut bytes).unwrap();
            assert!(!String::from_utf8(bytes).unwrap().contains("\x1b[31m"));
        } else {
            assert!(columns < 32);
            assert!(
                frame
                    .rows()
                    .iter()
                    .all(|row| !row.text().contains('◆') && !row.text().contains('\u{f1b2}'))
            );
        }
    }
}
