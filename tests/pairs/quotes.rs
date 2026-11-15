use super::editor::*;
use super::qualification::{Harness, finish};
use rev::coordinates::Scalar;
use rev::pairs::{Kind, Outcome, Query, Typed, Typing};

// All approved identities use their declared quote rules with colors disabled.
// Both typed admission and existing source proof must agree on quote eligibility.
#[test]
fn profiles() {
    for (language, single, backtick) in [
        ("plain", false, false),
        ("custom-code", false, false),
        ("rust", false, false),
        ("c", true, false),
        ("go", true, true),
        ("javascript", true, false),
        ("typescript", true, false),
        ("tsx", true, false),
        ("json", false, false),
        ("toml", true, false),
        ("css", true, false),
        ("org", false, false),
        ("markdown", false, false),
    ] {
        let mut h = Harness::new(language, false);
        for (kind, admitted) in [
            (Kind::DoubleQuote, true),
            (Kind::SingleQuote, single),
            (Kind::Backtick, backtick),
        ] {
            profile_case(&mut h, (kind, admitted));
        }
    }
}

fn profile_case(h: &mut Harness, case: (Kind, bool)) {
    let (kind, admitted) = case;
    let c = kind.characters().0;
    let id = h.source("");
    let answer = finish(h.buffers.get_mut(id).unwrap(), typed(0, c));
    assert_eq!(
        answer.outcome == Outcome::Typing(Typing::Open { kind, reused: 0 }),
        admitted,
        "{answer:?}"
    );
    let id = h.source(&c.to_string().repeat(2));
    let answer = finish(h.buffers.get_mut(id).unwrap(), Query::At(Scalar(0)));
    assert_eq!(
        matches!(answer.outcome, Outcome::Confirmed(_)),
        admitted,
        "{answer:?}"
    );
}

fn typed(at: usize, character: char) -> Query {
    Query::Typing(Typed {
        point: Scalar(at),
        end: Scalar(at),
        character,
        count: 1,
        skip: true,
    })
}

// Ambiguous literal contexts never gain pairing from a nearby quote or brace.
// These are independent source fixtures, including whole-token prefixes and
// unproved embedded code, rather than highlight-name assertions.
#[test]
fn boundaries() {
    for (language, marked, c) in [
        ("plain", "don|t", '\''),
        ("plain", "word|", '"'),
        ("plain", "|word", '"'),
        ("plain", "\\|", '"'),
        ("rust", "&'a |", '"'),
        ("rust", "r#\"raw |\"#", '"'),
        ("rust", "br#\"raw |\"#", '"'),
        ("rust", "'|", '\''),
        ("c", "/* | */", '\''),
        ("go", "`raw |`", '"'),
        ("javascript", "`template ${|}`", '"'),
        ("typescript", "/regexp |/", '"'),
        ("tsx", "<Item text=|/>", '"'),
        ("toml", "\"\"\"raw |\"\"\"", '"'),
        ("toml", "'''raw |'''", '\''),
        ("toml", "# |", '"'),
        ("css", "/* | */", '"'),
        ("html", "<a |>", '"'),
        ("html", "<script>const x = |</script>", '"'),
        ("html", "<SCRIPT>const x = |</SCRIPT>", '"'),
        ("html", "<style>p { x: | }</style>", '\''),
        ("html", "<!-- | -->", '"'),
        ("org", "=|=", '"'),
        ("org", "~|~", '"'),
        ("org", "#+begin_src rust\n|\n#+end_src", '"'),
        ("org", "src_rust{|}", '"'),
        ("markdown", "`|`", '"'),
        ("markdown", "```rust\n|\n```", '"'),
        ("markdown", "    |", '"'),
    ] {
        let mut h = Harness::new(language, false);
        let byte = marked.find('|').unwrap();
        let at = marked[..byte].chars().count();
        let id = h.source(&marked.replacen('|', "", 1));
        let answer = finish(h.buffers.get_mut(id).unwrap(), typed(at, c));
        assert!(
            !matches!(answer.outcome, Outcome::Typing(_)),
            "{language} {marked}: {answer:?}"
        );
    }
}

