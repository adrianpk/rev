use super::{Editor, marks, surface};
use rev::config::Rgb;

// The writer resolves the final merged semantic colors and explicit pair roles,
// cycles depth without touching other characters, and keeps equal-depth mates.
#[test]
fn depth_palette() {
    for config in [
        "palette.syntax.function='#010203'\npalette.syntax.number='#040506'\npalette.syntax.keyword='#070809'",
        "palette.pairs.depth_colors=['#010203','#040506','#070809']",
    ] {
        let source = "([({xyz})])";
        let mut e = Editor::new(config, source, 5);
        let frame = e.frame();
        let cells = surface::draw(&frame);
        let colors = &e.palette.pairs.depth_colors;
        assert_eq!(marks(&frame).len(), 8);
        for (column, depth) in [
            (0, 0),
            (1, 1),
            (2, 2),
            (3, 3),
            (7, 3),
            (8, 2),
            (9, 1),
            (10, 0),
        ] {
            assert_eq!(
                cells[&(0, column)].foreground,
                Some(colors[depth % colors.len()])
            );
            assert_eq!(cells[&(0, column)].background, Some(e.palette.background));
        }
        assert_eq!(cells[&(0, 4)].foreground, Some(e.palette.foreground));
        e.unchanged(source, 5);
    }
}

// Match colors affect only the two confirmed endpoints; depth preserves its
// foreground while adding the active background/underline. Off restores source.
#[test]
fn active_and_off() {
    let source = "([x])";
    let config = "palette.pairs.depth_colors=['#112233','#223344']\npalette.pairs.match_text='#334455'\npalette.pairs.match_background='#445566'";
    let mut e = Editor::new(config, source, 1);
    for style in ["depth", "match", "off"] {
        e.choose(style);
        let frame = e.frame();
        let cells = surface::draw(&frame);
        let active = style != "off";
        for column in [1, 3] {
            let foreground = match style {
                "depth" => Rgb(0x22, 0x33, 0x44),
                "match" => Rgb(0x33, 0x44, 0x55),
                _ => e.palette.foreground,
            };
            assert_eq!(cells[&(0, column)].foreground, Some(foreground), "{style}");
            assert_eq!(cells[&(0, column)].underline, active, "{style}");
            assert_eq!(
                cells[&(0, column)].background,
                Some(if active {
                    Rgb(0x44, 0x55, 0x66)
                } else {
                    e.palette.background
                })
            );
        }
        assert_eq!(
            marks(&frame).len(),
            match style {
                "depth" => 4,
                "match" => 2,
                _ => 0,
            }
        );
        assert!(!cells[&(0, 0)].underline && !cells[&(0, 4)].underline);
        e.unchanged(source, 1);
    }
}

// At-point wins over immediately-before, and ordinary body text cannot select
// arbitrary enclosing pairs. Missing/mismatched/uncertain facts do not become errors.
#[test]
fn entry_and_uncertainty() {
    for (source, point, expected) in [
        ("()[]", 2, vec![2, 3]),
        ("(x)", 3, vec![0, 2]),
        ("(body)", 3, vec![]),
        ("(unfinished", 0, vec![]),
        ("([)]", 0, vec![]),
        ("(ok) ([)]", 0, vec![0, 3]),
        ("(\"unfinished", 0, vec![]),
    ] {
        let mut e = Editor::new("pairs.highlight='match'", source, point);
        let frame = e.frame();
        assert_eq!(
            marks(&frame).iter().map(|m| m.0).collect::<Vec<_>>(),
            expected,
            "{source}"
        );
        let cells = surface::draw(&frame);
        assert!(
            cells
                .values()
                .all(|c| c.foreground != Some(e.palette.pairs.mismatch))
        );
        e.unchanged(source, point);
    }
}

// Native lexical identity excludes comments, strings and fenced source even
// with syntax disabled; eligible quote endpoints keep their ordinary string ink.
#[test]
fn lexical_and_quotes() {
    for colors in [true, false] {
        for style in ["depth", "match"] {
            quote_case(colors, style);
        }
    }
}

fn quote_case(colors: bool, style: &str) {
    let source = "// [ignored]\n{\"(literal)\"}";
    let config = format!(
        "files.associations.'*.case'='rust'\nsyntax.enabled={colors}\npairs.highlight='{style}'"
    );
    let mut e = Editor::new(&config, source, 14);
    let frame = e.frame();
    let cells = surface::draw(&frame);
    assert_eq!(
        marks(&frame).iter().map(|m| m.0).collect::<Vec<_>>(),
        if style == "depth" {
            vec![13, 14, 24, 25]
        } else {
            vec![14, 24]
        }
    );
    for column in [1, 11] {
        assert_eq!(
            cells[&(1, column)].foreground,
            Some(if colors {
                e.palette.syntax.string
            } else {
                e.palette.foreground
            })
        );
        assert!(cells[&(1, column)].underline);
    }
    for column in [2, 10] {
        assert!(!cells[&(1, column)].underline);
    }
    e.unchanged(source, 14);
}

// Fenced source is lexical context, not punctuation to color. Prose outside the
// fence retains native depth through both Org and Markdown presentation owners.
#[test]
fn fenced_source() {
    for (language, open, close) in [
        ("org", "#+begin_src rust", "#+end_src"),
        ("markdown", "```rust", "```"),
    ] {
        let source = format!("Body ([x])\n{open}\n{{[excluded]}}\n{close}\n");
        let mut e = Editor::new(
            &format!("files.associations.'*.case'='{language}'"),
            &source,
            0,
        );
        let frame = e.frame();
        assert_eq!(
            marks(&frame).iter().map(|m| m.0).collect::<Vec<_>>(),
            [5, 6, 8, 9]
        );
        e.unchanged(&source, 0);
    }
}
