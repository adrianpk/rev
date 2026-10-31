use super::*;

// Distinct bundled web parsers supply both frame colors and real declaration
// proposals. Acceptance and undo retain the common checked literal edit owner.
#[test]
fn web_grammars() {
    for (file, mode, document, name) in [
        (
            "source.html",
            "HTML",
            "<article></article>\n<ar|>",
            "article",
        ),
        ("source.css", "CSS", ".sample {}\n.sam| {}", "sample"),
        (
            "source.js",
            "JavaScript",
            "function sample() {}\nfunction main() { sam|; }",
            "sample",
        ),
        (
            "source.ts",
            "TypeScript",
            "function sample(): void {}\nfunction main() { sam|; }",
            "sample",
        ),
        (
            "source.tsx",
            "TSX",
            "function sample(): void {}\nconst view = <div />;\nfunction main() { sam|; }",
            "sample",
        ),
    ] {
        let (_fixture, mut session) = named_source(file, document, "");
        let original = text(&session);
        assert_eq!(session.buffer().unwrap().mode(), mode, "{file}");
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
        assert_eq!(session.buffer().unwrap().history().len(), 1, "{file}");
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), original, "{file}");
    }
}

fn names(session: &Session) -> Vec<&str> {
    session
        .completion()
        .map(|completion| {
            completion
                .candidates()
                .iter()
                .map(|candidate| candidate.text())
                .collect()
        })
        .unwrap_or_default()
}

// Explicit script declarations include aliases, classes/types, members and
// local/parameter bindings. Uses and names from a closed scope are excluded.
#[test]
fn script_scopes() {
    for (file, document, expected) in [
        (
            "scope.js",
            r#"
import sampleDefault, { remote as sampleAlias, sampleImport } from 'package';
import * as sampleNamespace from 'other';
class SampleClass { sampleField = 1; sampleMethod() {} }
function sampleFunction() {}
function other(sampleHidden) { let sampleClosedFunction = 1; }
function main(sampleParameter) {
  const sampleLocal = 1;
  const { sampleDestructure } = {};
  { var sampleFunctionScoped = 2; let sampleClosedBlock = 2; }
  sam|;
  let sampleLater = 3;
}
"#,
            vec![
                "sampleDefault",
                "sampleAlias",
                "sampleImport",
                "sampleNamespace",
                "SampleClass",
                "sampleField",
                "sampleMethod",
                "sampleFunction",
                "sampleParameter",
                "sampleLocal",
                "sampleDestructure",
                "sampleFunctionScoped",
            ],
        ),
        (
            "scope.ts",
            r#"
import sampleDefault, { remote as sampleAlias } from 'package';
interface SampleInterface { sampleField: number; sampleSignature(input: number): void; }
type SampleType = number;
enum SampleEnum { Entry }
class SampleClass { sampleMember: number = 1; sampleMethod(): void {} }
function other(sampleHidden: number): void { let sampleClosedFunction = 1; }
function main(sampleParameter: number, sampleOptional?: number): void {
  let sampleLocal = 1;
  { let sampleClosedBlock = 2; }
  sam|;
  let sampleLater = 3;
}
"#,
            vec![
                "sampleDefault",
                "sampleAlias",
                "SampleInterface",
                "sampleField",
                "sampleSignature",
                "SampleType",
                "SampleEnum",
                "SampleClass",
                "sampleMember",
                "sampleMethod",
                "sampleParameter",
                "sampleOptional",
                "sampleLocal",
            ],
        ),
    ] {
        let (_fixture, mut session) = named_source(file, document, "");
        complete(&mut session);
        let actual = names(&session);
        for name in expected {
            assert!(
                actual.contains(&name),
                "{file}: missing {name} in {actual:?}: {}",
                session.echo()
            );
        }
        for name in [
            "remote",
            "sampleHidden",
            "sampleClosedFunction",
            "sampleClosedBlock",
            "sampleLater",
        ] {
            assert!(!actual.contains(&name), "{file}: leaked {name}");
        }
        assert!(session.buffer().unwrap().history().is_empty());
    }
}

// HTML combines observed names with finite standard names only in tag/attribute
// contexts. Hyphenated names replace as a whole; values and prose remain quiet.
#[test]
fn html_contexts() {
    for (document, expected, absent) in [
        (
            "<custom-card data-user='x'></custom-card>\n<cus|>",
            "custom-card",
            "data-user",
        ),
        (
            "<custom-card data-user='x'></custom-card>\n<div data-u|>",
            "data-user",
            "custom-card",
        ),
        ("<|", "article", "href"),
        ("<div |>", "class", "article"),
        (
            "<custom-card></custom-card>\n</cus|>",
            "custom-card",
            "class",
        ),
    ] {
        let (_fixture, mut session) = named_source("source.html", document, "");
        complete(&mut session);
        let actual = names(&session);
        assert!(
            actual.contains(&expected),
            "{document}: {actual:?}: {}",
            session.echo()
        );
        assert!(!actual.contains(&absent), "{document}");
    }
    for document in [
        "<div class='cl|'></div>",
        "<div>|",
        "<div>arti|</div>",
        "<!-- arti| -->",
        "<script>arti|</script>",
    ] {
        let (_fixture, mut session) = named_source("source.html", document, "");
        complete(&mut session);
        assert!(
            session.completion().is_none(),
            "{document}: {:?}",
            names(&session)
        );
        assert!(session.buffer().unwrap().history().is_empty());
    }
    let (_fixture, mut session) = named_source(
        "source.html",
        "<custom-card></custom-card>\n<custom-c|>",
        "",
    );
    complete(&mut session);
    key(&mut session, KeyCode::Enter, KeyModifiers::NONE);
    assert!(text(&session).ends_with("<custom-card>"));
    session.execute(Command::History(History::Undo));
    assert!(text(&session).ends_with("<custom-c>"));
}

