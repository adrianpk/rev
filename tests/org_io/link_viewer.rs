use super::links::editor;
use super::*;
// Explicit follow alone never starts an external resource. Confirmation and
// literal argv preserve source bytes and avoid shell interpolation of targets.
#[test]
fn links_viewer_confirmation() {
    let uri = "https://example.invalid/$((1+2))?q=a%20b";
    let source = format!("[[{uri}][Reference 界]]\n");
    let policy = "org.tools.viewer=['/bin/sh','viewer.sh','{target}']";
    let (f, mut s) = editor(policy, &source, "");
    f.write(
        "viewer.sh",
        "printf '%s' \"$1\" > called; pwd > directory; printf 'opened'; printf 'detail' >&2\n",
    );
    let stamp = s.buffer().unwrap().stamp();
    action(&mut s, C::OpenLink);
    assert!(s.echo().contains("example.invalid"), "{}", s.echo());
    assert!(!f.0.join("called").exists());
    argument(&mut s, "no");
    assert!(!f.0.join("called").exists());
    action(&mut s, C::OpenLink);
    cancel(&mut s);
    settle(&mut s);
    assert!(!f.0.join("called").exists());
    action(&mut s, C::OpenLink);
    argument(&mut s, "yes");
    assert!(s.echo().contains("Viewer completed"), "{}", s.echo());
    assert!(s.echo().contains("openeddetail"), "{}", s.echo());
    assert_eq!(std::fs::read_to_string(f.0.join("called")).unwrap(), uri);
    assert_eq!(
        std::fs::read_to_string(f.0.join("directory"))
            .unwrap()
            .trim(),
        f.0.to_str().unwrap()
    );
    assert_eq!(s.buffer().unwrap().stamp(), stamp);
    assert_eq!(text(&s), source);
    assert!(s.buffer().unwrap().history().is_empty());
    assert_eq!(
        std::fs::read_to_string(f.0.join("source.org")).unwrap(),
        source
    );
}
// Missing/disabled programs, nonzero status and output overflow are observable
// failures. Every case leaves the real source, point and history untouched.
#[test]
fn links_viewer_failures() {
    for (name, policy, expected) in [
        ("disabled", "", "No external viewer configured"),
        (
            "missing",
            "org.tools.viewer=['./absent']",
            "Start configured program",
        ),
        (
            "status",
            "org.tools.viewer=['/bin/sh','-c','exit 9']",
            "Viewer failed",
        ),
        (
            "output",
            "org.tools.viewer=['/bin/sh','-c','printf abcdef']\norg.limits.job_output_bytes=5",
            "output limit",
        ),
    ] {
        let (f, mut s) = editor(policy, "[[https://example.invalid/]]\n", "");
        let origin = s.view().point;
        action(&mut s, C::OpenLink);
        if !policy.is_empty() {
            argument(&mut s, "yes");
        }
        assert!(s.echo().contains(expected), "{name}: {}", s.echo());
        assert_eq!(s.view().point, origin, "{name}");
        assert!(s.buffer().unwrap().history().is_empty());
        assert_eq!(
            std::fs::read_to_string(f.0.join("source.org")).unwrap(),
            text(&s)
        );
        assert!(s.remaining_input(Instant::now()).is_none());
    }
}
// Explicit system/file media links pass an existing encoded path as one argv
// value. Native overrides keep their existing visit path; missing media creates
// no file and never invokes a configured viewer.
#[test]
fn links_media() {
    for (name, target) in [
        ("media", "file:picture%20one.png"),
        ("system", "file+sys:picture%20one.png"),
    ] {
        let (f, mut s) = editor(
            "org.tools.viewer=['/bin/sh','viewer.sh']",
            &format!("[[{target}]]\n"),
            "",
        );
        f.write("viewer.sh", "printf '%s' \"$1\" > called\n");
        let path = f.write("picture one.png", "native content\n");
        action(&mut s, C::OpenLink);
        assert!(!f.0.join("called").exists(), "{name}");
        argument(&mut s, "yes");
        assert!(
            s.echo().contains("Viewer completed"),
            "{name}: {}",
            s.echo()
        );
        assert_eq!(
            std::fs::read_to_string(f.0.join("called")).unwrap(),
            path.to_str().unwrap()
        );
    }
    let (f, mut s) = editor("", "[[file+emacs:picture.png]]\n", "");
    f.write("picture.png", "native text\n");
    action(&mut s, C::OpenLink);
    assert!(s.echo().contains("Org link destination"), "{}", s.echo());
    assert_eq!(text(&s), "native text\n");
    let (f, mut s) = editor(
        "org.tools.viewer=['./absent']",
        "[[file:missing.png]]\n",
        "",
    );
    action(&mut s, C::OpenLink);
    argument(&mut s, "yes");
    assert!(s.echo().contains("does not exist"), "{}", s.echo());
    assert!(!f.0.join("missing.png").exists());
}
