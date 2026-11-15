use crate::qualification::{Harness, finish};
use rev::coordinates::Scalar;
use rev::pairs::{Kind, Outcome, Query, Wrapping};

// Region proof requires both boundaries in one ordinary context and strictly
// balanced selected source. Comments/strings cannot donate a partial boundary.
#[test]
fn boundaries() {
    for (language, source, start, end, admitted) in [
        ("plain", "alpha", 0, 5, true),
        ("plain", "[alpha]", 0, 7, true),
        ("plain", "(alpha)", 1, 6, true),
        ("plain", "(alpha)", 1, 7, false),
        ("plain", "([)]", 0, 4, false),
        ("rust", "// alpha", 3, 8, false),
        ("rust", "\"alpha\"", 1, 6, false),
        ("html", "<a x=alpha>", 5, 10, false),
        ("markdown", "`alpha`", 1, 6, false),
    ] {
        let mut h = Harness::new(language, false);
        let id = h.source(source);
        let answer = finish(
            h.buffers.get_mut(id).unwrap(),
            Query::Wrap(Wrapping {
                start: Scalar(start),
                end: Scalar(end),
                kind: Kind::Parenthesis,
                count: 1,
            }),
        );
        assert_eq!(
            matches!(answer.outcome, Outcome::Wrap(_)),
            admitted,
            "{language} {source}: {answer:?}"
        );
    }
}

// Cached pair facts retain quote literal-safety evidence, and a virtual quoted
// wrapping request never becomes proof that the original selection is a string.
#[test]
fn retained_proof() {
    let mut h = Harness::new("plain", false);
    let id = h.source("\"a\\nb\"");
    let b = h.buffers.get_mut(id).unwrap();
    assert!(matches!(
        finish(b, Query::At(Scalar(0))).outcome,
        Outcome::Confirmed(_)
    ));
    assert!(matches!(
        finish(b, Query::Unwrap(Scalar(2))).outcome,
        Outcome::Uncertain(_)
    ));
    let id = h.source("alpha");
    let b = h.buffers.get_mut(id).unwrap();
    assert_eq!(
        finish(
            b,
            Query::Wrap(Wrapping {
                start: Scalar(0),
                end: Scalar(5),
                kind: Kind::DoubleQuote,
                count: 1
            })
        )
        .outcome,
        Outcome::Wrap(Kind::DoubleQuote)
    );
    assert_eq!(
        finish(b, Query::Context(Scalar(2))).outcome,
        Outcome::Context(rev::pairs::Context::Code)
    );
}
