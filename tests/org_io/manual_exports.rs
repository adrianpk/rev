use super::*;
use rev::{
    config::{ExportBackend as B, Org},
    org::{
        Cancellation,
        export::{Request, Scope, Work},
    },
};
fn rendered(source: &str, backend: B) -> Result<String, String> {
    let f = Fixture::new();
    let path = f.write("manual.org", source);
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let request = Request {
        backend,
        scope: Scope::Buffer,
        options: None,
        time: clock(&Org::default().time).unwrap(),
    };
    let snapshot =
        Work::new(&buffers, id, request, &Org::default())?.run(&Cancellation::default())?;
    Ok(snapshot.render(&Cancellation::default())?.text().into())
}
// Handwritten expectations cover native envelopes, hierarchy and escaped data;
// neither export path has authority to invoke makeinfo, groff or a shell.
#[test]
fn manual_document() {
    let source = "#+TITLE: Notes @ {home}\n#+OPTIONS: toc:nil num:nil\n* First\nBody *bold* and /italic/.\n** Child\n#+begin_src text\n@end example\n.so secret\n\\X'escape'\n#+end_src\n";
    let tex = rendered(source, B::Texinfo).unwrap();
    let man = rendered(source, B::Man).unwrap();
    for expected in [
        "\\input texinfo\n",
        "@settitle Notes @@ @{home@}",
        "@unnumbered First",
        "@unnumberedsec Child",
        "@strong{bold}",
        "@@end example\n.so secret\n@backslashchar{}X'escape'",
        "@bye\n",
    ] {
        assert!(tex.contains(expected), "{tex}");
    }
    for expected in [
        ".TH \"Notes @ {home}\" \"1\"",
        ".SH \"First\"",
        ".SS \"Child\"",
        "\\fBbold\\fP",
        "\\&.so secret",
        "\\[rs]X\\[aq]escape\\[aq]",
    ] {
        assert!(man.contains(expected), "{man}");
    }
    assert!(!man.contains("\n.so"));
}
