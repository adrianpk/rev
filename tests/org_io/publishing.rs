use super::*;
use rev::file::Publication;
fn project(extra: &str) -> (Fixture, Session) {
    let configuration = format!(
        r#"
org.time.zone = "UTC"
{extra}
[[org.publish.projects]]
name = "docs"
roots = ["."]
include = ["source.org"]
output = "site"
formats = ["html", "markdown"]
assets = ["assets/**"]
"#
    );
    let (f, s) = super::links::editor(
        &configuration,
        "\u{feff}* Heading 界\r\nBody\r\n",
        "* Other\n",
    );
    (f, s.with_org_clock(clock))
}
fn publish(s: &mut Session) {
    action(s, C::Publish);
    assert!(s.echo().contains("Select"), "{}", s.echo());
    argument(s, "docs");
}
// Publishing observes canonical unsaved source and copies selected binary assets,
// while BOM/CRLF disk bytes, history and unowned output files remain untouched.
#[test]
fn project_native_outputs() {
    let (f, mut s) = project("");
    f.write("assets/picture.bin", "asset");
    std::fs::write(f.0.join("assets/picture.bin"), [0, 255, 13, 10]).unwrap();
    f.write("site/unowned", "keep");
    let disk = std::fs::read(f.0.join("source.org")).unwrap();
    s.handle(Event::Paste("unsaved ".into()));
    let before = text(&s);
    let stamp = s.buffer().unwrap().stamp();
    publish(&mut s);
    assert_eq!(
        s.publishing_results().len(),
        3,
        "{} {:?}",
        s.echo(),
        s.publishing_results()
    );
    assert!(
        s.publishing_results()
            .iter()
            .all(|r| r.publication == Publication::Complete),
        "{:?}",
        s.publishing_results()
    );
    let output = std::fs::read_to_string(f.0.join("site/source.md")).unwrap();
    assert!(output.contains("unsaved"));
    assert_eq!(
        std::fs::read(f.0.join("site/assets/picture.bin")).unwrap(),
        [0, 255, 13, 10]
    );
    assert_eq!(
        std::fs::read_to_string(f.0.join("site/unowned")).unwrap(),
        "keep"
    );
    assert_eq!(std::fs::read(f.0.join("source.org")).unwrap(), disk);
    assert_eq!(s.buffer().unwrap().stamp(), stamp);
    assert_eq!(text(&s), before);
    assert!(s.buffer().unwrap().is_modified());
    s.execute(Command::History(rev::command::History::Undo));
    assert!(!text(&s).contains("unsaved"));
}
