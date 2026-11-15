use std::time::{Duration, Instant};

use rev::buffer::{Buffer, BufferId, Buffers};
use rev::coordinates::Scalar;
use rev::pairs::{
    Answer, Context, Kind, Limit, MAX_BYTES, MAX_DEPTH, MAX_METADATA, Outcome, Provenance, Query,
    SLICE_TIME, SLICE_TOKENS, Status, Ticket,
};

use crate::fixture::Fixture;

pub(super) struct Harness {
    fixture: Fixture,
    pub(super) buffers: Buffers,
}

impl Harness {
    pub(super) fn new(language: &str, colors: bool) -> Self {
        let fixture = Fixture::new();
        let config = format!("syntax.enabled={colors}\nfiles.associations.'*.case'='{language}'");
        let root = fixture.write("config.toml", &config);
        let startup = rev::config::prepare(&root);
        assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
        let buffers = Buffers::with_settings(startup.snapshot).unwrap();
        Self { fixture, buffers }
    }

    pub(super) fn source(&mut self, source: &str) -> BufferId {
        let name = format!("source-{}.case", self.buffers.iter().len());
        let path = self.fixture.write(&name, source);
        self.buffers.visit(&path).unwrap()
    }
}

pub(super) fn finish(buffer: &mut Buffer, query: Query) -> Answer {
    let ticket = buffer.start_pair_request(query).unwrap();
    complete(buffer, ticket)
}

fn complete(buffer: &mut Buffer, ticket: Ticket) -> Answer {
    timed_complete(buffer, ticket).0
}

fn timed_complete(buffer: &mut Buffer, ticket: Ticket) -> (Answer, Duration) {
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut maximum = Duration::ZERO;
    while Instant::now() < deadline {
        let before = Instant::now();
        let status = buffer.poll_pair_request(ticket, before + SLICE_TIME);
        maximum = maximum.max(before.elapsed());
        match status {
            Status::Pending => (),
            Status::Ready => {
                let answer = buffer.pair_answer(ticket).unwrap().clone();
                assert!(answer.stats.inspected_bytes <= MAX_BYTES);
                assert!(answer.stats.retained_bytes <= MAX_METADATA);
                assert!(answer.stats.maximum_slice_tokens <= SLICE_TOKENS);
                return (answer, maximum);
            }
            Status::Retired => panic!("live pair request was retired"),
        }
    }
    panic!("pair request exceeded its finite test deadline");
}

fn marked(source: &str) -> (String, Scalar) {
    let byte = source.find('|').unwrap();
    let point = Scalar(source[..byte].chars().count());
    (source.replacen('|', "", 1), point)
}

fn pair(answer: &Answer, expected: (Kind, usize, usize, usize)) {
    assert_eq!(answer.policy_version, rev::pairs::POLICY_VERSION);
    let Outcome::Confirmed(pair) = &answer.outcome else {
        panic!("expected typed pair: {answer:?}")
    };
    assert_eq!(pair.kind, expected.0);
    assert_eq!(pair.opening.range, expected.1..expected.1 + 1);
    assert_eq!(pair.closing.range, expected.2..expected.2 + 1);
    assert_eq!(pair.opening.stamp, answer.stamp);
    assert_eq!(pair.closing.stamp, answer.stamp);
    assert_eq!(pair.depth, Some(expected.3));
}

