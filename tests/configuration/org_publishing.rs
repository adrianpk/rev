use super::fixture::Fixture;
use rev::config::{ExportBackend, prepare};
const PROJECT: &str =
    "[[org.publish.projects]]\nname='docs'\nroots=['notes']\noutput='site'\nformats=['html']\n";
// Typed projects retain assigning-file origins and replace arrays as one leaf.
// Configuration loading performs no discovery, directory creation or upload.
#[test]
fn publishing_configuration() {
    let f = Fixture::new();
    let base = f.write(
        "nested/base.toml",
        &format!("org.tools.upload=['missing-program','{{output_dir}}']\n{PROJECT}"),
    );
    let config = f.write("config.toml", "include=['nested/base.toml']\n");
    let startup = prepare(&config);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let policy = &startup.snapshot.settings().org;
    assert_eq!(policy.publish.projects.len(), 1);
    assert_eq!(policy.publish.projects[0].include, vec!["**/*.org"]);
    assert_eq!(
        policy.publish.projects[0].formats,
        vec![ExportBackend::Html]
    );
    assert!(!policy.publish.projects[0].upload);
    assert_eq!(
        startup.snapshot.origin(&["org", "publish", "projects"]),
        Some(base.as_path())
    );
    assert!(!f.0.join("nested/site").exists());
    assert!(!f.0.join("nested/notes").exists());
    f.write(
        "config.toml",
        "include=['nested/base.toml']\norg.publish.projects=[]\norg.tools.upload=[]\n",
    );
    let startup = prepare(&config);
    assert!(startup.diagnostic.is_none());
    assert!(startup.snapshot.settings().org.publish.projects.is_empty());
    assert!(startup.snapshot.settings().org.tools.upload.is_empty());
}
// Closed records reject ambiguous projects, unsupported patterns/formats and
// executable substitutions before any filesystem selection or worker admission.
#[test]
fn publishing_rejections() {
    let f = Fixture::new();
    let cases = [
        ("unknown field", format!("{PROJECT}hook='shell'")),
        ("duplicate name", format!("{PROJECT}{PROJECT}")),
        ("empty name", PROJECT.replace("name='docs'", "name=''")),
        (
            "empty roots",
            PROJECT.replace("roots=['notes']", "roots=[]"),
        ),
        (
            "remote root",
            PROJECT.replace("roots=['notes']", "roots=['https://host/file.org']"),
        ),
        (
            "traversal",
            PROJECT.replace("output='site'", "output='../site'"),
        ),
        (
            "empty formats",
            PROJECT.replace("formats=['html']", "formats=[]"),
        ),
        (
            "duplicate format",
            PROJECT.replace("formats=['html']", "formats=['html','html']"),
        ),
        (
            "unsupported format",
            PROJECT.replace("formats=['html']", "formats=['pdf']"),
        ),
        ("absolute pattern", format!("{PROJECT}include=['/*.org']")),
        (
            "traversal pattern",
            format!("{PROJECT}exclude=['../*.org']"),
        ),
        ("shell pattern", format!("{PROJECT}assets=['[ab].png']")),
        (
            "name bound",
            PROJECT.replace("name='docs'", &format!("name='{}'", "a".repeat(129))),
        ),
        (
            "root bound",
            PROJECT.replace(
                "roots=['notes']",
                &format!("roots=[{}]", vec!["'notes'"; 65].join(",")),
            ),
        ),
        (
            "pattern bound",
            format!("{PROJECT}include=['{}']", "a".repeat(257)),
        ),
        ("empty argv", "org.tools.upload=['']".into()),
        (
            "embedded substitution",
            "org.tools.upload=['program','prefix{output_dir}']".into(),
        ),
        (
            "executable substitution",
            "org.tools.upload=['{output_dir}']".into(),
        ),
        (
            "unknown substitution",
            "org.tools.upload=['program','{input}']".into(),
        ),
    ];
    for (name, source) in cases {
        let startup = prepare(&f.write("config.toml", &source));
        assert!(startup.diagnostic.is_some(), "{name}");
        assert!(!f.0.join("site").exists(), "{name}");
    }
}
