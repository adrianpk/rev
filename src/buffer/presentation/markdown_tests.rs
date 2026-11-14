use super::tests::edit;
use crate::buffer::{Buffer, Mode, edit::Point};
use crate::coordinates::Scalar;
use crate::markdown::{DisplayItem, Observation, Work};
use crate::process::Cancellation;
use ropey::Rope;

fn fresh(buffer: &Buffer) -> Observation {
    Work::new(buffer.stamp(), buffer.text.clone())
        .run(&Cancellation::default())
        .unwrap()
}

fn observed(source: &str) -> Buffer {
    let mut buffer = Buffer::from_rope(Rope::from_str(source)).unwrap();
    buffer.select_mode(Mode::Markdown).unwrap();
    let observation = fresh(&buffer);
    buffer.install_markdown(observation);
    buffer
}

fn items(
    items: impl Iterator<Item = DisplayItem>,
) -> Vec<(std::ops::Range<usize>, usize, Option<u8>)> {
    items
        .map(|item| (item.marker, item.depth, item.bullet))
        .collect()
}

fn compare(buffer: &Buffer) {
    let fresh = fresh(buffer);
    for row in 0..buffer.len_lines() {
        let start = buffer.text.line_to_byte(row);
        let line = start..start + buffer.text.line(row).len_bytes();
        assert_eq!(
            buffer.markdown_heading_prefix(row),
            fresh.heading_prefix(line.clone()),
            "row {row}"
        );
        assert_eq!(
            items(buffer.markdown_display_items(row).into_iter()),
            items(fresh.presentation_items(line)),
            "row {row}"
        );
    }
}

// Real publications merge Unicode changes on two rows. Every positive carry
// agrees with full pulldown-cmark output and supplies no list/table authority.
#[test]
fn markdown_carries() {
    let mut buffer =
        observed("# Root\nordinary body\n  ## Child\n\n- Item\n  - Child\n\n### Deep\n");
    for step in 0..64 {
        let row = if step % 2 == 0 { 1 } else { 2 };
        let column = if row == 1 { 4 } else { 6 };
        let at = buffer.text.line_to_char(row) + column;
        edit(&mut buffer, at..at, ["x", "界", "e\u{301}", "z"][step % 4]);
        assert!(buffer.presentation.is_some(), "step {step}");
        assert!(buffer.markdown_observation().is_none());
        compare(&buffer);
    }
    let base = buffer.markdown.observation.as_ref().unwrap();
    assert!(
        base.list_edit(
            &buffer,
            crate::command::MarkdownCommand::IndentItem,
            Scalar(0)
        )
        .is_err()
    );
    assert!(
        base.table_work(
            &buffer,
            Point {
                stamp: buffer.stamp(),
                position: Scalar(0)
            }
        )
        .is_err()
    );
    println!("Markdown generated mapping: accepted=64, full-parser comparisons=64");
}

// Both source versions and complete parser context participate. Definitions,
// lazy list continuations, Setext, HTML, literal code and changed indentation
// must refuse even when the edited text alone looks like a plain body word.
#[test]
fn markdown_admission() {
    let cases = [
        (
            "body insert",
            "# Root\nbody text\n\n## Child\n",
            "body",
            "bod界y",
            true,
        ),
        ("body delete", "# Root\nbody text\n", "body", "bod", true),
        (
            "indented body",
            "# Root\n  body text\n",
            "body",
            "body x",
            true,
        ),
        ("title", "  ## Child\n", "Child", "C界hild", true),
        ("quote title", "> ## Child\n", "Child", "C界hild", true),
        ("list title", "- ## Child\n", "Child", "C界hild", true),
        ("empty title", "## Child\n", "Child", "", true),
        (
            "indent change",
            "# Root\n  body\n",
            "  body",
            " body",
            false,
        ),
        ("blank", "# Root\nbody\n", "body", "", false),
        ("prefix", "## Child\n", "##", "#", false),
        ("separator", "## Child\n", "## ", "##", false),
        ("new heading", "body\n", "body", "# Title", false),
        ("BOM opener", "body\n", "body", "\u{feff}# Title", false),
        ("new fence", "body\n", "body", "~~~", false),
        ("fence", "```text\nbody\n```\n", "body", "body x", false),
        (
            "fence deletion",
            "```text\nbody\n```\n",
            "```text",
            "body",
            false,
        ),
        ("code", "    body\n", "body", "body x", false),
        ("list", "- body\n", "body", "body x", false),
        (
            "lazy list",
            "- item\nbody continuation\n",
            "body",
            "body x",
            false,
        ),
        (
            "quote",
            "> quote\nbody continuation\n",
            "body",
            "body x",
            false,
        ),
        ("Setext", "Title\n====\n", "Title", "Title x", false),
        (
            "table",
            "| a | b |\n| - | - |\n| body | x |\n",
            "body",
            "body x",
            false,
        ),
        ("new pipe", "body\n", "body", "body | x", false),
        ("definition", "[ref]: /a\n\n# Root\n", "/a", "/b", false),
        ("destination", "[ref]:\n /a\n\n# Root\n", "/a", "/b", false),
        (
            "definition title",
            "[ref]: /a\n \"title\"\n\n# Root\n",
            "title",
            "title x",
            false,
        ),
        (
            "duplicate definition",
            "[ref]: /a\n[ref]:\n /b\n\n# Root\n",
            "/b",
            "/c",
            false,
        ),
        ("HTML", "<div>\nbody\n</div>\n", "body", "body x", false),
    ];
    let mut counts = (0, 0);
    for (name, source, removed, inserted, accepted) in cases {
        let mut buffer = observed(source);
        let at = source.find(removed).unwrap();
        let from = buffer.text.byte_to_char(at);
        edit(&mut buffer, from..from + removed.chars().count(), inserted);
        assert_eq!(buffer.presentation.is_some(), accepted, "{name}");
        if accepted {
            compare(&buffer);
            counts.0 += 1;
        } else {
            counts.1 += 1;
        }
    }
    assert_eq!(counts, (7, 21));
    println!(
        "Markdown admission matrix: accepted={}, rejected={}",
        counts.0, counts.1
    );
}

// Oversized inspection retires presentation without impeding canonical source
// editing; an ordinary later key cannot bridge the rejected revision.
#[test]
fn markdown_budget() {
    let source = format!("# Root\n{}\n\n## Child\n", "a".repeat(8000));
    let mut buffer = observed(&source);
    edit(&mut buffer, 8..8, "界");
    assert!(buffer.presentation.is_none());
    assert!(buffer.markdown.observation.is_none());
    assert_eq!(buffer.len_bytes(), source.len() + 3);
    edit(&mut buffer, 9..9, "x");
    assert!(buffer.presentation.is_none());
}
