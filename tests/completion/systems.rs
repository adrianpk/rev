use super::*;

// Each bundled systems grammar drives both real native highlighting and manual
// declaration extraction. Acceptance stays one canonical, reversible edit.
#[test]
fn systems_grammars() {
    for (file, mode, document, name) in [
        (
            "source.rs",
            "Rust",
            "fn sample() {}\nfn main() { sam| }",
            "sample",
        ),
        (
            "source.c",
            "C",
            "int sample(void) { return 1; }\nint main(void) { sam|; }",
            "sample",
        ),
        (
            "source.go",
            "Go",
            "package main\nfunc sample() {}\nfunc main() { sam| }",
            "sample",
        ),
    ] {
        let (_fixture, mut session) = named_source(file, document, "");
        let original = text(&session);
        assert_eq!(session.buffer().unwrap().mode(), mode, "{file}");
        let buffer = session.buffer().unwrap();
        assert!(
            !buffer
                .syntax()
                .captures(buffer, 0..buffer.len_bytes())
                .is_empty(),
            "{file}"
        );
        assert!(
            session
                .frame()
                .unwrap()
                .rows()
                .iter()
                .any(|row| !row.syntax().is_empty()),
            "{file}"
        );
        complete(&mut session);
        let completion = session
            .completion()
            .unwrap_or_else(|| panic!("{file}: {}", session.echo()));
        assert_eq!(completion.candidates()[0].text(), name, "{file}");
        key(&mut session, KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(
            text(&session),
            original.replace("{ sam", &format!("{{ {name}"))
        );
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), original);
    }
}

// Other open buffers contribute their actual unsaved revision. Neither an
// unopened sibling nor a different language identity becomes an implicit index.
#[test]
fn open_sources() {
    let (fixture, mut session) = source("fn sample_current() {}\n// sam|");
    let current = fixture.0.join("source.rs");
    let other = fixture.write("other.rs", "fn sample_disk() {}\n");
    fixture.write("unopened.rs", "fn sample_unopened() {}\n");
    let foreign = fixture.write("foreign.go", "package main\nfunc sample_foreign() {}\n");
    session.load(&foreign).unwrap();
    session.load(&other).unwrap();
    session.execute(Command::Motion(Motion::BufferEnd));
    session.handle(Event::Paste("fn sample_unsaved() {}\n".into()));
    session.load(&current).unwrap();
    session.execute(Command::Motion(Motion::BufferEnd));
    complete(&mut session);
    let names: Vec<_> = session
        .completion()
        .unwrap()
        .candidates()
        .iter()
        .map(|c| c.text())
        .collect();
    assert_eq!(names[0], "sample_current");
    assert!(names.contains(&"sample_disk"));
    assert!(names.contains(&"sample_unsaved"));
    assert!(!names.contains(&"sample_unopened"));
    assert!(!names.contains(&"sample_foreign"));
    assert!(
        !std::fs::read_to_string(other)
            .unwrap()
            .contains("sample_unsaved")
    );
    assert!(session.buffer().unwrap().history().is_empty());
}