// Mixed kinds are checked together. A loose same-kind scan could claim a mate
// across a crossing delimiter; typed qualification must report the mismatch.
#[test]
fn strict_pairs() {
    let mut buffers = Buffers::new().unwrap();
    for (name, source, point, expected) in [
        ("outer", "([{}])", 0, (Kind::Parenthesis, 0, 5, 0)),
        ("nested", "([{}])", 1, (Kind::Bracket, 1, 4, 1)),
        ("closer", "([{}])", 3, (Kind::Brace, 2, 3, 2)),
        ("Unicode", "(界🙂)", 0, (Kind::Parenthesis, 0, 3, 0)),
        ("CRLF", "(\r\n界)\r\n", 4, (Kind::Parenthesis, 0, 4, 0)),
        ("combining", "[e\u{301}]", 0, (Kind::Bracket, 0, 3, 0)),
    ] {
        let id = buffers.create(source).unwrap();
        let answer = finish(buffers.get_mut(id).unwrap(), Query::At(Scalar(point)));
        pair(&answer, expected);
        assert_eq!(answer.provenance, Provenance::Lexical, "{name}");
    }
    for (name, source, expected) in [
        (
            "crossed",
            "([)]",
            Outcome::Mismatched {
                expected: Kind::Bracket,
                found: Kind::Parenthesis,
            },
        ),
        (
            "wrong closer",
            "{]",
            Outcome::Mismatched {
                expected: Kind::Brace,
                found: Kind::Bracket,
            },
        ),
        ("missing closer", "(", Outcome::Unmatched),
        ("missing opener", ")", Outcome::Unmatched),
    ] {
        let id = buffers.create(source).unwrap();
        let answer = finish(buffers.get_mut(id).unwrap(), Query::At(Scalar(0)));
        assert_eq!(answer.outcome, expected, "{name}");
    }
}

// Ordinary punctuation works for every admitted identity without syntax colors.
// Inner lookup uses native source positions and never invents an angle pair.
#[test]
fn language_profiles() {
    for language in [
        "plain",
        "rust",
        "c",
        "go",
        "javascript",
        "typescript",
        "tsx",
        "json",
        "toml",
        "css",
        "html",
        "org",
        "markdown",
        "custom-code",
    ] {
        let mut h = Harness::new(language, false);
        let id = h.source("([x])");
        let b = h.buffers.get_mut(id).unwrap();
        let answer = finish(b, Query::Enclosing(Scalar(2)));
        pair(&answer, (Kind::Bracket, 1, 3, 1));
        assert_eq!(answer.provenance, Provenance::Lexical, "{language}");
        let id = h.source("<x>");
        assert!(
            matches!(
                finish(h.buffers.get_mut(id).unwrap(), Query::At(Scalar(0))).outcome,
                Outcome::Uncertain(_)
            ),
            "{language}"
        );
    }
}

// These boundaries must remain literal in the declared fallback profile. Each
// case also verifies context queries without relying on syntax highlight names.
#[test]
fn literal_contexts() {
    for (language, cases) in [
        (
            "rust",
            vec!["// |()", "/* outer /* nested */ |() */", "\"|()\""],
        ),
        (
            "c",
            vec![
                "// |()",
                "/* |() */",
                "'|('\n",
                "\"|()\"",
                "// \\\r\n|()\r\n",
            ],
        ),
        ("go", vec!["`|()`", "// |()", "'|('"]),
        (
            "javascript",
            vec!["// |()", "/* |() */", "'|()'", "`x ${|()}`", "/|()/"],
        ),
        ("toml", vec!["# |()", "'|()'", "\"\"\"|()\"\"\""]),
        (
            "html",
            vec!["<!-- |() -->", "<a x=\"|()\">", "<script>|()</script>"],
        ),
        (
            "org",
            vec![
                "=|()=",
                "~|()~",
                "#+begin_src rust\n|()\n#+end_src",
                "src_rust{|()}",
            ],
        ),
        (
            "markdown",
            vec![
                "`|()`",
                "```rust\n|()\n```",
                "~~~rust\n|()\n~~~",
                "````\n|()\n````",
            ],
        ),
    ] {
        let mut h = Harness::new(language, false);
        for case in cases {
            let (source, point) = marked(case);
            let id = h.source(&source);
            let answer = finish(h.buffers.get_mut(id).unwrap(), Query::At(point));
            assert!(
                matches!(answer.outcome, Outcome::Uncertain(_)),
                "{language}: {case}: {answer:?}"
            );
        }
    }
}