// Safe ordinary quote boundaries and eligible existing mates work through real
// physical input. Escaped closers, raw/triple contexts and apostrophes stay literal.
#[test]
fn input() {
    for (language, source, c, expected) in [
        ("json", "{\"key\": |}", '"', "{\"key\": \"|\"}"),
        ("css", "p { x: |; }", '\'', "p { x: '|'; }"),
        ("html", "<a x= |>", '\'', "<a x= '|'>"),
        ("org", "Prose |.", '"', "Prose \"|\"."),
        ("markdown", "Prose |.", '"', "Prose \"|\"."),
        ("rust", "\"x\\|\"", '"', "\"x\\\"|\""),
        ("go", "`x|`", '`', "`x`|"),
        ("c", "'x|'", '\'', "'x'|"),
        ("toml", "x = |\"\"", '"', "x = \"|\"\""),
        ("plain", "don|t", '\'', "don'|t"),
    ] {
        let mut e = Editor::new("", language, source);
        e.type_key(c);
        e.expect(expected);
    }
}

// Fresh syntax can exclude raw/embedded literals, but ordinary C and Go quote
// kinds still come from the recognized profile rather than a generic string label.
#[test]
fn native_info() {
    use rev::pairs::Provenance;
    for (language, marked, c, skip) in [
        ("c", "char c = 'x|';", '\'', true),
        ("go", "package p\nvar x = `x|`", '`', true),
        ("html", "<a x=\"x|\"></a>", '"', true),
        ("rust", "fn main() { let x = r#\"raw |\"#; }", '"', false),
        ("javascript", "const x = `text ${1|}`;", '"', false),
    ] {
        let mut h = Harness::new(language, true);
        let byte = marked.find('|').unwrap();
        let at = marked[..byte].chars().count();
        let id = h.source(&marked.replacen('|', "", 1));
        let answer = finish(h.buffers.get_mut(id).unwrap(), typed(at, c));
        assert_eq!(
            answer.outcome == Outcome::Typing(Typing::Skip(1)),
            skip,
            "{answer:?}"
        );
        assert!(
            matches!(
                answer.provenance,
                Provenance::CurrentSyntax | Provenance::CurrentSyntaxAndLexical
            ),
            "{answer:?}"
        );
    }
}

// Changing the admitted language retires quote proof even when source and
// revision are unchanged. An old C character fact cannot authorize Rust edits.
#[test]
fn language_change() {
    use crate::fixture::Fixture;
    use rev::pairs::Status;
    use std::time::Instant;
    let mut h = Harness::new("c", false);
    let id = h.source("''");
    let b = h.buffers.get_mut(id).unwrap();
    assert!(matches!(
        finish(b, Query::At(Scalar(0))).outcome,
        Outcome::Confirmed(_)
    ));
    let old = b.start_pair_request(Query::At(Scalar(0))).unwrap();
    let stamp = b.stamp();
    let fixture = Fixture::new();
    let settings = rev::config::prepare(&fixture.write(
        "config.toml",
        "syntax.enabled=false\nfiles.associations.'*.case'='rust'",
    ));
    assert!(settings.diagnostic.is_none());
    b.apply_settings(&settings.snapshot);
    assert_eq!(
        b.poll_pair_request(old, Instant::now() + rev::pairs::SLICE_TIME),
        Status::Retired
    );
    let answer = finish(b, Query::At(Scalar(0)));
    assert_eq!(answer.profile, rev::pairs::Profile::Rust);
    assert!(!matches!(answer.outcome, Outcome::Confirmed(_)));
    assert_eq!(b.stamp(), stamp);
    assert!(b.history().is_empty());
}
