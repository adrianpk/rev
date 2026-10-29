use super::*;
use crate::config::{CaptureKind, CaptureTarget, CaptureTemplate};
#[path = "../../../../tests/configuration/fixture.rs"]
mod fixture;
fn file() -> CaptureTarget {
    CaptureTarget::File {
        file: String::new(),
    }
}
fn heading() -> CaptureTarget {
    CaptureTarget::Headline {
        file: String::new(),
        headline: "Tasks".into(),
    }
}
struct Scenario<'a> {
    source: &'a str,
    draft: &'a str,
    kind: CaptureKind,
    target: CaptureTarget,
    prepend: bool,
}
fn planned(scenario: Scenario<'_>) -> (fixture::Fixture, Insertion) {
    let Scenario {
        source,
        draft,
        kind,
        target,
        prepend,
    } = scenario;
    let fixture = fixture::Fixture::new();
    let path = fixture.write("destination.org", source);
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.capture_draft(draft).unwrap();
    let choice = Choice {
        file: Some(path),
        template: CaptureTemplate {
            key: "t".into(),
            description: "Test".into(),
            kind,
            target,
            template: "%?".into(),
            prepend,
        },
    };
    let date = Date::new(2024, 3, 1).unwrap();
    let resolved = choice
        .discover(&buffers, &Settings::default().org, date)
        .unwrap()
        .run(&Cancellation::default())
        .unwrap();
    let insertion = resolved
        .finalize(
            &buffers,
            buffers.get(id).unwrap(),
            &Settings::default(),
            date,
        )
        .unwrap()
        .run(&Cancellation::default())
        .unwrap();
    (fixture, insertion)
}
fn output(insertion: &Insertion) -> String {
    let mut text = insertion.source.text().to_string();
    for edit in insertion.edits.iter().rev() {
        text.replace_range(edit.bytes.clone(), &edit.text)
    }
    text
}
// Plans preserve preamble, metadata, nested headings and exact old fragments;
// native entry levels and new separators follow the actual target context.
#[test]
fn finalize_placements() {
    for (name, source, draft, kind, target, prepend, expected) in [
        (
            "file",
            "#+TITLE: Keep\n* Old\n",
            "* New\n** Child",
            CaptureKind::Entry,
            file(),
            true,
            "#+TITLE: Keep\n* New\n** Child\n* Old\n",
        ),
        (
            "child",
            "* Tasks\r\n:PROPERTIES:\r\n:ID: keep\r\n:END:\r\n** Old\r\n* Peer",
            "* New\n** Child",
            CaptureKind::Entry,
            heading(),
            true,
            "* Tasks\r\n:PROPERTIES:\r\n:ID: keep\r\n:END:\r\n** New\r\n*** Child\r\n** Old\r\n* Peer",
        ),
        (
            "plain",
            "literal EOF",
            "%(opaque)\r\n界",
            CaptureKind::Plain,
            file(),
            false,
            "literal EOF%(opaque)\r\n界",
        ),
        (
            "list",
            "* Tasks\n- Old\n\n** Child\n",
            "- New",
            CaptureKind::Item,
            heading(),
            false,
            "* Tasks\n- Old\n- New\n\n** Child\n",
        ),
        (
            "ordered",
            "* Tasks\n1) Old\n2) Peer\n",
            "- [ ] New",
            CaptureKind::Checkitem,
            heading(),
            true,
            "* Tasks\n1) [ ] New\n2) Old\n3) Peer\n",
        ),
        (
            "table",
            "* Tasks\r\n| Old |\r\n\r\n#+TBLFM: keep\r\n",
            "| New |",
            CaptureKind::TableLine,
            heading(),
            false,
            "* Tasks\r\n| Old |\r\n| New |\r\n\r\n#+TBLFM: keep\r\n",
        ),
    ] {
        let (_fixture, insertion) = planned(Scenario {
            source,
            draft,
            kind,
            target,
            prepend,
        });
        assert_eq!(output(&insertion), expected, "{name}");
    }
}
// Missing date-tree headings use the retained civil date and chronological
// anchor. Planning leaves disk unchanged; ordinary publication saves the exact
// planned tree as one undoable edit through the actual destination owner.
#[test]
fn finalize_date_tree() {
    let source = "* 2024\n** 2024-04 April\n*** 2024-04-01 Monday\n";
    let target = CaptureTarget::DateTree {
        file: String::new(),
        outline: Vec::new(),
    };
    let (fixture, insertion) = planned(Scenario {
        source,
        draft: "* New",
        kind: CaptureKind::Entry,
        target,
        prepend: false,
    });
    assert_eq!(
        output(&insertion),
        "* 2024\n** 2024-03 March\n*** 2024-03-01 Friday\n**** New\n** 2024-04 April\n*** 2024-04-01 Monday\n"
    );
    assert_eq!(
        std::fs::read_to_string(fixture.0.join("destination.org")).unwrap(),
        source
    );
    let expected = output(&insertion);
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&fixture.0.join("destination.org")).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let changes = insertion.replacements(buffer).unwrap();
    let prepared = buffer.prepare(&changes, &[], None).unwrap();
    let outcome = buffer
        .publish_saved_edit(prepared, &mut [], &mut crate::file::Files)
        .unwrap();
    assert!(outcome.is_complete());
    assert_eq!(buffer.history().len(), 1);
    assert_eq!(
        std::fs::read_to_string(fixture.0.join("destination.org")).unwrap(),
        expected
    );
}

// The editable draft bound is checked before worker admission, including a
// caller-inserted NUL. An exact-limit plain draft remains literal and non-writing.
#[test]
fn finalize_draft_bounds() {
    let fixture = fixture::Fixture::new();
    let path = fixture.write("destination.org", "base");
    let mut buffers = Buffers::new().unwrap();
    let choice = Choice {
        file: Some(path.clone()),
        template: CaptureTemplate {
            key: "t".into(),
            description: "Plain".into(),
            kind: CaptureKind::Plain,
            target: file(),
            template: "%?".into(),
            prepend: false,
        },
    };
    let date = Date::new(2024, 3, 1).unwrap();
    let resolved = choice
        .discover(&buffers, &Settings::default().org, date)
        .unwrap()
        .run(&Cancellation::default())
        .unwrap();
    for (name, value, admitted) in [
        ("exact", "x".repeat(EXPANSION_BYTES), true),
        ("over", "x".repeat(EXPANSION_BYTES + 1), false),
        ("NUL", "a\0b".into(), false),
    ] {
        let id = buffers.capture_draft(&value).unwrap();
        let work = resolved.finalize(
            &buffers,
            buffers.get(id).unwrap(),
            &Settings::default(),
            date,
        );
        assert_eq!(work.is_ok(), admitted, "{name}");
        if let Ok(work) = work {
            let insertion = work.run(&Cancellation::default()).unwrap();
            assert_eq!(insertion.edits[0].text, value);
        }
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "base");
    }
}
