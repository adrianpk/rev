//! Native OpenDocument packages have no file, tool, asset or network authority.
mod xml;
mod zip;
use super::*;
use xml::Xml;
const MIME: &str = "application/vnd.oasis.opendocument.text";
/// One complete bounded ODT package. Text rendering remains a separate API.
pub struct Archive {
    pub(super) bytes: Vec<u8>,
    pub(super) diagnostics: Vec<String>,
}
impl Archive {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }
}
impl Snapshot {
    /// Package the selected ODT snapshot without creating files or running tools.
    /// Other backends retain their text renderer rather than an implicit conversion.
    pub fn package(&self, cancel: &Cancellation) -> Result<Archive, String> {
        if self.request.backend != ExportBackend::Odt {
            return Err("Native package rendering requires the ODT backend".into());
        }
        let cancel = cancel.bounded(Instant::now() + Duration::from_secs(self.job_seconds as u64));
        document::check(&cancel)?;
        let content = content(self, &cancel)?;
        let styles = styles(self.export_bytes, &cancel)?;
        let meta = metadata(self, &cancel)?;
        let manifest = manifest(self.export_bytes, &cancel)?;
        let members = [
            ("mimetype", MIME.as_bytes()),
            ("content.xml", content.as_bytes()),
            ("styles.xml", styles.as_bytes()),
            ("meta.xml", meta.as_bytes()),
            ("META-INF/manifest.xml", manifest.as_bytes()),
        ];
        let bytes = zip::package(members, self.export_bytes, &cancel)?;
        Ok(Archive {
            bytes,
            diagnostics: self.diagnostics.clone(),
        })
    }
}
fn content(snapshot: &Snapshot, cancel: &Cancellation) -> Result<String, String> {
    let mut xml = Xml::new(snapshot.export_bytes, cancel);
    xml.start("document-content")?;
    xml.emit("<office:body><office:text><text:p>")?;
    xml.prose(&snapshot.text.to_string())?;
    xml.emit("</text:p></office:text></office:body></office:document-content>")?;
    Ok(xml.output)
}
fn styles(limit: usize, cancel: &Cancellation) -> Result<String, String> {
    let mut xml = Xml::new(limit, cancel);
    xml.start("document-styles")?;
    xml.emit("<office:styles/></office:document-styles>")?;
    Ok(xml.output)
}
fn metadata(snapshot: &Snapshot, cancel: &Cancellation) -> Result<String, String> {
    let mut xml = Xml::new(snapshot.export_bytes, cancel);
    xml.start("document-meta")?;
    xml.emit("<office:meta>")?;
    let m = &snapshot.metadata;
    for (tag, value, enabled) in [
        ("dc:title", &m.title, snapshot.options.title),
        ("dc:creator", &m.author, snapshot.options.author),
        ("dc:language", &m.language, true),
    ] {
        if let Some(value) = value.as_ref().filter(|_| enabled) {
            xml.emit(&format!("<{tag}>"))?;
            xml.text(value)?;
            xml.emit(&format!("</{tag}>"))?;
        }
    }
    xml.emit("</office:meta></office:document-meta>")?;
    Ok(xml.output)
}
fn manifest(limit: usize, cancel: &Cancellation) -> Result<String, String> {
    let mut xml = Xml::new(limit, cancel);
    xml.emit("<?xml version=\"1.0\" encoding=\"UTF-8\"?><manifest:manifest xmlns:manifest=\"urn:oasis:names:tc:opendocument:xmlns:manifest:1.0\" manifest:version=\"1.3\">")?;
    for (path, media) in [
        ("/", MIME),
        ("content.xml", "text/xml"),
        ("styles.xml", "text/xml"),
        ("meta.xml", "text/xml"),
    ] {
        xml.emit(&format!(
            "<manifest:file-entry manifest:full-path=\"{path}\" manifest:media-type=\"{media}\"/>"
        ))?;
    }
    xml.emit("</manifest:manifest>")?;
    Ok(xml.output)
}
