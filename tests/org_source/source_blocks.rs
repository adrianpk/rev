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
        ("ambiguous name","#+NAME: same\n#+BEGIN_SRC sh\nx\n#+END_SRC\n#+NAME: same\n#+BEGIN_SRC sh\ny\n#+END_SRC\n".into()),
        ("duplicate property","* A\n:PROPERTIES:\n:header-args: :cache yes\n:header-args: :cache no\n:END:\n#+BEGIN_SRC sh\nx\n#+END_SRC\n".into()),
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

// Ordered additive defaults retain generic and language-specific header data
// without needing the ordinary property inheritance configuration.
#[test]
fn additive_headers() {
    let source = "#+PROPERTY: header-args :cache yes\n#+PROPERTY: header-args+ :exports both\n* A\n:PROPERTIES:\n:header-args:rust: :results output\n:header-args:rust+: :noweb yes\n:END:\n#+BEGIN_SRC rust\nbody\n#+END_SRC\n";
    let d = observe(source).unwrap();
    for (key, value) in [
        ("cache", "yes"),
        ("exports", "both"),
        ("results", "output"),
        ("noweb", "yes"),
    ] {
        assert_eq!(d.headers.get(key), Some(value), "{key}");
    }
}

// A header prompt admits one literal argument only; rejected input cannot
// smuggle a second argument or line into the block opening.
#[test]
fn header_argument_admission() {
    let source = "#+BEGIN_SRC sh\nexit 99\n#+END_SRC\n";
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
    let b = buffers.get(id).unwrap();
    let d = Scan::new(b, Byte(0))
        .unwrap()
        .run(&Cancellation::default())
        .unwrap();
    for (name, value) in [
        ("empty", ""),
        ("missing key", "cache yes"),
        ("multiple", ":cache yes :exports no"),
        ("newline", ":var x=1\n"),
        ("unclosed", ":var x=(1"),
        ("control", ":var x=\u{0001}"),
    ] {
        assert!(d.header_replacement(b, value).is_err(), "{name}");
    }
    let edit = d.header_replacement(b, ":var x=\"a :cache yes\"").unwrap();
    assert_eq!(edit.text, "#+BEGIN_SRC sh :var x=\"a :cache yes\"\n");
    assert_eq!(bytes(b), source.as_bytes());
    assert!(b.history().is_empty());
}
// Local header insertion and body return must fit the final admissible source,
// rather than validating only the separately entered argument or draft body.
#[test]
fn publication_bounds() {
    let source = "#+BEGIN_SRC sh\nbody\n#+END_SRC\n";
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
    let b = buffers.get(id).unwrap();
    let d = Scan::new(b, Byte(0))
        .unwrap()
        .run(&Cancellation::default())
        .unwrap();
    let body = "x".repeat(8 * 1024 * 1024 - 1) + "\n";
    assert!(d.replacement(b, &body).is_err());
    let capacity = 8 * 1024 * 1024 - source.len() + d.body.len();
    let at_bound = "x".repeat(capacity - 1) + "\n";
    assert!(d.replacement(b, &at_bound).is_ok());
    for (name, args) in [
        ("line", format!(":var x={}", "1".repeat(4080))),
        ("argument count", ":var x=1 ".repeat(249)),
    ] {
        let source = format!("#+BEGIN_SRC sh {args}\nbody\n#+END_SRC\n");
        let id = buffers.create(&source).unwrap();
        let b = buffers.get(id).unwrap();
        let d = Scan::new(b, Byte(0))
            .unwrap()
            .run(&Cancellation::default())
            .unwrap();
        assert!(d.header_replacement(b, ":var extra=2").is_err(), "{name}");
        assert!(
            d.header_replacement(b, ":cache n").is_ok(),
            "{name} boundary"
        );
        assert_eq!(bytes(b), source.as_bytes());
        assert!(b.history().is_empty());
    }
}