// Quote kinds follow the recognized language rather than a generic toggle or
// grammar availability; apostrophes, raw/triple and template ambiguity refuse.
#[test]
fn quote_profiles() {
    for (language, source, point, kind, closing) in [
        ("plain", "\"x\"", 0, Kind::DoubleQuote, 2),
        ("rust", "\"x\"", 0, Kind::DoubleQuote, 2),
        ("c", "'x'", 0, Kind::SingleQuote, 2),
        ("go", "`x`", 0, Kind::Backtick, 2),
        ("javascript", "'x'", 0, Kind::SingleQuote, 2),
        ("typescript", "\"x\"", 0, Kind::DoubleQuote, 2),
        ("json", "\"x\"", 0, Kind::DoubleQuote, 2),
        ("toml", "'x'", 0, Kind::SingleQuote, 2),
        ("css", "'x'", 0, Kind::SingleQuote, 2),
        ("html", "<a x=\"x\">", 5, Kind::DoubleQuote, 7),
        ("org", "\"x\"", 0, Kind::DoubleQuote, 2),
        ("markdown", "\"x\"", 0, Kind::DoubleQuote, 2),
    ] {
        let mut h = Harness::new(language, false);
        let id = h.source(source);
        pair(
            &finish(h.buffers.get_mut(id).unwrap(), Query::At(Scalar(point))),
            (kind, point, closing, 0),
        );
    }
    for (language, source, point) in [
        ("plain", "can't", 3),
        ("rust", "'x'", 0),
        ("rust", "&'a ()", 4),
        ("rust", "r#\"()\"#", 3),
        ("javascript", "`()`", 0),
        ("json", "'x'", 0),
        ("toml", "'''()'''", 0),
        ("html", "\"x\"", 0),
        ("plain", "\\\"x", 1),
    ] {
        let mut h = Harness::new(language, false);
        let id = h.source(source);
        let answer = finish(h.buffers.get_mut(id).unwrap(), Query::At(Scalar(point)));
        assert!(
            matches!(answer.outcome, Outcome::Uncertain(_)),
            "{language}: {source}: {answer:?}"
        );
    }
}

