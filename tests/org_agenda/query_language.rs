use rev::org::{Cancellation, MatchData, MatchQuery, Regexp, SearchQuery};
use ropey::Rope;
// Search phrases normalize whitespace; Boolean words, negation and finite
// regexp terms select whole entry text without backtracking or code execution.
#[test]
fn query_search_language() {
    for (query, text, expected) in [
        ("computer equipment", "Computer\n  equipment", true),
        ("computer equipment", "computer and equipment", false),
        (
            "+computer wifi -ethernet -{8\\.11[bg]}",
            "Computer wifi",
            true,
        ),
        (
            "+computer wifi -ethernet -{8\\.11[bg]}",
            "computer wifi 8.11g",
            false,
        ),
        ("+word", "sword", false),
        ("(notes)", "notes", false),
        ("(notes)", "(notes)", true),
        ("+\"two words\" -bad", "Two\nwords", true),
        ("+{^(work|home)[0-9]+$}", "work21", true),
        ("+{^(work|home)[0-9]+$}", "work", false),
    ] {
        let plan = SearchQuery::parse(query, 4096, 32).unwrap();
        assert_eq!(
            plan.matches(Rope::from_str(text).slice(..), &Cancellation::default())
                .unwrap(),
            expected,
            "{query}: {text}"
        );
    }
    for query in [
        "+{[}",
        "+{a**}",
        "+{(a}",
        "+{a{2}}",
        "+{\\1}",
        "+\"unclosed",
        "+",
        "",
    ] {
        assert!(SearchQuery::parse(query, 4096, 32).is_err(), "{query}");
    }
}
// Both query parsers enforce their admitted limits; malformed syntax cannot
// become a complete empty result or execute a source predicate.
#[test]
fn query_match_bounds() {
    for query in [
        "work|home+LEVEL>=2",
        "work/!TODO",
        r#"NAME="two\"quotes""#,
        r#"boss\-prio="C""#,
    ] {
        assert!(MatchQuery::bounded(query, 4096, 32).is_ok(), "{query}");
    }
    for query in [
        "work&&home",
        "LEVEL=>2",
        "(work",
        r#"NAME="bad\q""#,
        "(lambda ())",
    ] {
        assert!(MatchQuery::bounded(query, 4096, 32).is_err(), "{query}");
    }
    assert!(MatchQuery::bounded("work", 3, 32).is_err());
    assert!(MatchQuery::bounded("(((work)))", 4096, 2).is_err());
    assert!(Regexp::parse("((x))", 1).is_err());
    let query = MatchQuery::parse("work+EFFORT<*120").unwrap();
    let data = MatchData {
        tags: vec!["work".into()],
        properties: [("EFFORT".into(), "1:30".into())].into_iter().collect(),
        finished: false,
    };
    assert!(query.matches(&data).unwrap());
}