// Real declaration queries distinguish names from uses and keep local/parameter
// bindings in their lexical scope, including destructuring and C declarators.
#[test]
fn declaration_scopes() {
    let cases = [
        (
            "rust",
            "scope.rs",
            r#"
struct SampleType { sample_field: i32 }
trait SampleTrait { fn sample_method(&self) {} }
mod sample_module {}
const SAMPLE_CONST: i32 = 1;
enum SampleEnum { SampleVariant }
type SampleAlias = i32;
fn other(sample_hidden: i32) { let sample_hidden_local = 0; }
fn main(sample_param: i32) {
    let (sample_tuple, sample_pair) = (1, 2);
    let Some(sample_option) = Some(1) else { return; };
    { let sample_closed = 0; }
    let sample_local = 0;
    sa|;
    let sample_later = 0;
}
"#,
            vec![
                "SampleType",
                "sample_field",
                "SampleTrait",
                "sample_method",
                "sample_module",
                "SAMPLE_CONST",
                "SampleEnum",
                "SampleVariant",
                "SampleAlias",
                "sample_param",
                "sample_tuple",
                "sample_pair",
                "sample_option",
                "sample_local",
            ],
        ),
        (
            "c",
            "scope.c",
            r#"
typedef int SampleAlias;
struct SampleType { int sample_field; };
enum SampleEnum { SAMPLE_CONST };
int sample_function(int sample_hidden) { int sample_hidden_local; return 0; }
int main(int sample_param) {
    int sample_local = 0;
    int *sample_pointer;
    int sample_array[2];
    int (*sample_callback)(int);
    { int sample_closed; }
    sa|;
    int sample_later;
}
"#,
            vec![
                "SampleAlias",
                "SampleType",
                "sample_field",
                "SampleEnum",
                "SAMPLE_CONST",
                "sample_function",
                "sample_param",
                "sample_local",
                "sample_pointer",
                "sample_array",
                "sample_callback",
            ],
        ),
        (
            "go",
            "scope.go",
            r#"
package main
type SampleType struct { SampleField int }
type SampleAlias = SampleType
const SampleConst = 1
var SampleGlobal = 2
func (receiver SampleType) SampleMethod() {}
func other(sample_hidden int) { sample_hidden_local := 0 }
func main(sample_param int) {
    sample_local := 1
    var sample_variable int
    { sample_closed := 0 }
    sa|
    sample_later := 1
}
"#,
            vec![
                "SampleType",
                "SampleField",
                "SampleAlias",
                "SampleConst",
                "SampleGlobal",
                "SampleMethod",
                "sample_param",
                "sample_local",
                "sample_variable",
            ],
        ),
    ];
    for (name, file, document, expected) in cases {
        let (_fixture, mut session) = named_source(file, document, "");
        complete(&mut session);
        let actual: Vec<_> = session
            .completion()
            .unwrap_or_else(|| panic!("{name}: {}", session.echo()))
            .candidates()
            .iter()
            .map(|candidate| candidate.text())
            .collect();
        for expected in expected {
            assert!(
                actual.contains(&expected),
                "{name}: missing {expected} in {actual:?}"
            );
        }
        for hidden in [
            "sample_hidden",
            "sample_hidden_local",
            "sample_closed",
            "sample_later",
        ] {
            assert!(!actual.contains(&hidden), "{name}: leaked {hidden}");
        }
        assert!(session.buffer().unwrap().history().is_empty(), "{name}");
    }
}

// Broken and unsupported source cannot manufacture declarations or expand C
// macros; valid independent declarations remain useful during incomplete edits.
#[test]
fn incomplete_source() {
    for (file, document) in [
        ("partial.rs", "fn sample_good() {}\nfn sample_bad(\n// sam|"),
        (
            "partial.c",
            "#define sample_macro 1\nint sample_good(void) {}\nint sample_bad(\n// sam|",
        ),
        (
            "partial.go",
            "package main\nfunc sample_good() {}\nfunc sample_bad(\n// sam|",
        ),
    ] {
        let (_fixture, mut session) = named_source(file, document, "");
        complete(&mut session);
        let actual: Vec<_> = session
            .completion()
            .unwrap_or_else(|| panic!("{file}: {}", session.echo()))
            .candidates()
            .iter()
            .map(|candidate| candidate.text())
            .collect();
        assert_eq!(actual, ["sample_good"], "{file}");
        assert!(session.buffer().unwrap().history().is_empty());
    }
    for (file, document) in [
        ("local.rs", "fn main() { let sample_local = 1;\n sam|"),
        ("local.c", "int main(void) { int sample_local = 1;\n sam|"),
        (
            "local.go",
            "package main\nfunc main() { sample_local := 1\n sam|",
        ),
    ] {
        let (_fixture, mut session) = named_source(file, document, "");
        complete(&mut session);
        assert!(
            session
                .completion()
                .unwrap_or_else(|| panic!("{file}: {}", session.echo()))
                .candidates()
                .iter()
                .any(|candidate| candidate.text() == "sample_local"),
            "{file}"
        );
    }
}