// Escaped quotes and skipped comment punctuation cannot terminate a typed
// structural match; malformed closers inside those contexts remain ignored.
#[test]
fn lexical_balance() {
    for (language, source) in [
        ("rust", r#"("\" ]" /* ] */ [])"#),
        ("c", "( /* ] */ '\\'' [x])"),
        ("go", "( `]` /* ) */ [x])"),
        ("toml", "( ']' # )\n[x])"),
        ("markdown", "( `]` [x])"),
    ] {
        let mut h = Harness::new(language, false);
        let id = h.source(source);
        let answer = finish(h.buffers.get_mut(id).unwrap(), Query::At(Scalar(0)));
        pair(
            &answer,
            (Kind::Parenthesis, 0, source.chars().count() - 1, 0),
        );
    }
}

// Byte boundaries count UTF-8 inspection, including oversized single lexical
// tokens. Larger requests report limited instead of a guessed context or mate.
#[test]
fn byte_bounds() {
    let mut buffers = Buffers::new().unwrap();
    for (name, source, limited) in [
        ("ASCII boundary", "x".repeat(MAX_BYTES), false),
        ("ASCII overflow", "x".repeat(MAX_BYTES + 1), true),
        ("UTF-8 boundary", "é".repeat(MAX_BYTES / 2), false),
        ("UTF-8 overflow", "é".repeat(MAX_BYTES / 2 + 1), true),
        (
            "oversized quoted token",
            format!("(\"{}\")", "x".repeat(MAX_BYTES)),
            true,
        ),
    ] {
        let id = buffers.create(&source).unwrap();
        let b = buffers.get_mut(id).unwrap();
        let query = Query::Context(Scalar(b.len_chars()));
        let answer = finish(b, query);
        if limited {
            assert_eq!(answer.outcome, Outcome::Limited(Limit::Bytes), "{name}");
        } else {
            assert_eq!(answer.outcome, Outcome::Context(Context::Code), "{name}");
        }
        assert_eq!(answer.stats.inspected_bytes, MAX_BYTES, "{name}");
        if limited {
            assert_eq!(answer.context, Context::Unknown, "{name}");
        }
    }
}

// Complete depth 256 is admitted; one more opener and actual metadata
// exhaustion refuse without returning the last apparently balanced fragment.
#[test]
fn retention_and_depth() {
    let mut buffers = Buffers::new().unwrap();
    for (depth, limited) in [(MAX_DEPTH, false), (MAX_DEPTH + 1, true)] {
        let source = format!("{}{}", "(".repeat(depth), ")".repeat(depth));
        let id = buffers.create(&source).unwrap();
        let answer = finish(buffers.get_mut(id).unwrap(), Query::At(Scalar(0)));
        if limited {
            assert_eq!(answer.outcome, Outcome::Limited(Limit::Depth));
        } else {
            pair(&answer, (Kind::Parenthesis, 0, depth * 2 - 1, 0));
        }
        assert_eq!(answer.stats.maximum_depth, MAX_DEPTH);
        eprintln!(
            "PAIR_BOUND depth={depth} inspected={} retained={} peak={} outcome={:?}",
            answer.stats.inspected_bytes,
            answer.stats.retained_bytes,
            answer.stats.maximum_depth,
            answer.outcome
        );
    }
    let source = "()".repeat(MAX_BYTES / 2);
    let id = buffers.create(&source).unwrap();
    let answer = finish(
        buffers.get_mut(id).unwrap(),
        Query::Context(Scalar(source.len())),
    );
    assert_eq!(answer.outcome, Outcome::Limited(Limit::Retention));
    assert!(answer.stats.retained_bytes > MAX_METADATA - 1024);
    assert!(answer.stats.inspected_bytes < MAX_BYTES);
    eprintln!(
        "PAIR_BOUND retention inspected={} retained={} outcome={:?}",
        answer.stats.inspected_bytes, answer.stats.retained_bytes, answer.outcome
    );
}

// Continuing the same admitted request reuses proof and the same cumulative
// inspection budget, including a later counted lookup that reaches the cap.
#[test]
fn counted_reuse() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers
        .create(&format!(
            "(){}[]{}",
            "x".repeat(130_000),
            "x".repeat(140_000)
        ))
        .unwrap();
    let b = buffers.get_mut(id).unwrap();
    let ticket = b.start_pair_request(Query::At(Scalar(0))).unwrap();
    let first = complete(b, ticket);
    pair(&first, (Kind::Parenthesis, 0, 1, 0));
    assert!(
        b.continue_pair_request(ticket, Query::At(Scalar(130_002)))
            .unwrap()
    );
    let next = complete(b, ticket);
    pair(&next, (Kind::Bracket, 130_002, 130_003, 0));
    assert!(next.stats.inspected_bytes > first.stats.inspected_bytes);
    assert!(
        b.continue_pair_request(ticket, Query::At(Scalar(0)))
            .unwrap()
    );
    let cached = complete(b, ticket);
    assert_eq!(cached.stats.inspected_bytes, next.stats.inspected_bytes);
    assert!(
        b.continue_pair_request(ticket, Query::Context(Scalar(b.len_chars())))
            .unwrap()
    );
    let limited = complete(b, ticket);
    assert_eq!(limited.outcome, Outcome::Limited(Limit::Bytes));
    assert_eq!(limited.stats.inspected_bytes, MAX_BYTES);
}

