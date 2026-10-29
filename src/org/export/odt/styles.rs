use super::*;
pub(super) fn render(limit: usize, cancel: &Cancellation) -> Result<String, String> {
    let mut xml = Xml::new(limit, cancel);
    xml.start("document-styles")?;
    xml.emit("<office:styles>")?;
    spans(&mut xml)?;
    paragraphs(&mut xml)?;
    lists(&mut xml)?;
    xml.emit("<text:outline-style style:name=\"OrgOutline\">")?;
    for level in 1..=32 {
        xml.emit(&format!("<text:outline-level-style text:level=\"{level}\" style:num-format=\"1\" text:display-levels=\"{level}\" style:num-suffix=\".\"/>"))?;
    }
    xml.emit("</text:outline-style></office:styles></office:document-styles>")?;
    Ok(xml.output)
}
fn paragraphs(xml: &mut Xml<'_>) -> Result<(), String> {
    for (name, property) in [
        ("Title", "fo:font-size=\"20pt\" fo:font-weight=\"bold\""),
        ("Heading", "fo:font-size=\"14pt\" fo:font-weight=\"bold\""),
        ("Literal", "fo:font-family=\"monospace\""),
        ("Caption", "fo:font-style=\"italic\""),
    ] {
        xml.emit(&format!("<style:style style:name=\"{name}\" style:family=\"paragraph\"><style:text-properties {property}/></style:style>"))?;
    }
    Ok(())
}
fn lists(xml: &mut Xml<'_>) -> Result<(), String> {
    for (name, kind, property) in [
        (
            "NumberList",
            "number",
            "style:num-format=\"1\" style:num-suffix=\".\"",
        ),
        ("BulletList", "bullet", "text:bullet-char=\"•\""),
    ] {
        xml.emit(&format!("<text:list-style style:name=\"{name}\">"))?;
        for level in 1..=32 {
            xml.emit(&format!(
                "<text:list-level-style-{kind} text:level=\"{level}\" {property}/>"
            ))?;
        }
        xml.emit("</text:list-style>")?;
    }
    Ok(())
}

fn spans(xml: &mut Xml<'_>) -> Result<(), String> {
    for (name, property) in [
        ("Bold", "fo:font-weight=\"bold\""),
        ("Italic", "fo:font-style=\"italic\""),
        (
            "Underline",
            "style:text-underline-style=\"solid\" style:text-underline-width=\"auto\"",
        ),
        ("Strike", "style:text-line-through-style=\"solid\""),
        ("Code", "fo:font-family=\"monospace\""),
    ] {
        xml.emit(&format!("<style:style style:name=\"{name}\" style:family=\"text\"><style:text-properties {property}/></style:style>"))?;
    }
    Ok(())
}