// Conditional pattern names belong to their admitted body, and a function's
// locals never become cross-buffer proposals. Duplicate literal edits collapse.
#[test]
fn local_visibility() {
    let (fixture, mut session) = source("fn sample_shared() {}\nfn main() { sa|; }");
    let current = fixture.0.join("source.rs");
    let other = fixture.write("other.rs", "const sample_shared: i32 = 1;\nfn other(sample_parameter: i32) { let sample_private = 0; }\n");
    session.load(&other).unwrap();
    session.load(&current).unwrap();
    session.execute(Command::Motion(Motion::BufferStart));
    for _ in 0.."fn sample_shared() {}\nfn main() { sa".chars().count() {
        session.execute(Command::Motion(Motion::ForwardChar));
    }
    complete(&mut session);
    let candidates = session.completion().unwrap().candidates();
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].label(), "sample_shared");
    assert_eq!(candidates[0].kind(), rev::session::CompletionKind::Function);
    assert_eq!(candidates[0].detail(), "function");
    for document in [
        "fn main() { if let Some(sample_branch) = Some(1) { sa|; } else {} }",
        "fn main() { if let Some(sample_branch) = Some(1) {} else { sa|; } }",
    ] {
        let (_fixture, mut session) = source(document);
        complete(&mut session);
        assert_eq!(
            session.completion().is_some(),
            document.contains("{ sa|; } else")
        );
    }
}

// C and Go retain the same automatic literal suppression, while C's pointer
// separator admits an empty prefix without requiring receiver type knowledge.
#[test]
fn native_contexts() {
    for (file, document) in [
        ("literal.c", "int sample(void) {}\n// sa|"),
        ("literal.go", "package main\nfunc sample() {}\n// sa|"),
        ("string.c", "int sample(void) {}\nchar *s = \"sa|\";"),
        (
            "string.go",
            "package main\nfunc sample() {}\nvar s = \"sa|\"",
        ),
    ] {
        let (_fixture, mut session) = named_source(file, document, "completion.delay_ms = 0");
        key(&mut session, KeyCode::Char('m'), KeyModifiers::NONE);
        settle(&mut session);
        assert!(session.completion().is_none(), "{file}");
        complete(&mut session);
        assert_eq!(
            session.completion().unwrap().candidates()[0].text(),
            "sample",
            "{file}"
        );
    }
    let (_fixture, mut session) = named_source(
        "pointer.c",
        "struct Sample { int field; };\nint main() { unknown-|; }",
        "completion.delay_ms = 0",
    );
    key(&mut session, KeyCode::Char('>'), KeyModifiers::NONE);
    settle(&mut session);
    assert!(
        session
            .completion()
            .unwrap()
            .candidates()
            .iter()
            .any(|candidate| candidate.text() == "field")
    );
}

// Admission/extraction limits report their cause rather than claiming a complete
// no-match result; large inputs and names leave source and history untouched.
#[test]
fn source_limits() {
    let mut many = (0..4100)
        .map(|n| format!("fn sample_{n}() {{}}\n"))
        .collect::<String>();
    many.push_str("// sam|");
    let cases = [
        (
            "buffer",
            format!("//{}\n// sam|", "x".repeat(2 * 1024 * 1024)),
            "2 MiB",
        ),
        ("symbols", many, "limit"),
        (
            "label",
            format!("fn sample_{}() {{}}\n// sam|", "x".repeat(512)),
            "label limit",
        ),
    ];
    for (name, document, message) in cases {
        let (_fixture, mut session) = source(&document);
        let original = text(&session);
        complete(&mut session);
        assert!(session.completion().is_none(), "{name}");
        assert!(
            session.echo().contains(message),
            "{name}: {}",
            session.echo()
        );
        assert_eq!(text(&session), original, "{name}");
        assert!(session.buffer().unwrap().history().is_empty(), "{name}");
    }
}
