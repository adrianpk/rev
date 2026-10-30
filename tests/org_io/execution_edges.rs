use super::execution::configured;
use super::*;
use rev::org::execution::State;
fn host_with(body: &str, fields: &str, global: &str) -> (Fixture, Session) {
    let source = format!("#+BEGIN_SRC sh\n{body}\n#+END_SRC\n#+RESULTS:\n: previous\n");
    let config = format!(
        "org.time.zone='UTC'\n{global}\n[[org.source.programs]]\nname='shell'\nlanguage='sh'\ncwd='source'\n{fields}"
    );
    configured(&source, &config)
}
// Stdin, literal file argv and a real compile/run chain preserve argv boundaries.
// Source input is code only for the explicitly chosen, confirmed shell program.
#[test]
fn execute_delivery() {
    let cases = [
        (
            "stdin",
            "executable='/bin/sh'",
            "printf stdin",
            b"stdin".as_slice(),
        ),
        (
            "file",
            "executable='/bin/sh'\ndelivery='file'\nargv=['{input}']",
            "printf file",
            b"file".as_slice(),
        ),
        (
            "compiled",
            "executable='{output}'\ndelivery='file'\nargv=['literal ; not shell']\nsteps=[{executable='/bin/cp',argv=['{input}','{output}']},{executable='/bin/chmod',argv=['700','{output}']}]",
            "#!/bin/sh\nprintf '%s' \"$1\"",
            b"literal ; not shell".as_slice(),
        ),
    ];
    for (name, profile, body, wanted) in cases {
        let (_f, mut s) = host_with(body, profile, "");
        let before = text(&s);
        action(&mut s, C::ExecuteSrc);
        argument(&mut s, "yes");
        let out = s.execution_result().unwrap();
        assert_eq!(out.state, State::Succeeded, "{name}: {}", out.detail);
        assert_eq!(out.stdout, wanted, "{name}");
        assert_eq!(
            text(&s),
            before.replace(
                ": previous",
                &format!(": {}", std::str::from_utf8(wanted).unwrap())
            )
        );
        assert_eq!(s.buffer().unwrap().history().len(), 1);
    }
}
// Missing tools, nonzero exits and combined output ceilings remain failures;
// exact byte limits pass, and neither truncation nor prior steps imply success.
#[test]
fn execute_failures() {
    for (name, body, profile, global, state, detail) in [
        (
            "missing",
            "printf bad",
            "executable='/missing-source-runner'",
            "",
            State::Failed,
            "Start configured program",
        ),
        (
            "nonzero",
            "printf kept; printf error >&2; exit 7",
            "executable='/bin/sh'",
            "",
            State::Failed,
            "failed",
        ),
        (
            "ceiling",
            "printf abc; printf def >&2",
            "executable='/bin/sh'\noutput_bytes=5",
            "",
            State::Failed,
            "output limit",
        ),
        (
            "exact",
            "printf abc; printf def >&2",
            "executable='/bin/sh'\noutput_bytes=6",
            "",
            State::Succeeded,
            "",
        ),
        (
            "global ceiling",
            "printf abc; printf def >&2",
            "executable='/bin/sh'\noutput_bytes=100",
            "org.limits.job_output_bytes=5",
            State::Failed,
            "output limit",
        ),
        (
            "expansion",
            "printf should-not-run > marker",
            "executable='/bin/sh'",
            "org.limits.expansion_bytes=4",
            State::Failed,
            "expansion byte limit",
        ),
    ] {
        let (f, mut s) = host_with(body, profile, global);
        let before = text(&s);
        action(&mut s, C::ExecuteSrc);
        argument(&mut s, "yes");
        let out = s.execution_result().unwrap();
        assert_eq!(out.state, state, "{name}: {}", out.detail);
        assert!(out.detail.contains(detail), "{name}: {}", out.detail);
        if name == "nonzero" {
            assert_eq!(out.status, Some(7));
            assert_eq!(out.stdout, b"kept");
            assert_eq!(out.stderr, b"error");
        }
        if name == "exact" {
            assert_eq!(out.stdout, b"abc");
            assert_eq!(out.stderr, b"def");
        }
        assert!(!f.0.join("marker").exists(), "{name}");
        assert_eq!(
            text(&s),
            if name == "exact" {
                before.replace(": previous", ": abc")
            } else {
                before
            },
            "{name}"
        );
    }
}
const LIVE: &str =
    "echo $$ > leader; sleep 30 & echo $! > descendant; printf '%s' \"$HOME\" > home; wait";
