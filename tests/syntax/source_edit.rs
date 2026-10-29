use super::*;
use rev::{
    command::{Command, OrgCommand as C},
    session::Session,
};
use std::time::{Duration, Instant};
fn settle(session: &mut Session) {
    let until = Instant::now() + Duration::from_secs(5);
    loop {
        session.frame().unwrap();
        if session.remaining_input(Instant::now()).is_none() {
            return;
        }
        assert!(Instant::now() < until, "source edit job did not finish");
        session.expire_input(Instant::now() + Duration::from_millis(10));
        std::thread::yield_now();
    }
}
// Source drafts resolve grammar and editing policy from their declared language
// without a fake filename. Global parser limits and literal Org ownership hold.
#[test]
fn declared_presentation() {
    let cases = [
        ("bundled", "rust", "", "Org Src Rust", 4),
        ("disabled", "rust", "syntax.enabled=false", "Org Src", 4),
        ("cutoff", "rust", "syntax.max_bytes=4", "Org Src", 4),
        (
            "plain override",
            "rust",
            "[language.rust]\ngrammar='plain'",
            "Org Src",
            4,
        ),
        ("unknown", "missing", "", "Org Src", 4),
        (
            "declared grammar",
            "custom",
            "[language.custom]\ngrammar='rust'\ntab_width=7\nindent_width=3",
            "Org Src Rust",
            7,
        ),
        (
            "literal org",
            "org",
            "[language.org]\ngrammar='rust'",
            "Org Src Rust",
            4,
        ),
        ("no language", "", "", "Org Src", 4),
    ];
    for (name, language, config, mode, width) in cases {
        let f = Fixture::new();
        let startup = rev::config::prepare(&f.file("config.toml", config));
        assert!(startup.diagnostic.is_none(), "{name}");
        let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
        let body = "fn main() {}\n,* literal\n,#+END_SRC\n";
        let source = format!("#+BEGIN_SRC {language}\n{body}#+END_SRC\n");
        let id = buffers.visit(&f.file("notes.org", &source)).unwrap();
        let view = View::new(buffers.get(id).unwrap());
        let mut s = Session::new(
            buffers,
            view,
            Size {
                width: 80,
                height: 10,
            },
        )
        .unwrap();
        s.execute(Command::Org(C::EditSpecial));
        settle(&mut s);
        let draft = s
            .source_edit_draft()
            .unwrap_or_else(|| panic!("{name}: {}", s.echo()));
        assert_eq!(draft.mode(), mode, "{name}");
        assert_eq!(draft.policy().tab_width, width, "{name}");
        assert!(!draft.is_org(), "{name}");
        assert!(draft.org_index().is_none(), "{name}");
        assert!(draft.file().is_none(), "{name}");
        assert_eq!(
            text(draft),
            "fn main() {}\n* literal\n#+END_SRC\n",
            "{name}"
        );
        assert_eq!(
            !draft
                .syntax()
                .captures(draft, 0..draft.len_bytes())
                .is_empty(),
            mode == "Org Src Rust",
            "{name}"
        );
        assert!(s.load(&f.path("other.rs")).is_err(), "{name}");
        s.execute(Command::Org(C::EditSrcAbort));
        settle(&mut s);
        assert_eq!(text(s.buffer().unwrap()), source, "{name}");
    }
}