// Expired cooperative deadlines are inert. Cancellation, supersession and a
// native source publication retire both pending work and previously ready proof.
#[test]
fn retirement() {
    use rev::buffer::edit::Replacement;
    let mut buffers = Buffers::new().unwrap();
    let id = buffers
        .create(&format!("({})", "x".repeat(64_000)))
        .unwrap();
    let b = buffers.get_mut(id).unwrap();
    let ticket = b.start_pair_request(Query::At(Scalar(0))).unwrap();
    assert_eq!(b.poll_pair_request(ticket, Instant::now()), Status::Pending);
    assert_eq!(b.pair_stats(ticket).unwrap().inspected_bytes, 0);
    assert_eq!(
        b.poll_pair_request(ticket, Instant::now() + SLICE_TIME),
        Status::Pending
    );
    assert!(b.pair_stats(ticket).unwrap().inspected_bytes <= SLICE_TOKENS * 4);
    assert!(b.cancel_pair_request(ticket));
    assert_eq!(
        b.poll_pair_request(ticket, Instant::now() + SLICE_TIME),
        Status::Retired
    );
    assert!(b.pair_stats(ticket).is_none());
    let ticket = b.start_pair_request(Query::At(Scalar(0))).unwrap();
    complete(b, ticket);
    let edits = [Replacement {
        source: b.span(0..b.len_chars()).unwrap(),
        text: "[x]".into(),
    }];
    let prepared = b.prepare(&edits, &[], None).unwrap();
    b.publish(prepared, &mut []).unwrap();
    assert!(b.pair_answer(ticket).is_none());
    assert_eq!(
        b.poll_pair_request(ticket, Instant::now() + SLICE_TIME),
        Status::Retired
    );
    assert!(
        !b.continue_pair_request(ticket, Query::At(Scalar(0)))
            .unwrap()
    );
    pair(&finish(b, Query::At(Scalar(0))), (Kind::Bracket, 0, 2, 0));
}

// Current syntax strengthens literal exclusion, while source edits invalidate
// that evidence. Disabled colors continue to use the same recognized profile.
#[test]
fn syntax_freshness() {
    use rev::buffer::edit::Replacement;
    let mut h = Harness::new("rust", true);
    let id = h.source("fn main() { let x = \"()\"; }");
    let b = h.buffers.get_mut(id).unwrap();
    let answer = finish(b, Query::At(Scalar(7)));
    pair(&answer, (Kind::Parenthesis, 7, 8, 0));
    assert_eq!(answer.provenance, Provenance::CurrentSyntaxAndLexical);
    let literal = finish(b, Query::At(Scalar(21)));
    assert!(matches!(literal.outcome, Outcome::Uncertain(_)));
    assert_eq!(literal.provenance, Provenance::CurrentSyntax);
    let edits = [Replacement {
        source: b.span(0..b.len_chars()).unwrap(),
        text: "([])".into(),
    }];
    let prepared = b.prepare(&edits, &[], None).unwrap();
    b.publish(prepared, &mut []).unwrap();
    let fallback = finish(b, Query::At(Scalar(0)));
    pair(&fallback, (Kind::Parenthesis, 0, 3, 0));
    assert_eq!(fallback.provenance, Provenance::Lexical);
}

