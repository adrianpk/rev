use super::*;
use rev::org::{
    Cancellation,
    block::{Definition, Scan},
};

fn observe(source: &str) -> Result<Definition, String> {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
    let buffer = buffers.get(id).unwrap();
    let at = source.find("#+BEGIN_SRC").unwrap_or(0);
    Scan::new(buffer, Byte(at))
        .unwrap()
        .run(&Cancellation::default())
}
// Header properties inherit independently of general property inheritance;
// local arguments win while unevaluated variables and switches retain data.
#[test]
fn header_precedence() {
    let source = "#+PROPERTY: header-args :exports results :cache yes\n#+PROPERTY: header-args:rust :results silent\n* Parent\n:PROPERTIES:\n:header-args: :exports both :var x=1\n:header-args:rust: :results output\n:END:\n** Child\n:PROPERTIES:\n:header-args: :cache no\n:END:\n#+NAME: demo\n#+HEADER: :results verbatim :var y=\"a :cache yes\"\n#+BEGIN_SRC rust -n 20 :exports code :var z=(list 1 2)\n  fn main() {}\n#+END_SRC\n";
    let d = observe(source).unwrap();
    assert_eq!(
        (d.language.as_str(), d.name.as_deref(), d.switches.as_str()),
        ("rust", Some("demo"), "-n 20")
    );
    for (key, value) in [
        ("exports", "code"),
        ("cache", "no"),
        ("results", "verbatim"),
        ("tangle", "no"),
    ] {
        assert_eq!(d.headers.get(key), Some(value), "{key}");
    }
    assert_eq!(
        d.headers
            .iter()
            .filter(|h| h.key == "var")
            .map(|h| h.value.as_str())
            .collect::<Vec<_>>(),
        ["x=1", "y=\"a :cache yes\"", "z=(list 1 2)"]
    );
    assert_eq!(d.draft, "fn main() {}\n");
}
// Bounded malformed metadata cannot become authority for source reinsertion.
#[test]
fn rejected_blocks() {
    let long = ":".to_owned() + &"a".repeat(4096);
    let nesting = format!(":var x={}1{}", "(".repeat(65), ")".repeat(65));
    for (name, source) in [
        (
            "unterminated",
            "#+BEGIN_SRC rust\nfn main() {}\n".to_owned(),
        ),
        (
            "quote",
            "#+BEGIN_SRC sh :var x=\"open\nbody\n#+END_SRC\n".into(),
        ),
        (
            "delimiters",
            "#+BEGIN_SRC sh :var x=(]\nbody\n#+END_SRC\n".into(),
        ),
        (
            "duplicate name",
            "#+NAME: first\n#+NAME: second\n#+BEGIN_SRC sh\nbody\n#+END_SRC\n".into(),
        ),
        (
            "no language args",
            "#+BEGIN_SRC :cache yes\nbody\n#+END_SRC\n".into(),
        ),
        (
            "long line",
            format!("#+BEGIN_SRC sh {long}\nbody\n#+END_SRC\n"),
        ),
        (
            "deep value",
            format!("#+BEGIN_SRC sh {nesting}\nbody\n#+END_SRC\n"),
        ),
    ] {
        assert!(observe(&source).is_err(), "{name}");
    }
}
// Literal escapes, indentation and newlines round trip exactly on no-op; new
// Org-looking content is escaped before returning to its authoritative source.
#[test]
fn body_reinsertion() {
    let source = "#+BEGIN_SRC org\r\n\t,* literal\r\n\t,#+END_SRC\r\n#+END_SRC\r\ntail";
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let d = Scan::new(buffer, Byte(0))
        .unwrap()
        .run(&Cancellation::default())
        .unwrap();
    assert_eq!(d.draft, "* literal\n#+END_SRC\n");
    assert_eq!(
        d.replacement(buffer, &d.draft).unwrap().text,
        "\t,* literal\r\n\t,#+END_SRC\r\n"
    );
    let edit = d.replacement(buffer, "* new\n#+END_SRC\n").unwrap();
    let prepared = buffer.prepare(&[edit], &[], None).unwrap();
    buffer.publish(prepared, &mut []).unwrap();
    assert_eq!(
        bytes(buffer),
        source.replace("* literal", "* new").as_bytes()
    );
    assert!(d.replacement(buffer, "new").is_err());
}
// Cancellation and finite effective argument bounds refuse observations without
// changing the canonical buffer, history, saved state or source bytes.
#[test]
fn observation_bounds() {
    let source = "#+BEGIN_SRC sh\nexit 99\n#+END_SRC\n";
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
    let buffer = buffers.get(id).unwrap();
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(Scan::new(buffer, Byte(0)).unwrap().run(&cancel).is_err());
    assert_eq!(bytes(buffer), source.as_bytes());
    assert!(buffer.history().is_empty());
    let headers = "#+HEADER: :var x=1\n".repeat(256);
    assert!(observe(&format!("{headers}{source}")).is_err());
}
