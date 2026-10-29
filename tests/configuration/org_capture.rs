use super::fixture::Fixture;
use rev::{
    config::{CaptureKind, CaptureTarget, CaptureTemplate, Settings, prepare},
    org::capture::Choice,
};

// Factory values and empty-array reset preserve the reference Task fallback,
// while the empty destination remains a real request for explicit user input.
#[test]
fn capture_factory() {
    let f = Fixture::new();
    let startup = prepare(&f.write("config.toml", "org.capture.templates=[]"));
    assert!(startup.diagnostic.is_none());
    assert_eq!(
        startup.snapshot.settings().org.capture,
        Settings::default().org.capture
    );
    let mut choice = Choice::configured(&startup.snapshot, "t").unwrap();
    assert_eq!(choice.template, CaptureTemplate::task());
    assert_eq!(choice.template.text(), "* TODO %?\n  %u\n  %a");
    assert!(choice.file.is_none());
    assert!(choice.enter_file("", &f.0).is_err());
    choice.enter_file("uncreated/inbox.org", &f.0).unwrap();
    assert_eq!(choice.file, Some(f.0.join("uncreated/inbox.org")));
    assert!(!f.0.join("uncreated").exists());
    assert!(Choice::configured(&startup.snapshot, "unknown").is_err());
}
// Each native type/target is data only. Invalid shapes, duplicate keys and
// exact/over field bounds cannot enter the resolved configuration snapshot.
#[test]
fn capture_admission() {
    let f = Fixture::new();
    for (name, target) in [
        ("file", "{type='file',file='notes.org'}"),
        ("headline", "{type='file+headline',headline='Tasks'}"),
        ("outline", "{type='file+olp',outline=['Work','Tasks']}"),
        ("date tree", "{type='file+olp+datetree',outline=[]}"),
    ] {
        let value =
            format!("org.capture.templates=[{{key='t',description='Task',target={target}}}]");
        let startup = prepare(&f.write("valid.toml", &value));
        assert!(
            startup.diagnostic.is_none(),
            "{name}: {:?}",
            startup.diagnostic
        );
    }
    for (name, definition) in [
        ("unknown type", "kind='function',target={type='file'}"),
        (
            "executable target",
            "target={type='function',function='run'}",
        ),
        ("unknown property", "target={type='file'},hook='run'"),
        ("target extra", "target={type='file',headline='ignored'}"),
        ("empty outline", "target={type='file+olp',outline=[]}"),
        ("empty heading", "target={type='file+headline',headline=''}"),
        (
            "remote path",
            "target={type='file',file='https://example.test/file'}",
        ),
    ] {
        let source = format!("org.capture.templates=[{{key='t',description='Task',{definition}}}]");
        assert!(
            prepare(&f.write("bad.toml", &source)).diagnostic.is_some(),
            "{name}"
        );
    }
    let mut template = CaptureTemplate::task();
    template.template = "x".repeat(65536);
    assert!(template.validate().is_ok());
    template.template.push('x');
    assert!(template.validate().is_err());
}
// Included arrays replace as leaves, independently of the default-file leaf.
// Their paths retain their own declaring directories; no destination is read.
#[test]
fn capture_source_roots() {
    let f = Fixture::new();
    f.write("base.toml", "org.capture.default_file='default.org'");
    f.write("nested/templates.toml", "org.capture.templates=[{key='n',description='Note',kind='plain',target={type='file',file='local.org'}}]");
    let startup = prepare(&f.write(
        "config.toml",
        "include=['base.toml','nested/templates.toml']",
    ));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let local = Choice::configured(&startup.snapshot, "n").unwrap();
    assert_eq!(local.file, Some(f.0.join("nested/local.org")));
    assert_eq!(local.template.kind, CaptureKind::Plain);
    let reset = prepare(&f.write(
        "reset.toml",
        "include=['config.toml']\norg.capture.templates=[]",
    ));
    assert!(reset.diagnostic.is_none());
    let choice = Choice::configured(&reset.snapshot, "t").unwrap();
    assert_eq!(choice.file, Some(f.0.join("default.org")));
    assert!(matches!(
        choice.template.target,
        CaptureTarget::Headline { .. }
    ));
    assert!(!f.0.join("default.org").exists());
    assert!(!f.0.join("nested/local.org").exists());
}