fn started(s: &mut Session, f: &Fixture) {
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        s.frame().unwrap();
        if f.0.join("home").exists()
            && std::fs::read_to_string(f.0.join("home")).is_ok_and(|s| !s.is_empty())
        {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "owned child did not start: {}",
            s.echo()
        );
        s.expire_input(Instant::now() + Duration::from_millis(10));
        std::thread::yield_now();
    }
}
fn stopped(pid: &str) -> bool {
    std::fs::read_to_string(format!("/proc/{}/stat", pid.trim()))
        .map_or(true, |s| s.split_once(") ").unwrap().1.starts_with('Z'))
}
// Cancel, timeout and session drop stop only the owned process group. The direct
// child is reaped, descendants stop, private directories disappear and idle ends.
#[test]
fn execute_lifetime() {
    for mode in ["cancel", "timeout", "profile timeout", "drop"] {
        let (f, mut s) = host_with(LIVE, "executable='/bin/sh'", "org.limits.job_seconds=1");
        let before = text(&s);
        action(&mut s, C::ExecuteSrc);
        s.handle(Event::Paste("yes".into()));
        s.handle(Event::Key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE,
        )));
        started(&mut s, &f);
        let leader = std::fs::read_to_string(f.0.join("leader")).unwrap();
        let descendant = std::fs::read_to_string(f.0.join("descendant")).unwrap();
        let home = std::fs::read_to_string(f.0.join("home")).unwrap();
        use std::os::unix::fs::PermissionsExt;
        let root = Path::new(&home).parent().unwrap();
        assert_eq!(
            std::fs::metadata(root).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            std::fs::metadata(root.join("source.code"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        if mode == "drop" {
            drop(s);
        } else {
            if mode == "cancel" {
                s.handle(Event::Paste("blocked".into()));
                s.execute(Command::DeleteChar);
                cancel(&mut s);
            }
            settle(&mut s);
            let out = s.execution_result().unwrap();
            assert_eq!(
                out.state,
                if mode == "cancel" {
                    State::Cancelled
                } else {
                    State::TimedOut
                },
                "{}",
                out.detail
            );
            assert_eq!(text(&s), before);
            assert_eq!(s.remaining_input(Instant::now()), None);
        }
        assert!(
            !std::path::PathBuf::from(format!("/proc/{}", leader.trim())).exists(),
            "{mode}: leader not reaped"
        );
        assert!(stopped(&descendant), "{mode}: descendant running");
        assert!(!Path::new(&home).exists(), "{mode}: private HOME remains");
    }
}
// Selected noweb expansion reuses bounded literal references. Unresolved or
// evaluated references never launch a runner or replace old canonical RESULTS.
#[test]
fn execute_noweb() {
    for (name, reference, state) in [
        ("resolved", "fragment", State::Succeeded),
        ("unresolved", "absent", State::Failed),
        ("evaluated", "fragment()", State::Failed),
    ] {
        let source = format!(
            "#+NAME: fragment\n#+BEGIN_SRC sh\nprintf expanded > marker\n#+END_SRC\n#+BEGIN_SRC sh :noweb eval\n<<{reference}>>\n#+END_SRC\n#+RESULTS:\n: previous\n"
        );
        let config = "[[org.source.programs]]\nname='shell'\nlanguage='sh'\nexecutable='/bin/sh'\ncwd='source'";
        let (f, mut s) = configured(&source, config);
        s.execute(Command::GotoLine);
        argument(&mut s, "6");
        action(&mut s, C::ExecuteSrc);
        argument(&mut s, "yes");
        let out = s.execution_result().unwrap();
        assert_eq!(out.state, state, "{name}: {}", out.detail);
        assert_eq!(f.0.join("marker").exists(), name == "resolved");
        assert_eq!(
            text(&s),
            if name == "resolved" {
                source.replace(": previous", ":")
            } else {
                source
            }
        );
    }
}

// Compile and final-run streams share one output ceiling. A failed intermediate
// step also prevents final code from running, retaining its actual exit/output.
#[test]
fn execute_step_budget() {
    for (name, step, wanted) in [
        ("combined", "printf abc", b"abc".as_slice()),
        ("failed", "printf abc; exit 9", b"abc".as_slice()),
    ] {
        let profile = format!(
            "executable='/bin/sh'\noutput_bytes=5\nsteps=[{{executable='/bin/sh',argv=['-c','{step}']}},{{executable='/bin/sh',argv=['-c','printf def']}}]"
        );
        let (f, mut s) = host_with("printf escaped > marker", &profile, "");
        action(&mut s, C::ExecuteSrc);
        argument(&mut s, "yes");
        let out = s.execution_result().unwrap();
        assert_eq!(out.state, State::Failed, "{name}: {}", out.detail);
        assert_eq!(out.stdout, wanted, "{name}");
        assert!(!f.0.join("marker").exists(), "{name}");
    }
}
