use super::Fixture;
use rev::{
    buffer::{BufferId, Buffers},
    config::{ExportBackend, Org},
    coordinates::Byte,
    org::{
        Cancellation, Source, StructureKind,
        export::{Options, Request, Scope, Work},
        time::{ZoneEnvironment, ZoneSelection, ZonedOperation},
    },
};
use std::time::UNIX_EPOCH;
fn request(scope: Scope) -> Request {
    Request {
        backend: ExportBackend::Html,
        scope,
        options: None,
        time: ZonedOperation::capture(&ZoneSelection::Utc, &ZoneEnvironment::system(), || {
            Ok(UNIX_EPOCH)
        })
        .unwrap(),
    }
}
fn work(buffers: &Buffers, id: BufferId, scope: Scope) -> Work {
    Work::new(buffers, id, request(scope), &Org::default()).unwrap()
}
// The export layer retains canonical unsaved bytes, native structure and a single
// captured time. It cannot save or alter source history while preparing data.
#[test]
fn export_snapshot_contract() {
    let f = Fixture::new();
    let original = "\u{feff}#+TITLE: Native 界\r\n#+OPTIONS: toc:nil num:nil\r\n* Heading\r\nbody";
    let path = f.write("source.org", original);
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let edit = rev::buffer::edit::Replacement {
        source: buffer.span(buffer.len_chars()..buffer.len_chars()).unwrap(),
        text: " unsaved".into(),
    };
    let plan = buffer.prepare(&[edit], &[], None).unwrap();
    buffer.publish(plan, &mut []).unwrap();
    let stamp = buffer.stamp();
    let snapshot = work(&buffers, id, Scope::Buffer)
        .run(&Cancellation::default())
        .unwrap();
    assert!(snapshot.text().to_string().ends_with("body unsaved"));
    assert_eq!(
        snapshot.sources()[0].version(),
        &rev::org::export::SourceVersion::Buffer(stamp)
    );
    assert_eq!(snapshot.metadata().title.as_deref(), Some("Native 界"));
    assert!(!snapshot.options().toc);
    assert!(!snapshot.options().numbered);
    assert!(
        snapshot
            .elements()
            .any(|n| matches!(n.kind, StructureKind::Heading { level: 1 }))
    );
    assert_eq!(snapshot.request().time.instant().unix_seconds(), 0);
    assert_eq!(buffers.get(id).unwrap().stamp(), stamp);
    assert_eq!(buffers.get(id).unwrap().history().len(), 1);
    assert_eq!(std::fs::read_to_string(path).unwrap(), original);
}
// Region and subtree scope retain exact UTF-8 boundaries; a source range from
// another revision or buffer never becomes export authority over this buffer.
#[test]
fn export_scope_contract() {
    let f = Fixture::new();
    let path = f.write(
        "scope.org",
        "#+TITLE: File\n* One\n界 body\n** Child\ntext\n* Two\nnext\n",
    );
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let buffer = buffers.get(id).unwrap();
    let source = Source::new(buffer);
    let text = source
        .text(&source.range(Byte(0)..Byte(buffer.len_bytes())).unwrap())
        .unwrap()
        .to_string();
    let start = text.find("界").unwrap();
    let region = source.range(Byte(start)..Byte(start + 3)).unwrap();
    let result = work(&buffers, id, Scope::Region(region))
        .run(&Cancellation::default())
        .unwrap();
    assert_eq!(result.text().to_string(), "界");
    let result = work(&buffers, id, Scope::Subtree(Byte(start)))
        .run(&Cancellation::default())
        .unwrap();
    assert_eq!(
        result.text().to_string(),
        "* One\n界 body\n** Child\ntext\n"
    );
    assert_eq!(result.metadata().title.as_deref(), Some("File"));
    assert!(
        work(&buffers, id, Scope::Subtree(Byte(0)))
            .run(&Cancellation::default())
            .is_err()
    );
}
// Typed boolean options and the fixed backend registry reject unsupported forms
// instead of interpreting Lisp, a plugin name or a tool request.
#[test]
fn export_options_contract() {
    for value in ["toc:2", "num:(eval x)", "unknown:t", "toc"] {
        assert!(Options::parse(value).is_err(), "{value}");
    }
    assert!(!Options::parse("tags:0 title:nil").unwrap().tags);
    let f = Fixture::new();
    let path = f.write("disabled.org", "body");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let mut policy = Org::default();
    policy.export.backends = vec![ExportBackend::Org];
    policy.export.default_format = ExportBackend::Org;
    assert!(Work::new(&buffers, id, request(Scope::Buffer), &policy).is_err());
}
