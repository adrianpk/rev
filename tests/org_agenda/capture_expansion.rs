use super::*;
use rev::{
    config::CaptureTemplate,
    coordinates::Byte,
    org::{
        Cancellation,
        capture::*,
        time::{CivilDateTime, CivilTime, Date},
    },
};
fn context() -> Context {
    Context::new(CivilDateTime {
        date: Date::new(2024, 2, 29).unwrap(),
        time: CivilTime::new(9, 7, 6).unwrap(),
    })
}
fn expansion(text: &str, values: Context) -> Result<Expansion, String> {
    let mut template = CaptureTemplate::task();
    template.template = text.into();
    Expansion::new(&template, values)
}
fn ready(work: &mut Expansion) -> DraftText {
    let cancel = Cancellation::default();
    loop {
        match work.advance(&cancel).unwrap() {
            Step::Progress => (),
            Step::Ready(text) => return text,
            Step::Prompt(p) => panic!("unexpected prompt {}", p.label),
        }
    }
}
// The declared data substitutions share one civil snapshot. Opaque inserted
// values preserve Unicode, CRLF and escape-looking text without a second scan.
#[test]
fn data_substitutions() {
    let mut values = context();
    values.initial = "界\r\n%(opaque)\n".into();
    values.annotation = "annotation".into();
    values.link = "[[file:/notes.org::2]]".into();
    values.kill = "%?".into();
    values.user = Some("Ada".into());
    values.clipboard = Some("%[unread]".into());
    let mut work = expansion("  %i%a %l %c %n %x\n%t %T %u %U\n%<%F %a %A %b %B %H:%M:%S %j %u %w %%>\n\\%literal %% %?e\u{301}%?", values).unwrap();
    let result = ready(&mut work);
    assert_eq!(
        result.text,
        "  界\r\n  %(opaque)\nannotation [[file:/notes.org::2]] %? Ada %[unread]\n<2024-02-29 Thu> <2024-02-29 Thu 09:07> [2024-02-29 Thu] [2024-02-29 Thu 09:07]\n2024-02-29 Thu Thursday Feb February 09:07:06 060 4 4 %\n%literal % e\u{301}"
    );
    assert_eq!(&result.text[result.cursor.0..], "e\u{301}");
    assert!(work.advance(&Cancellation::default()).is_err());
}
// Prompts expose bounded defaults/completions; answers and unavailable host
// values are explicit text, and asking repeatedly does not consume prompt slots.
#[test]
fn prompt_data() {
    let mut work = expansion("%^{Topic|default|one|two} %n %n %x%?", context()).unwrap();
    for (label, default, completions, answer) in [
        ("Topic: ", "default", vec!["one", "two"], "%(literal)"),
        ("Capture user name: ", "", vec![], "Ada"),
        ("Clipboard text (explicit input): ", "", vec![], "界"),
    ] {
        for _ in 0..70 {
            let Step::Prompt(prompt) = work.advance(&Cancellation::default()).unwrap() else {
                panic!("expected prompt")
            };
            assert_eq!(prompt.label, label);
            assert_eq!(prompt.default, default);
            assert_eq!(prompt.completions, completions);
        }
        work.answer(answer).unwrap();
    }
    let result = ready(&mut work);
    assert_eq!(result.text, "%(literal) Ada Ada 界");
    assert_eq!(result.cursor, Byte(result.text.len()));
    assert!(work.answer("late").is_err());
}
// Admission rejects malformed/unsupported instructions before obtaining any
// answers. The original template remains literal and never triggers evaluation.
#[test]
fn malformed_templates() {
    for input in [
        "%",
        "%q",
        "%^(label)",
        "%^{",
        "%^{}",
        "%<%Q>",
        "%<%>",
        "%<",
        "%(shell-command)",
        "%[file]",
        "%^{first}%<%z>",
    ] {
        assert!(expansion(input, context()).is_err(), "{input}");
    }
    assert_eq!(
        ready(&mut expansion("\\%(literal) \\%[literal]", context()).unwrap()).text,
        "%(literal) %[literal]"
    );
}
// Expansion has independent input, token, prompt and output bounds. Cancel is
// effective while waiting for a human as well as between finite work quanta.
#[test]
fn expansion_bounds() {
    let mut values = context();
    values.kill = "x".repeat(VALUE_BYTES);
    assert_eq!(
        ready(&mut expansion("%c%c%c%c", values.clone()).unwrap())
            .text
            .len(),
        EXPANSION_BYTES
    );
    let mut over = expansion("%c%c%c%c!", values.clone()).unwrap();
    assert!(over.advance(&Cancellation::default()).is_err());
    values.kill.push('x');
    assert!(expansion("%c", values).is_err());
    assert!(expansion(&"%?".repeat(4097), context()).is_err());
    let mut work = expansion(&"%?".repeat(4096), context()).unwrap();
    assert!(matches!(
        work.advance(&Cancellation::default()).unwrap(),
        Step::Progress
    ));
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(work.advance(&cancel).is_err());
    let mut work = expansion(&"%^{x}".repeat(65), context()).unwrap();
    for _ in 0..64 {
        assert!(matches!(
            work.advance(&Cancellation::default()).unwrap(),
            Step::Prompt(_)
        ));
        work.answer("").unwrap();
    }
    assert!(work.advance(&Cancellation::default()).is_err());
    let mut work = expansion("%^{x}", context()).unwrap();
    assert!(matches!(
        work.advance(&Cancellation::default()).unwrap(),
        Step::Prompt(_)
    ));
    assert!(work.advance(&cancel).is_err());
}
// Origin links admit only actual checked source points and encode syntax bytes;
// an unnamed origin yields no fabricated file or source identity.
#[test]
fn checked_origin() {
    let (fixture, session) = editor("", "a] 界.org", "one\ntwo\n");
    let buffer = session.buffer().unwrap();
    let point = rev::buffer::edit::Point {
        stamp: buffer.stamp(),
        position: rev::coordinates::Scalar(4),
    };
    let link = origin_link(buffer, point).unwrap().unwrap();
    assert!(link.ends_with("a%5D%20%E7%95%8C.org::2]]"));
    assert!(link.contains(fixture.0.to_str().unwrap()));
    let other = Buffers::new().unwrap();
    let unnamed = other.get(other.initial()).unwrap();
    assert!(origin_link(unnamed, point).is_err());
    assert_eq!(
        origin_link(
            unnamed,
            rev::buffer::edit::Point {
                stamp: unnamed.stamp(),
                position: rev::coordinates::Scalar(0)
            }
        )
        .unwrap(),
        None
    );
}
