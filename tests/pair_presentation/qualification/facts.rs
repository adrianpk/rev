use super::{Harness, complete, request, view};
use rev::pairs::{Kind, PresentationOutcome as Outcome, Profile, Provenance};

// Every endpoint is checked against an independent scalar/mate/depth oracle,
// including wide and combining graphemes and CRLF source offsets.
#[test]
fn structural_depth() {
    let mut h = Harness::new("plain", false);
    for source in ["([{}])", "(界[e\u{301}{🙂}])\r\n"] {
        let id = h.source(source);
        let b = h.buffers.get_mut(id).unwrap();
        let ticket = request(b, view(b));
        let stamp = b.stamp();
        let state = b.state();
        let maximum = complete(b, ticket);
        let answer = b.pair_presentation(ticket).unwrap();
        assert_eq!(answer.outcome, Outcome::Complete);
        assert_eq!(answer.profile, Profile::Plain);
        assert_eq!(answer.stamp, stamp);
        assert_eq!(answer.spans.len(), 6);
        let expected: Vec<_> = source
            .chars()
            .enumerate()
            .filter(|(_, c)| "()[]{}".contains(*c))
            .collect();
        for (i, span) in answer.spans.iter().enumerate() {
            assert_eq!(span.at.0, expected[i].0);
            assert_eq!(span.mate.0, expected[5 - i].0);
            assert_eq!(span.depth, [0, 1, 2, 2, 1, 0][i]);
            assert_eq!(
                span.kind,
                [
                    Kind::Parenthesis,
                    Kind::Bracket,
                    Kind::Brace,
                    Kind::Brace,
                    Kind::Bracket,
                    Kind::Parenthesis
                ][i]
            );
            assert_eq!(span.opening, i < 3);
        }
        assert_eq!(b.state(), state);
        assert_eq!(b.stamp(), stamp);
        assert!(b.history().is_empty() && !b.is_modified());
        eprintln!(
            "qualification structural: {} bytes, {} retained, max poll {maximum:?}",
            source.len(),
            b.retained_pair_bytes()
        );
    }
}

// Native identity drives fallback even without colors or an available grammar.
// Comments/literal delimiters never become structural proof; eligible quotes do.
#[test]
fn lexical_profiles() {
    for (language, source, count, quotes) in [
        ("plain", "([x]) <tag>", 4, 0),
        ("custom-code", "([x])", 4, 0),
        ("rust", "// [excluded]\n{\"(literal)\"}", 4, 2),
        ("c", "/* (excluded) */ [\"{literal}\"]", 4, 2),
        ("go", "// (excluded)\n{`[literal]`}", 4, 2),
        ("javascript", "/* [excluded] */ ({})", 4, 0),
        ("json", "{\"key\": [1]}", 6, 2),
        ("toml", "# [excluded]\na=[1]", 2, 0),
        ("css", "/* (excluded) */ a { x: (b); }", 4, 0),
        ("html", "<!-- [excluded] --> (x)", 2, 0),
        (
            "org",
            "* Heading\nBody ([x])\n#+begin_src rust\n{excluded}\n#+end_src\n",
            4,
            0,
        ),
        (
            "markdown",
            "# Heading\nBody ([x])\n```rust\n{excluded}\n```\n",
            4,
            0,
        ),
    ] {
        for colors in [false, true] {
            let mut h = Harness::new(language, colors);
            let id = h.source(source);
            let b = h.buffers.get_mut(id).unwrap();
            let ticket = request(b, view(b));
            complete(b, ticket);
            let result = b.pair_presentation(ticket).unwrap();
            assert_eq!(result.outcome, Outcome::Complete, "{language}, {colors}");
            assert_eq!(
                result.spans.len(),
                count,
                "{language}, {colors}: {result:?}"
            );
            assert_eq!(
                result.spans.iter().filter(|s| s.kind.quote()).count(),
                quotes
            );
            assert_eq!(result.provenance, Provenance::Lexical);
        }
    }
}

// Only a typed wrong closer is a mismatch. Missing mates and ambiguous lexical
// constructs carry distinct conservative outcomes, with no invented endpoint.
#[test]
fn malformed_and_uncertain() {
    for (language, source, expected, spans) in [
        (
            "plain",
            "([)]",
            Outcome::Mismatched {
                expected: Kind::Bracket,
                found: Kind::Parenthesis,
            },
            0,
        ),
        (
            "plain",
            "() {]",
            Outcome::Mismatched {
                expected: Kind::Brace,
                found: Kind::Bracket,
            },
            2,
        ),
        ("plain", "(", Outcome::Unmatched, 0),
        ("plain", ")", Outcome::Unmatched, 0),
        (
            "rust",
            "\"unterminated",
            Outcome::Uncertain(rev::pairs::Reason::LiteralContext),
            0,
        ),
        (
            "javascript",
            "/regex/",
            Outcome::Uncertain(rev::pairs::Reason::AmbiguousSlash),
            0,
        ),
    ] {
        let mut h = Harness::new(language, false);
        let id = h.source(source);
        let b = h.buffers.get_mut(id).unwrap();
        let ticket = request(b, view(b));
        complete(b, ticket);
        let result = b.pair_presentation(ticket).unwrap();
        assert_eq!(result.outcome, expected, "{language}: {source}");
        assert_eq!(result.spans.len(), spans);
    }
}
