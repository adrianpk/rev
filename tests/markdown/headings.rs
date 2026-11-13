use rev::config::Rgb;

use super::*;

const GLYPHS: [char; 6] = ['◉', '○', '✸', '✿', '◆', '▶'];
const COLORS: [Rgb; 6] = [
    Rgb(122, 162, 247),
    Rgb(157, 124, 216),
    Rgb(192, 202, 245),
    Rgb(155, 185, 249),
    Rgb(181, 156, 225),
    Rgb(188, 208, 251),
];

// Six exact default faces and symbols resolve to real cells in both views;
// custom Org colors cannot influence Markdown. Inline semantics retain priority.
#[test]
fn visual_profile() {
    let mut source = (1..=6)
        .map(|level| {
            format!(
                "{} Title *em* `code` [link](https://example.com)\n",
                "#".repeat(level)
            )
        })
        .collect::<String>();
    source.push_str("####### Plain\n\nSetext one\n==========\n\nSetext two\n----------\n");
    for pretty in [true, false] {
        let config = format!("markdown.pretty={pretty}\npalette.org.heading='#010203'");
        let (_fixture, mut session) = editor(&config, "notes.md", &source);
        settle(&mut session);
        let frame = session.frame().unwrap();
        let cells = surface::draw(&frame);
        for row in 0..6 {
            let marker = cells[&(row, 7 + row)];
            assert_eq!(marker.character, if pretty { GLYPHS[row] } else { '#' });
            assert_eq!(marker.foreground, Some(COLORS[row]));
            assert_eq!(cells[&(row, 9 + row)].foreground, Some(COLORS[row]));
            for (word, role) in [
                ("em", MarkdownRole::Emphasis),
                ("code", MarkdownRole::Code),
                ("link", MarkdownRole::Link),
            ] {
                assert_eq!(style_at(&frame, row, word), Some(Style::Markdown(role)));
            }
        }
        assert_eq!(style_at(&frame, 6, "Plain"), None);
        assert_eq!(
            style_at(&frame, 8, "Setext"),
            Some(Style::Markdown(MarkdownRole::Heading(1)))
        );
        assert_eq!(
            style_at(&frame, 11, "Setext"),
            Some(Style::Markdown(MarkdownRole::Heading(2)))
        );
        assert!(frame.rows()[9].text().contains("=========="));
        assert!(frame.rows()[12].text().contains("----------"));
        assert_eq!(text(session.buffer().unwrap()), source);
        assert!(session.buffer().unwrap().history().is_empty());
    }
}

// Only parser-admitted ATX prefixes are replaced, including container prefixes;
// a Setext title starting with a hash is not an ATX marker.
#[test]
fn admitted_prefixes() {
    for (name, source, row, expected, glyph) in [
        ("quote", "> ## Title\n", 0, ">  ○ Title", true),
        ("list", "- ### Title\n", 0, "•   ✸ Title", true),
        ("indent", "   ## Title ##\n", 0, "    ○ Title ##", true),
        ("empty", "######\n", 0, "     ▶", true),
        ("tab", "##\tTitle\n", 0, " ○", true),
        ("fence", "```\n## Title\n```\n", 1, "## Title", false),
        ("indented code", "    ## Title\n", 0, "    ## Title", false),
        ("seven", "####### Title\n", 0, "####### Title", false),
        ("escaped", "\\## Title\n", 0, "\\## Title", false),
        ("Setext hash", "#word\n=====\n", 0, "#word", false),
    ] {
        let (_fixture, mut session) = editor("", "notes.md", source);
        settle(&mut session);
        let frame = session.frame().unwrap();
        let body: String = frame.rows()[row].text().chars().skip(7).collect();
        assert!(body.starts_with(expected), "{name}: {body:?}");
        assert_eq!(body.chars().any(|c| GLYPHS.contains(&c)), glyph, "{name}");
        assert_eq!(text(session.buffer().unwrap()), source);
    }
}

// Each heading palette override changes only its admitted level, including
// marker and ordinary title cells, while inline colors keep their own leaves.
#[test]
fn heading_overrides() {
    let source = (1..=6)
        .map(|level| format!("{} Title\n", "#".repeat(level)))
        .collect::<String>();
    for (index, key) in [
        "heading",
        "heading_2",
        "heading_3",
        "heading_4",
        "heading_5",
        "heading_6",
    ]
    .into_iter()
    .enumerate()
    {
        let (_fixture, mut session) = editor(
            &format!("palette.markdown.{key}='#012345'"),
            "notes.md",
            &source,
        );
        settle(&mut session);
        let frame = session.frame().unwrap();
        let cells = surface::draw(&frame);
        for (row, color) in COLORS.iter().enumerate() {
            let expected = Some(if row == index { Rgb(1, 35, 69) } else { *color });
            assert_eq!(cells[&(row, 7 + row)].foreground, expected);
            assert_eq!(cells[&(row, 9 + row)].foreground, expected);
        }
    }
}
