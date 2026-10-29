use super::*;
use rev::{
    config::{ExportBackend as B, Org},
    org::{
        Cancellation,
        export::{Request, Scope, Work},
    },
};
fn package(source: &str, limit: usize) -> Result<Vec<u8>, String> {
    let f = Fixture::new();
    let path = f.write("source.org", source);
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let mut policy = Org::default();
    policy.limits.export_bytes = limit;
    let request = Request {
        backend: B::Odt,
        scope: Scope::Buffer,
        options: None,
        time: clock(&policy.time).unwrap(),
    };
    let snapshot = Work::new(&buffers, id, request, &policy)?.run(&Cancellation::default())?;
    Ok(snapshot.package(&Cancellation::default())?.bytes().into())
}
// Python's independent ZIP/CRC and namespace-aware XML readers check the actual
// binary output, including the special first uncompressed mimetype record.
#[test]
fn odt_package_envelope() {
    let f = Fixture::new();
    let source = "#+TITLE: Notes & plans\nText <tag> 界 and é\n";
    let bytes = package(source, 16777216).unwrap();
    assert_eq!(bytes, package(source, 16777216).unwrap());
    let path = f.0.join("document.odt");
    std::fs::write(&path, bytes).unwrap();
    let check = std::process::Command::new("python3").args(["-c", r#"import pathlib, sys, zipfile, xml.etree.ElementTree as E
p = pathlib.Path(sys.argv[1]); data = p.read_bytes()
with zipfile.ZipFile(p) as z:
    assert z.namelist() == ['mimetype', 'content.xml', 'styles.xml', 'meta.xml', 'META-INF/manifest.xml']
    assert z.testzip() is None
    first = z.infolist()[0]
    assert first.header_offset == 0 and first.compress_type == zipfile.ZIP_STORED
    assert first.extra == b'' and data[28:30] == b'\x00\x00'
    mime = z.read('mimetype').decode('ascii')
    assert mime == 'application/vnd.oasis.opendocument.text'
    assert data[38:38+len(mime)] == mime.encode()
    for name in z.namelist()[1:]: E.fromstring(z.read(name))
    ns = {'m': 'urn:oasis:names:tc:opendocument:xmlns:manifest:1.0', 'dc': 'http://purl.org/dc/elements/1.1/'}
    manifest = E.fromstring(z.read('META-INF/manifest.xml'))
    entries = {e.attrib['{'+ns['m']+'}full-path']: e.attrib['{'+ns['m']+'}media-type'] for e in manifest}
    assert entries == {'/': mime, 'content.xml': 'text/xml', 'styles.xml': 'text/xml', 'meta.xml': 'text/xml'}
    assert E.fromstring(z.read('meta.xml')).find('.//dc:title', ns).text == 'Notes & plans'
"#, path.to_str().unwrap()]).output().unwrap();
    assert!(
        check.status.success(),
        "{}",
        String::from_utf8_lossy(&check.stderr)
    );
}
// XML-invalid scalars and a finite final package budget reject output rather
// than substituting data or returning a partial or malformed archive.
#[test]
fn odt_package_bounds() {
    for source in ["invalid \u{1} scalar\n", "invalid \u{fffe} scalar\n"] {
        assert!(
            package(source, 16777216)
                .unwrap_err()
                .contains("ODT XML cannot represent")
        );
    }
    assert!(package("body\n", 100).is_err());
}