// CSS observes selectors and custom properties, with a finite property-name
// catalog. A var() reference is structural; ordinary values/strings are excluded.
#[test]
fn css_contexts() {
    for (document, expected, absent) in [
        (".sample-card {}\n.sam| {}", "sample-card", "background"),
        ("#sample-card {}\n#sam| {}", "sample-card", "background"),
        ("a { backg|: red; }", "background-color", "a"),
        ("a { | }", "display", "a"),
        (
            "a { --brand-color: red; color: var(--bra|); }",
            "--brand-color",
            "color",
        ),
    ] {
        let (_fixture, mut session) = named_source("source.css", document, "");
        complete(&mut session);
        let actual = names(&session);
        assert!(
            actual.contains(&expected),
            "{document}: {actual:?}: {}",
            session.echo()
        );
        assert!(!actual.contains(&absent), "{document}");
    }
    for document in ["a { color: re|; }", "a { content: 'di|'; }", "/* backg| */"] {
        let (_fixture, mut session) = named_source("source.css", document, "");
        complete(&mut session);
        assert!(
            session.completion().is_none(),
            "{document}: {:?}",
            names(&session)
        );
    }
    let (_fixture, mut session) = named_source(
        "source.css",
        "a { --brand-color: red; color: var(--bra|); }",
        "",
    );
    complete(&mut session);
    key(&mut session, KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(
        text(&session),
        "a { --brand-color: red; color: var(--brand-color); }"
    );
    session.execute(Command::History(History::Undo));
    assert_eq!(
        text(&session),
        "a { --brand-color: red; color: var(--bra); }"
    );
}

// Structural separators bypass the normal prefix minimum; ordinary literals
// stay quiet. JavaScript template expressions retain their normal code context.
#[test]
fn automatic_web_contexts() {
    for (file, document, ch, expected) in [
        ("auto.html", "|", '<', "article"),
        ("auto.html", "<div|>", ' ', "class"),
        ("auto.css", "a |}", '{', "display"),
        ("auto.css", ".sample-card {}\n| {}", '.', "sample-card"),
        (
            "auto.js",
            "function sample() {}\nconst s = `text ${sa|}`;",
            'm',
            "sample",
        ),
        (
            "auto.tsx",
            "function Sample() { return <div />; }\nconst view = <Sa| />;",
            'm',
            "Sample",
        ),
    ] {
        let (_fixture, mut session) = named_source(file, document, "completion.delay_ms = 0");
        key(&mut session, KeyCode::Char(ch), KeyModifiers::NONE);
        settle(&mut session);
        assert!(
            names(&session).contains(&expected),
            "{file} {document}: {:?}: {}",
            names(&session),
            session.echo()
        );
    }
    for (file, document) in [
        ("auto.js", "function sample() {}\n// sa|"),
        ("auto.ts", "function sample() {}\nconst s = 'sa|';"),
        ("auto.js", "function sample() {}\nconst s = `sa|`;"),
        (
            "auto.tsx",
            "function sample() {}\nconst view = <div>sa|</div>;",
        ),
        ("auto.html", "<div title='sa|'></div>"),
        ("auto.css", "a { color: sa|; }"),
    ] {
        let (_fixture, mut session) = named_source(file, document, "completion.delay_ms = 0");
        key(&mut session, KeyCode::Char('m'), KeyModifiers::NONE);
        settle(&mut session);
        assert!(
            session.completion().is_none(),
            "{file} {document}: {:?}",
            names(&session)
        );
    }
}

// TS and TSX share preference identity and observed names while keeping distinct
// parsers. Dollar-prefixed script names preserve the full identifier edit range.
#[test]
fn tsx_sources() {
    let (fixture, mut session) = named_source(
        "source.tsx",
        "function sampleTsx(): void {}\nconst element = <div />;\n// sam|",
        "language.typescript.tab_width = 7",
    );
    assert_eq!(session.buffer().unwrap().mode(), "TSX");
    assert_eq!(session.buffer().unwrap().policy().tab_width, 7);
    assert_eq!(
        session
            .buffer()
            .unwrap()
            .language(&rev::config::Snapshot::default()),
        "typescript"
    );
    let ts = fixture.write("other.ts", "function sampleTs(): void {}\n// sam");
    session.load(&ts).unwrap();
    session.execute(Command::Motion(Motion::BufferEnd));
    assert_eq!(session.buffer().unwrap().mode(), "TypeScript");
    assert_eq!(session.buffer().unwrap().policy().tab_width, 7);
    complete(&mut session);
    assert_eq!(names(&session)[0], "sampleTs");
    assert!(names(&session).contains(&"sampleTsx"));
    let (_fixture, mut session) = named_source(
        "source.js",
        "const $sample = 1;\nfunction main() { $sa|; }",
        "",
    );
    complete(&mut session);
    assert_eq!(names(&session), ["$sample"]);
    key(&mut session, KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(
        text(&session),
        "const $sample = 1;\nfunction main() { $sample; }"
    );
    session.execute(Command::History(History::Undo));
    assert!(text(&session).ends_with("{ $sa; }"));
}
