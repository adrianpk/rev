use super::fixture::Fixture;
use rev::config::{Snapshot, prepare};
// Literal argv layers replace as a leaf, including an explicit empty disable.
// Invalid substitutions/output bounds reject the whole configuration candidate.
#[test]
fn links_configuration() {
    let f = Fixture::new();
    let base = f.write(
        "base.toml",
        "org.tools.viewer=['viewer','{target}']\norg.limits.job_output_bytes=8",
    );
    let root = f.write(
        "config.toml",
        "include=['base.toml']\norg.attachments.directory='assets'",
    );
    let s = prepare(&root);
    assert!(s.diagnostic.is_none(), "{:?}", s.diagnostic);
    assert_eq!(
        s.snapshot.settings().org.tools.viewer,
        ["viewer", "{target}"]
    );
    assert_eq!(s.snapshot.settings().org.limits.job_output_bytes, 8);
    assert_eq!(
        s.snapshot.origin(&["org", "tools", "viewer"]),
        Some(base.as_path())
    );
    f.write("config.toml", "include=['base.toml']\norg.tools.viewer=[]");
    assert!(
        prepare(&root)
            .snapshot
            .settings()
            .org
            .tools
            .viewer
            .is_empty()
    );
    for (name, source) in [
        ("empty executable", "org.tools.viewer=['']"),
        (
            "partial token",
            "org.tools.viewer=['viewer','prefix{target}']",
        ),
        ("executable target", "org.tools.viewer=['{target}']"),
        (
            "two targets",
            "org.tools.viewer=['viewer','{target}','{target}']",
        ),
        ("zero", "org.limits.job_output_bytes=0"),
        ("negative", "org.limits.job_output_bytes=-1"),
        ("maximum", "org.limits.job_output_bytes=16777217"),
    ] {
        let path = f.write("invalid.toml", source);
        let s = prepare(&path);
        assert!(s.diagnostic.is_some(), "{name}");
        assert_eq!(s.snapshot, Snapshot::default(), "{name}");
    }
}

// PDF argv is validated as data during layered configuration, with no program
// lookup or launch. A leaf override can explicitly disable an inherited tool.
#[test]
fn pdf_configuration() {
    let f = Fixture::new();
    let base = f.write(
        "pdf/base.toml",
        "org.tools.pdf=['./absent','{input}','{output}','{output_dir}']",
    );
    let root = f.write("config.toml", "include=['pdf/base.toml']");
    let prepared = prepare(&root);
    assert!(prepared.diagnostic.is_none(), "{:?}", prepared.diagnostic);
    assert_eq!(
        prepared.snapshot.settings().org.tools.pdf,
        ["./absent", "{input}", "{output}", "{output_dir}"]
    );
    assert_eq!(
        prepared.snapshot.origin(&["org", "tools", "pdf"]),
        Some(base.as_path())
    );
    f.write("config.toml", "include=['pdf/base.toml']\norg.tools.pdf=[]");
    assert!(prepare(&root).snapshot.settings().org.tools.pdf.is_empty());
    for (name, tokens) in [
        ("empty token", "['','{input}']"),
        ("absent input", "['compiler']"),
        ("executable placeholder", "['{input}']"),
        ("embedded placeholder", "['compiler','x{input}']"),
        ("duplicate input", "['compiler','{input}','{input}']"),
        (
            "duplicate output",
            "['compiler','{input}','{output}','{output}']",
        ),
        ("unknown placeholder", "['compiler','{input}','{unknown}']"),
    ] {
        f.write("config.toml", &format!("org.tools.pdf={tokens}"));
        let prepared = prepare(&root);
        assert!(prepared.diagnostic.is_some(), "{name}");
        assert_eq!(prepared.snapshot, Snapshot::default(), "{name}");
    }
}
