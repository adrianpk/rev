use super::Fixture;
use rev::{
    buffer::{Buffers, edit::Replacement},
    config::{CaptureKind, CaptureTarget, CaptureTemplate, Org},
    coordinates::Byte,
    org::{
        Cancellation,
        capture::{Choice, Container, Resolved},
        io::DestinationVersion,
        time::Date,
    },
};
use std::path::Path;
pub(super) fn choice(path: &Path, kind: CaptureKind, target: CaptureTarget) -> Choice {
    Choice {
        file: Some(path.to_owned()),
        template: CaptureTemplate {
            key: "n".into(),
            description: "Note".into(),
            kind,
            target,
            template: "literal %?".into(),
            prepend: false,
        },
    }
}
pub(super) fn discover(choice: Choice, buffers: &Buffers) -> Result<Resolved, String> {
    choice
        .discover(buffers, &Org::default(), Date::new(2024, 3, 1).unwrap())?
        .run(&Cancellation::default())
}
pub(super) fn heading(name: &str) -> CaptureTarget {
    CaptureTarget::Headline {
        file: String::new(),
        headline: name.into(),
    }
}
// Discovery qualifies actual disk bytes without opening a buffer or writing a
// heading, date tree, list or table. Container ranges identify parsed structures.
#[test]
fn capture_target_shapes() {
    let f = Fixture::new();
    let buffers = Buffers::new().unwrap();
    for (name, kind, target, source, level, container) in [
        (
            "file",
            CaptureKind::Entry,
            CaptureTarget::File {
                file: String::new(),
            },
            "* Existing\n",
            0,
            "body",
        ),
        (
            "heading",
            CaptureKind::Entry,
            heading("Tasks"),
            "* TODO [#A] Tasks :work:\nbody\n** Child\n",
            1,
            "body",
        ),
        (
            "outline",
            CaptureKind::Plain,
            CaptureTarget::Outline {
                file: String::new(),
                outline: vec!["Work".into(), "Tasks".into()],
            },
            "* Work\n** Tasks\nbody\n* Other\n** Tasks\n",
            2,
            "body",
        ),
        (
            "date tree",
            CaptureKind::Entry,
            CaptureTarget::DateTree {
                file: String::new(),
                outline: vec![],
            },
            "* 2024\n** 2024-03 March\n*** 2024-03-01 Friday\n",
            3,
            "body",
        ),
        (
            "table",
            CaptureKind::TableLine,
            heading("Tasks"),
            "* Tasks\n#+begin_src text\n| literal |\n#+end_src\n| real |\n",
            1,
            "table",
        ),
        (
            "list",
            CaptureKind::Item,
            heading("Tasks"),
            "* Tasks\n1. first\n2. second\n** Child\n",
            1,
            "list",
        ),
        (
            "checkbox",
            CaptureKind::Checkitem,
            heading("Tasks"),
            "* Tasks\n- [ ] first\n",
            1,
            "list",
        ),
    ] {
        let original = format!("\u{feff}{}", source.replace('\n', "\r\n"));
        let path = f.write("target.org", &original);
        let result = discover(choice(&path, kind, target), &buffers)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        assert!(
            matches!(result.source.version, DestinationVersion::Disk(_)),
            "{name}"
        );
        assert!(result.source.bom, "{name}");
        assert_eq!(
            result.source.text().to_string(),
            source.replace('\n', "\r\n"),
            "{name}"
        );
        assert_eq!(result.target.level, level, "{name}");
        assert!(result.target.missing.is_empty(), "{name}");
        let actual = match result.target.container {
            Container::Body => "body",
            Container::Table { .. } => "table",
            Container::List { .. } => "list",
            _ => "missing",
        };
        assert_eq!(actual, container, "{name}");
        assert_eq!(std::fs::read_to_string(path).unwrap(), original, "{name}");
        assert_eq!(buffers.iter().len(), 1, "{name}");
    }
}
// Missing files and structures are plans, never create/visit side effects. The
// returned hierarchy states exactly what a later checked publication must add.
#[test]
fn capture_missing_targets() {
    let f = Fixture::new();
    let buffers = Buffers::new().unwrap();
    let path = f.0.join("absent/notes.org");
    let result = discover(
        choice(&path, CaptureKind::Entry, heading("Tasks")),
        &buffers,
    )
    .unwrap();
    assert_eq!(result.source.version, DestinationVersion::Missing);
    assert_eq!(result.target.missing[0].title, "Tasks");
    assert_eq!(result.target.missing[0].level, 1);
    assert!(!f.0.join("absent").exists());
    let path = f.write("notes.org", "* Journal\nbody\n");
    let target = CaptureTarget::DateTree {
        file: String::new(),
        outline: vec!["Journal".into()],
    };
    let result = discover(choice(&path, CaptureKind::Entry, target), &buffers).unwrap();
    assert_eq!(
        result
            .target
            .missing
            .iter()
            .map(|h| (h.level, h.title.as_str()))
            .collect::<Vec<_>>(),
        vec![(2, "2024"), (3, "2024-03 March"), (4, "2024-03-01 Friday")]
    );
    assert_eq!(result.target.parent, Some(Byte(0)));
    for (name, kind) in [
        ("list", CaptureKind::Item),
        ("table", CaptureKind::TableLine),
    ] {
        let result = discover(choice(&path, kind, heading("Journal")), &buffers).unwrap();
        assert!(
            matches!(
                result.target.container,
                Container::MissingList | Container::MissingTable
            ),
            "{name}"
        );
    }
    assert_eq!(std::fs::read_to_string(path).unwrap(), "* Journal\nbody\n");
}
// Global ambiguous headings and duplicate siblings reject, while a full unique
// path disambiguates. Comma-quoted source-block stars never become target headings.
#[test]
fn capture_target_identity() {
    let f = Fixture::new();
    let buffers = Buffers::new().unwrap();
    let source = "* Work\n** Tasks\n* Home\n** Tasks\n#+begin_src text\n,* Fake\n#+end_src\n";
    let path = f.write("notes.org", source);
    let error = discover(
        choice(&path, CaptureKind::Entry, heading("Tasks")),
        &buffers,
    )
    .err()
    .unwrap();
    assert!(error.contains("Ambiguous"), "{error}");
    let target = CaptureTarget::Outline {
        file: String::new(),
        outline: vec!["Home".into(), "Tasks".into()],
    };
    let result = discover(choice(&path, CaptureKind::Entry, target), &buffers).unwrap();
    assert_eq!(
        result.target.parent,
        Some(Byte(source.rfind("** Tasks").unwrap()))
    );
    let result = discover(choice(&path, CaptureKind::Entry, heading("Fake")), &buffers).unwrap();
    assert_eq!(result.target.missing[0].title, "Fake");
    assert_eq!(std::fs::read_to_string(path).unwrap(), source);
}
// Canonical dirty text wins over disk and aliases, with actual stamp and format
// evidence. Discovery may inspect readonly source but never obtains write rights.
#[test]
fn capture_canonical_source() {
    let f = Fixture::new();
    let path = f.write("notes.org", "\u{feff}* Old\r\n");
    let alias = f.0.join("alias.org");
    std::os::unix::fs::symlink(&path, &alias).unwrap();
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let edit = Replacement {
        source: buffer.span(0..buffer.len_chars()).unwrap(),
        text: "* Tasks\r\nunsaved\r\n".into(),
    };
    let prepared = buffer.prepare(&[edit], &[], None).unwrap();
    buffer.publish(prepared, &mut []).unwrap();
    buffer.set_readonly(true);
    let stamp = buffer.stamp();
    let result = discover(
        choice(&alias, CaptureKind::Plain, heading("Tasks")),
        &buffers,
    )
    .unwrap();
    assert_eq!(result.source.version, DestinationVersion::Buffer(stamp));
    assert!(result.source.readonly && result.source.bom);
    assert_eq!(result.source.text().to_string(), "* Tasks\r\nunsaved\r\n");
    assert_eq!(result.target.parent, Some(Byte(0)));
    assert_eq!(buffers.iter().len(), 2);
    assert_eq!(std::fs::read_to_string(path).unwrap(), "\u{feff}* Old\r\n");
}
