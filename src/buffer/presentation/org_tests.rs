use super::tests::{analyze, edit, observed};
use crate::org::{Cancellation, ColorScan, Limits};

fn compare(buffer: &crate::buffer::Buffer) {
    let coloring = ColorScan::new(buffer, Limits::default())
        .unwrap()
        .run(&Cancellation::default());
    let (_, _, fresh) = coloring.into_parts();
    for row in 0..buffer.len_lines() {
        let start = buffer.text.line_to_byte(row);
        assert_eq!(
            buffer.org_heading_level(row),
            fresh.heading_level_at(start),
            "row {row}"
        );
        assert_eq!(
            buffer.org_line_indentation(row),
            fresh.indentation_at(start),
            "row {row}"
        );
    }
}

// Accepted body/title mutations must actually carry and agree with the full
// owner, across repeated Unicode edits on two rows before any exact replacement.
#[test]
fn org_generated_carries() {
    let mut buffer =
        observed("* Root\nordinary body\n** TODO [#A] Child :tag:\nordinary body\n*** Deep\n");
    let mut accepted = 0;
    for step in 0..64 {
        let row = if step % 2 == 0 { 1 } else { 2 };
        let start = buffer.text.line_to_char(row) + 4;
        let value = ["x", "界", "e\u{301}", "z"][step % 4];
        edit(&mut buffer, start..start, value);
        assert!(buffer.presentation.is_some(), "step {step}");
        assert!(buffer.org_outline().is_none());
        compare(&buffer);
        accepted += 1;
    }
    assert_eq!(accepted, 64);
    println!(
        "Org generated body/title mapping: accepted={accepted}, full-analysis comparisons={accepted}"
    );
}

// Both sides of admission must be safe; deleting a delimiter, editing a
// definition, literal stars and the non-ASCII-space separator cannot carry.
#[test]
fn org_admission_matrix() {
    let cases = [
        (
            "body insertion",
            "* Root\nbody\n** Deep\n",
            9..9,
            "界",
            true,
        ),
        ("body deletion", "* Root\nbody\n** Deep\n", 9..10, "", true),
        (
            "title insertion",
            "** TODO [#A] Child :tag:\nbody\n",
            4..4,
            "界",
            true,
        ),
        ("title deletion", "** Child\nbody\n", 4..5, "", true),
        ("prefix star", "** Child\nbody\n", 0..1, "", false),
        ("prefix space", "** Child\nbody\n", 2..3, "", false),
        ("tab separator", "**\tChild\nbody\n", 4..4, "x", false),
        (
            "block deletion",
            "* Root\n#+begin_src text\n,* Literal\n#+end_src\n",
            7..23,
            "body",
            false,
        ),
        (
            "drawer deletion",
            "* Root\n:PROPERTIES:\n:END:\n",
            7..19,
            "body",
            false,
        ),
        ("keyword", "* Root\n#+TODO: TODO DONE\n", 16..16, "x", false),
        (
            "definition",
            "* Root\n[fn:a] definition\n",
            17..17,
            "x",
            false,
        ),
        (
            "literal star",
            "* Root\n#+begin_src text\n,* Literal\n#+end_src\n",
            27..27,
            "x",
            false,
        ),
        (
            "new opener",
            "* Root\nbody\n",
            7..11,
            "#+begin_src text",
            false,
        ),
        ("blank", "* Root\nbody\n", 7..11, "", false),
    ];
    let mut counts = (0, 0);
    for (name, source, range, inserted, accepted) in cases {
        let mut buffer = observed(source);
        edit(&mut buffer, range, inserted);
        assert_eq!(buffer.presentation.is_some(), accepted, "{name}");
        if accepted {
            compare(&buffer);
            counts.0 += 1;
        } else {
            counts.1 += 1;
        }
    }
    assert_eq!(counts, (4, 10));
    println!(
        "Org admission matrix: accepted={}, rejected={}",
        counts.0, counts.1
    );
}

// The classifier shares the same 32 KiB allowance with shape, line copies and
// prefix comparison. Refusal does not stop canonical source editing or rebase
// a failed chain; exact analysis supplies the next usable base.
#[test]
fn org_classifier_budget() {
    let mut buffer = observed(&format!("* Root\n{}\n** Deep\n", "a".repeat(8000)));
    edit(&mut buffer, 8..8, "x");
    assert!(buffer.presentation.is_none());
    assert_eq!(buffer.len_chars(), 8017);
    analyze(&mut buffer);
    assert_eq!(buffer.org_heading_level(2), Some(2));
}