// Identity-mapped prefixes survive append-only native edits without retaining
// stale results. Editing an earlier scalar invalidates that proof completely.
#[test]
fn prefix_reuse() {
    use rev::buffer::edit::Replacement;
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(&"x".repeat(65_536)).unwrap();
    let b = buffers.get_mut(id).unwrap();
    let old = b
        .start_pair_request(Query::Context(Scalar(b.len_chars())))
        .unwrap();
    let first = complete(b, old);
    assert_eq!(first.stats.inspected_bytes, 65_536);
    for value in ["(", ")"] {
        let end = b.len_chars();
        let edits = [Replacement {
            source: b.span(end..end).unwrap(),
            text: value.into(),
        }];
        let prepared = b.prepare(&edits, &[], None).unwrap();
        b.publish(prepared, &mut []).unwrap();
        assert!(b.pair_answer(old).is_none());
        let next = finish(b, Query::Context(Scalar(b.len_chars())));
        assert_eq!(next.outcome, Outcome::Context(Context::Code));
        assert_eq!(next.provenance, Provenance::MappedLexical);
        assert_eq!(next.stats.reused_chars, end);
        assert_eq!(next.stats.inspected_bytes, 1);
    }
    let matched = finish(b, Query::At(Scalar(65_536)));
    pair(&matched, (Kind::Parenthesis, 65_536, 65_537, 0));
    assert_eq!(matched.stats.inspected_bytes, 0);
    let edits = [Replacement {
        source: b.span(0..0).unwrap(),
        text: "\"".into(),
    }];
    let prepared = b.prepare(&edits, &[], None).unwrap();
    b.publish(prepared, &mut []).unwrap();
    let changed = finish(b, Query::At(Scalar(65_537)));
    assert!(matches!(changed.outcome, Outcome::Uncertain(_)));
    assert_eq!(changed.stats.reused_chars, 0);
    assert!(changed.stats.inspected_bytes >= 65_537);
}

// Cached searches obey cooperative work limits too. A cached outer pair cannot
// authorize a delimiter inside a non-admitted string or literal/comment region.
#[test]
fn cached_contexts() {
    let mut buffers = Buffers::new().unwrap();
    let source = "(\"[]\" x)";
    let id = buffers.create(source).unwrap();
    let b = buffers.get_mut(id).unwrap();
    let ticket = b.start_pair_request(Query::At(Scalar(0))).unwrap();
    complete(b, ticket);
    for point in [2, 3, 6] {
        assert!(
            b.continue_pair_request(ticket, Query::At(Scalar(point)))
                .unwrap()
        );
        let result = complete(b, ticket);
        assert!(
            matches!(result.outcome, Outcome::Uncertain(_)),
            "{point}: {result:?}"
        );
    }
    assert!(
        b.continue_pair_request(ticket, Query::Enclosing(Scalar(2)))
            .unwrap()
    );
    pair(&complete(b, ticket), (Kind::DoubleQuote, 1, 4, 1));
}

// Current native trees respect C characters and Go raw strings, while an
// unavailable external grammar still uses declared identity, not colors.
#[test]
fn native_quotes() {
    for (language, source, point, close, kind) in [
        ("c", "char x = 'a';", 9, 11, Kind::SingleQuote),
        ("go", "package p\nvar x = `a`", 18, 20, Kind::Backtick),
    ] {
        let mut h = Harness::new(language, true);
        let id = h.source(source);
        pair(
            &finish(h.buffers.get_mut(id).unwrap(), Query::At(Scalar(point))),
            (kind, point, close, 0),
        );
    }
}

// Native syntax observations cannot descend an unbounded AST path. Exhaustion
// returns limited before source inspection, rather than publishing partial proof.
#[test]
fn native_depth() {
    let mut h = Harness::new("rust", true);
    let prefix = "fn main() { let x = ";
    let source = format!(
        "{prefix}{}1{}; }}",
        "(".repeat(MAX_DEPTH + 4),
        ")".repeat(MAX_DEPTH + 4)
    );
    let id = h.source(&source);
    let b = h.buffers.get_mut(id).unwrap();
    let answer = finish(b, Query::At(Scalar(prefix.len() + MAX_DEPTH + 3)));
    assert_eq!(answer.outcome, Outcome::Limited(Limit::Depth));
    assert_eq!(answer.stats.syntax_nodes, MAX_DEPTH);
    assert_eq!(answer.stats.inspected_bytes, 0);
    assert_eq!(answer.context, Context::Unknown);
}

