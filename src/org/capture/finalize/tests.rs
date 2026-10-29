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
// anchor. No discovery or planning step creates the headings on disk.
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
}
