use rev::org::{MatchData, MatchQuery};
use std::collections::BTreeMap;

fn data() -> MatchData {
    MatchData {
        tags: vec!["work".into(), "urgent".into(), "Work".into()],
        properties: BTreeMap::from([
            ("EFFORT".into(), "2.5".into()),
            ("TITLE".into(), "A \"quote\" and \\ path".into()),
            ("OWNER-ROLE".into(), "lead".into()),
            ("TODO".into(), "WAITING".into()),
        ]),
        finished: false,
    }
}

// The actual public parser/evaluator boundary protects precedence, implicit
// conjunction, negation, typed comparisons, escaping and TODO suffix semantics.
#[test]
fn query_matching() {
    let values = data();
    for (name, source, expected) in [
        ("tag", "work", true),
        ("case", "WORK", false),
        ("implicit and", "+work+urgent-boss", true),
        ("precedence", "absent|work&urgent", true),
        ("parentheses", "(absent|work)&-urgent", false),
        ("nested not", "!(absent|!work)", true),
        ("number", "EFFORT>=2.5+EFFORT<3", true),
        ("negative exponent", "EFFORT> -2e+1", true),
        ("number inequality", "EFFORT/=2.5", false),
        ("absent numeric", "MISSING<1", true),
        ("present numeric", "MISSING<*1", false),
        ("string inequality", "OWNER\\-ROLE<>\"other\"", true),
        (
            "quoted escape",
            "TITLE=\"A \\\"quote\\\" and \\\\ path\"",
            true,
        ),
        ("property case", "owner\\-role==\"lead\"", true),
        ("todo", "work/WAITING|NEXT", true),
        ("unfinished", "/!", true),
        ("todo exclusion", "work/!-WAITING", false),
    ] {
        let query = MatchQuery::parse(source).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(query.matches(&values).unwrap(), expected, "{name}");
    }
}

// Malformed or executable-looking syntax is rejected at compilation, rather
// than silently producing a permissive query. Length/depth/node caps are finite.
#[test]
fn query_rejection() {
    for (name, source) in [
        ("empty", ""),
        ("trailing", "work|"),
        ("adjacent", "work home"),
        ("parenthesis", "(work"),
        ("extra close", "work)"),
        ("number", "EFFORT=NaN"),
        ("infinite", "EFFORT=1e309"),
        ("missing value", "OWNER="),
        ("bad escape", "OWNER=\"\\q\""),
        ("open quote", "OWNER=\"lead"),
        ("name escape", "OWNER\\"),
        ("predicate", "(lambda () t)"),
        ("regexp", "OWNER={.*}"),
        ("todo comparison", "work/TODO=\"DONE\""),
    ] {
        assert!(MatchQuery::parse(source).is_err(), "{name}: {source}");
    }
    for source in [
        "x".repeat(4097),
        format!("{}work{}", "(".repeat(33), ")".repeat(33)),
        vec!["x"; 258].join("|"),
    ] {
        assert!(MatchQuery::parse(&source).is_err());
    }
}

// Typed metadata never turns NaN or a non-number into an ordered comparison;
// a completed TODO is excluded by /! and oversized projections reject finitely.
#[test]
fn query_value_types() {
    let mut values = data();
    for actual in ["NaN", "inf", "invalid"] {
        values.properties.insert("EFFORT".into(), actual.into());
        assert!(
            !MatchQuery::parse("EFFORT<>3")
                .unwrap()
                .matches(&values)
                .unwrap()
        );
    }
    values.finished = true;
    assert!(!MatchQuery::parse("/!").unwrap().matches(&values).unwrap());
    values
        .properties
        .insert("LARGE".into(), "x".repeat(1048577));
    assert!(MatchQuery::parse("work").unwrap().matches(&values).is_err());
}

// Admission accepts the exact byte/depth/node/projection boundaries as well as
// rejecting their successors. This detects off-by-one cuts in the public grammar.
#[test]
fn query_boundaries() {
    let values = data();
    let depth = format!("{}work{}", "(".repeat(32), ")".repeat(32));
    assert!(MatchQuery::parse(&depth).unwrap().matches(&values).unwrap());
    let nodes = format!("!{}", vec!["absent"; 256].join("|"));
    assert!(MatchQuery::parse(&nodes).unwrap().matches(&values).unwrap());
    let literal = "x".repeat(4096 - 4);
    let bytes = format!("X=\"{literal}\"");
    let mut data = MatchData::default();
    data.properties.insert("X".into(), literal);
    assert!(MatchQuery::parse(&bytes).unwrap().matches(&data).unwrap());
    data.properties.insert("X".into(), "x".repeat(1048575));
    assert!(!MatchQuery::parse("absent").unwrap().matches(&data).unwrap());
    data.properties.get_mut("X").unwrap().push('x');
    assert!(MatchQuery::parse("absent").unwrap().matches(&data).is_err());
}

// TODO keywords have their own spelling rules. Escaped punctuation in a TODO
// suffix must not be rejected by the more restrictive heading-tag grammar.
#[test]
fn query_todo_names() {
    for (state, escaped) in [
        ("NEXT-ACTION", "NEXT\\-ACTION"),
        ("REVIEW.NOW", "REVIEW\\.NOW"),
    ] {
        let mut values = data();
        values.properties.insert("TODO".into(), state.into());
        let query = MatchQuery::parse(&format!("work/{escaped}")).unwrap();
        assert!(query.matches(&values).unwrap());
    }
}