// A native quote observation and fallback scanning share the same poll's
// scalar admission. The native point cannot add a hidden 257th source read.
#[test]
fn native_observation_budget() {
    let mut h = Harness::new("rust", true);
    let source = format!("fn main() {{ let x = \"{}\"; }}", "x".repeat(1024));
    let point = source.rfind('"').unwrap();
    let id = h.source(&source);
    let b = h.buffers.get_mut(id).unwrap();
    let ticket = b.start_pair_request(Query::At(Scalar(point))).unwrap();
    assert_eq!(
        b.poll_pair_request(ticket, Instant::now() + SLICE_TIME),
        Status::Pending
    );
    let stats = b.pair_stats(ticket).unwrap();
    assert!(stats.syntax_nodes > 0);
    assert!(stats.inspected_bytes <= SLICE_TOKENS);
    assert!(b.pair_answer(ticket).is_none());
    assert!(b.cancel_pair_request(ticket));
}

// Language/mode application and another view's request retire ticket ownership,
// while observation never changes canonical bytes, marks or source history.
#[test]
fn shared_lifecycle() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("([x])").unwrap();
    let b = buffers.get_mut(id).unwrap();
    b.set_mark(Some(Scalar(2))).unwrap();
    let stamp = b.stamp();
    let view = rev::view::View::new(b);
    let mut windows = rev::window::Windows::new(view).unwrap();
    windows
        .split(
            rev::window::Split::Right,
            rev::view::Size {
                width: 80,
                height: 24,
            },
        )
        .unwrap();
    let prior = b.start_pair_request(Query::At(Scalar(0))).unwrap();
    for (_, view) in windows.iter() {
        assert_eq!(view.point.stamp.buffer, id);
        pair(&finish(b, Query::At(Scalar(1))), (Kind::Bracket, 1, 3, 1));
    }
    assert_eq!(
        b.poll_pair_request(prior, Instant::now() + SLICE_TIME),
        Status::Retired
    );
    let prior = b.start_pair_request(Query::At(Scalar(0))).unwrap();
    b.select_mode(rev::buffer::Mode::Org).unwrap();
    assert!(b.pair_answer(prior).is_none());
    let prior = b.start_pair_request(Query::At(Scalar(0))).unwrap();
    b.apply_settings(&rev::config::Snapshot::default());
    assert_eq!(
        b.poll_pair_request(prior, Instant::now() + SLICE_TIME),
        Status::Retired
    );
    assert_eq!(b.stamp(), stamp);
    assert_eq!(b.mark(), Some(Scalar(2)));
    assert!(b.history().is_empty());
    assert!(!b.is_modified());
}

// Deletions restore a recent bounded lexical checkpoint instead of rescanning
// the entire unchanged prefix. EOF and lookahead are part of the proof horizon.
#[test]
fn prefix_horizons() {
    use rev::buffer::edit::Replacement;
    let mut h = Harness::new("c", false);
    let id = h.source(&format!("{}/", "x".repeat(65_536)));
    let b = h.buffers.get_mut(id).unwrap();
    finish(b, Query::Context(Scalar(b.len_chars())));
    let end = b.len_chars();
    let edits = [Replacement {
        source: b.span(end..end).unwrap(),
        text: "/()".into(),
    }];
    let prepared = b.prepare(&edits, &[], None).unwrap();
    b.publish(prepared, &mut []).unwrap();
    let answer = finish(b, Query::At(Scalar(end + 1)));
    assert!(matches!(answer.outcome, Outcome::Uncertain(_)));
    assert_eq!(answer.context, Context::Comment);
    assert!(answer.stats.reused_chars >= 65_280);
    assert!(answer.stats.inspected_bytes < 300);

    let id = h.source(&"x".repeat(65_537));
    let b = h.buffers.get_mut(id).unwrap();
    finish(b, Query::Context(Scalar(b.len_chars())));
    for _ in 0..3 {
        let end = b.len_chars();
        let edits = [Replacement {
            source: b.span(end - 1..end).unwrap(),
            text: String::new(),
        }];
        let prepared = b.prepare(&edits, &[], None).unwrap();
        b.publish(prepared, &mut []).unwrap();
        let answer = finish(b, Query::Context(Scalar(b.len_chars())));
        assert_eq!(answer.outcome, Outcome::Context(Context::Code));
        assert!(answer.stats.reused_chars >= 65_000);
        assert!(answer.stats.inspected_bytes <= 256);
    }
}

// Ambiguous multiline and embedded constructs never create a typed pair. An
// incomplete escape also cannot become an ordinary insertion context at EOF.
#[test]
fn conservative_boundaries() {
    for (language, source, point) in [
        ("plain", "\"\"\"x\" ()\"\"\"", 7),
        ("c", "\"x\n()\"", 3),
        ("go", "\"x\n()\"", 3),
        ("json", "\"x\n()\"", 3),
        ("toml", "'x\n()'", 3),
        ("tsx", "<div>text ()</div>", 10),
        ("markdown", "    ()", 4),
        ("markdown", "  \t()", 3),
        ("org", "#+begin_src rust\n#+end_example\n()\n#+end_src", 31),
    ] {
        let mut h = Harness::new(language, false);
        let id = h.source(source);
        let answer = finish(h.buffers.get_mut(id).unwrap(), Query::At(Scalar(point)));
        assert!(
            matches!(answer.outcome, Outcome::Uncertain(_)),
            "{language}: {source}: {answer:?}"
        );
    }
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("\\").unwrap();
    let answer = finish(buffers.get_mut(id).unwrap(), Query::Context(Scalar(1)));
    assert_eq!(answer.outcome, Outcome::Context(Context::Literal));
}

// Cold requests measure the real native owner at the planning sizes. Latencies
// are evidence, not universal deadlines; bytes, depth and retention are asserted.
#[test]
fn resource_measurements() {
    for bytes in [1024, 65_536, 262_144, 1_048_576] {
        for (name, source) in [
            ("complete", format!("({})", "x".repeat(bytes - 2))),
            ("malformed", format!("({}]", "x".repeat(bytes - 2))),
            (
                "oversized-token",
                format!("(\"{}\")", "x".repeat(bytes - 4)),
            ),
        ] {
            measured_row(name, &source);
        }
    }
}

fn measured_row(name: &str, source: &str) {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
    let b = buffers.get_mut(id).unwrap();
    let mut durations = Vec::new();
    let mut maximum = Duration::ZERO;
    let mut previous = None;
    let mut last = None;
    for _ in 0..11 {
        if let Some(ticket) = previous {
            assert!(b.cancel_pair_request(ticket));
        }
        let before = Instant::now();
        let ticket = b.start_pair_request(Query::At(Scalar(0))).unwrap();
        let (answer, slice) = timed_complete(b, ticket);
        durations.push(before.elapsed().as_micros());
        maximum = maximum.max(slice);
        previous = Some(ticket);
        last = Some(answer);
    }
    durations.sort_unstable();
    let answer = last.unwrap();
    assert_measured(name, source.len(), &answer);
    eprintln!(
        "PAIR_MEASURE {name} source={} median_us={} p95_us={} max_poll_us={} inspected={} retained={} depth={} max_slice_tokens={} slices={} outcome={:?}",
        source.len(),
        durations[5],
        durations[10],
        maximum.as_micros(),
        answer.stats.inspected_bytes,
        answer.stats.retained_bytes,
        answer.stats.maximum_depth,
        answer.stats.maximum_slice_tokens,
        answer.stats.slices,
        answer.outcome
    );
}

fn assert_measured(name: &str, bytes: usize, answer: &Answer) {
    if bytes > MAX_BYTES {
        assert_eq!(answer.outcome, Outcome::Limited(Limit::Bytes));
    } else if name == "malformed" {
        assert!(matches!(answer.outcome, Outcome::Mismatched { .. }));
    } else {
        assert!(matches!(answer.outcome, Outcome::Confirmed(_)));
    }
    assert!(answer.stats.maximum_depth <= MAX_DEPTH);
}
